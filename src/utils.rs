//! Random utilities, including an assortment of strings as arrays of char traits
//! that can be implemented on any string-like type.

use std::ops::Range;

/// Trait that implements operations which allows operations on characters
/// rather than on bytes.
/// As thus treating string as arrays of characters, instead of bytes.
pub trait CharOps: AsRef<str> {
    /// The type to which `Self` can be sliced.
    /// Should be &str for most uses.
    type SliceType: AsRef<str>;
    /// Creates a slice from a string.
    /// This should be equivalent to `get`, but for support for custom
    /// slice types, this method *exists*,
    /// also for parity with the char version.
    /// Returns `None`, if the range is out of bounds.
    fn byte_slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType>;
    /// Creates a slice from a string.
    /// The `char_range` is a range of **characters**, not bytes like `get`.
    /// Returns `None`, if the range is out of bounds.
    fn char_slice(&self, char_range: Range<usize>) -> Option<Self::SliceType> {
        let byte_range = self.char_to_byte_range(char_range)?;
        self.byte_slice(byte_range)
    }
    /// Returns the equivalent char range of a byte range on this string.
    /// May return `None` if the `byte_range` is out of bounds.
    fn byte_to_char_range(&self, byte_range: Range<usize>) -> Option<Range<usize>>;
    /// Returns the equivalent byte range of a char range on this string.
    /// May return `None` if the `byte_range` is out of bounds.
    fn char_to_byte_range(&self, char_range: Range<usize>) -> Option<Range<usize>>;
}

impl<'a> CharOps for &'a str {
    type SliceType = &'a str;

    fn char_to_byte_range(&self, char_range: Range<usize>) -> Option<Range<usize>> {
        let mut byte_start_index = None;
        let mut byte_end_index= None;
        let mut char_index = 0;

        if char_range.start == 0 {
            byte_start_index = Some(0);
        }

        // okay... this is weird if this happens
        if char_range.end == 0 {
            return Some(0..0)
        }

        // NOTE: This may be laggy
        for byte_index in 1..=self.as_bytes().len() {
            if !self.is_char_boundary(byte_index) {
                continue
            }

            char_index += 1;

            if char_index == char_range.start {
                byte_start_index = Some(byte_index);
            }

            if char_index == char_range.end {
                byte_end_index = Some(byte_index);
                break;
            }
        };

        if let (Some(s), Some(e)) = (byte_start_index, byte_end_index) {
            Some(s..e)
        } else {
            None
        }
    }

    fn byte_slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        self.get(byte_range)
    }

    fn byte_to_char_range(&self, byte_range: Range<usize>) -> Option<Range<usize>> {
        let mut char_start_index = None;
        let mut char_end_index= None;

        if byte_range.end == 0 {
            return Some(0..0)
        }

        // the enumerator does not go until the end so we do this check
        if byte_range.end == self.len() {
            char_end_index = Some(self.chars().count());
        }

        // NOTE: This may be laggy too
        for (char_index, (byte_index, _)) in self.char_indices().enumerate() {
            if byte_range.start == byte_index {
                char_start_index = Some(char_index);
            }

            if byte_range.end == byte_index {
                char_end_index = Some(char_index);
                break;
            }
        };


        let (char_start_index, char_end_index) = { 
            if let (Some(s), Some(e)) = (char_start_index, char_end_index) {
                (s, e)
            } else {
                return None;
            }
        };

        Some(char_start_index..char_end_index)
    }
}

impl<'a> CharOps for &'a String {
    type SliceType = &'a str;

    fn byte_slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        self.as_str().byte_slice(byte_range)
    }

    fn byte_to_char_range(&self, byte_range: Range<usize>) -> Option<Range<usize>> {
        self.as_str().byte_to_char_range(byte_range)
    }

    fn char_to_byte_range(&self, char_range: Range<usize>) -> Option<Range<usize>> {
        self.as_str().char_to_byte_range(char_range)
    }
}

/// Trait implementing utilities to find the `(ln, col)` char position
/// of an element in a string.
pub trait FindLnCol: CharOps {
    /// Returns the `(ln, col)` position in char of the `nth` byte.
    /// May return `None` if the character is not in the string.
    /// # Note
    /// This might not meet your preconseptions of lines and columns.
    /// Lines and columns start counting from one, not from zero.
    /// ```
    /// use basm::utils::FindLnCol;
    /// 
    /// // Lines and columns start counting from 1.
    /// assert_eq!("".byte_find_ln_col(0), Some((1, 1)));
    /// // Do NOT be fooled!
    /// assert_ne!("".byte_find_ln_col(0), Some((0, 0)));
    /// ```
    fn byte_find_ln_col(&self, nth_byte: usize) -> Option<(usize, usize)> {
        let string = self.as_ref();
        let sub_string = string.get(0..nth_byte)?;
        let new_lines = sub_string.char_indices()
            .filter(|(_, c)| *c == '\n');

        let mut line = new_lines.clone().count();
        let last_nl_byte_index = new_lines.map(|(i, _)| i)
            .last();
        // because \n is one byte long and we don't want to include it
        let start_line_byte_index = last_nl_byte_index.map_or(0, |b| b + 1);

        let mut column = string[start_line_byte_index..nth_byte].len();

        // line and column are zero-indexed right now, let's switch
        // them to the more common one-index
        line += 1;
        column += 1;

        Some((line, column))
    }

    /// Returns the `(ln, col)` position in char of the `nth` char.
    /// May return `None` if the character is not in the string.
    fn char_find_ln_col(&self, nth_char: usize) -> Option<(usize, usize)> {
        let byte = self.char_to_byte_range(nth_char..nth_char)?
            .start;
        self.byte_find_ln_col(byte)
    }
}

impl<T: CharOps + ?Sized> FindLnCol for T {}

/// Allows to check whether or not something is alphanumeric, alphabetic or numeric.
pub trait IsAlphanumeric {
    /// Returns `true` if it is alphanumeric.
    fn is_alphanumeric(&self) -> bool;
    /// Returns `true` if it is alphabetic.
    fn is_alphabetic(&self) -> bool;
    /// Returns `true` if it is numeric.
    fn is_numeric(&self) -> bool;
}

impl<T: AsRef<str>> IsAlphanumeric for T {
    fn is_alphanumeric(&self) -> bool {
        let string= self.as_ref();
        if string.is_empty() { return false }

        for ch in string.chars() {
            if !ch.is_alphanumeric() {
                return false;
            }
        }

        true
    }

    fn is_alphabetic(&self) -> bool {
        let string= self.as_ref();
        if string.is_empty() { return false }

        for ch in string.chars() {
            if !ch.is_alphabetic() {
                return false;
            }
        }

        true
    }

    fn is_numeric(&self) -> bool {
        let string= self.as_ref();
        if string.is_empty() { return false }

        for ch in string.chars() {
            if !ch.is_numeric() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_get_empty() {
        assert_eq!("".char_slice(0..0), Some(""));
        assert!("".char_slice(0..98).is_none())
    }

    #[test]
    fn char_get_oob() {
        assert_eq!(
            "my fw the love inside you will stop indefinitly and till then envisions the world that you wish to see and we will help in any way we can and we will help in anyway we cannn and we will will help in anyway we cannnn [music]"
            .char_slice(50..732),
            None
        );
        assert_eq!(
            "my fw the love inside you will stop indefinitly and till then envisions the world that you wish to see and we will help in any way we can and we will help in anyway we cannn and we will will help in anyway we cannnn [music]"
            .char_slice(732..2024),
            None
        );
    }

    #[test]
    fn char_get_normal_behaviour() {
        assert_ne!("🇭ello, 🇭i!".get(0..5),      Some("🇭ello"));
        assert_eq!("🇭ello, 🇭i!".char_slice(0..5), Some("🇭ello"));

        assert_eq!("ĥello, ĥi!".char_slice(0..5), Some("ĥello"));

        // from the str::get doc
        assert_eq!("🗻∈🌏".char_slice(1..3), Some("∈🌏"));
        assert_eq!("🗻∈🌏".char_slice(0..1), Some("🗻"))
    }

    #[test]
    fn byte_to_char_range_empty() {
        assert_eq!("".byte_to_char_range(0..0), Some(0..0));
        assert_eq!("jimmy ĥ↑mert".byte_to_char_range(11..11), Some(8..8));
    }

    #[test]
    fn byte_to_char_range_oob() {
        assert_eq!("".byte_to_char_range(10..11), None);
        assert_eq!("jimmy ĥ↑mert".byte_to_char_range(72..10), None);
    }

    #[test]
    fn byte_to_char_range_normal() {
        assert_eq!("y̆es".byte_to_char_range(0..3), Some(0..2));
        assert_eq!("y̆es".byte_to_char_range(0..4), Some(0..3));
        assert_eq!("🇭 🇭ello, 🇭i!".byte_to_char_range(0..10), Some(0..4));
    }

    #[test]
    fn is_alphanumeric() {
        assert!(!"".is_alphanumeric());
        assert!("Joe".is_alphanumeric());
        assert!("732".is_alphanumeric());
        assert!("732Conspiracy".is_alphanumeric());
        assert!(!"the 732 Conspiracy".is_alphanumeric());
        assert!(!".".is_alphanumeric());
        assert!(!"_".is_alphanumeric());
    }

    #[test]
    fn is_alphabetic() {
        assert!(!"".is_alphabetic());
        assert!("Joe".is_alphabetic());
        assert!(!"732".is_alphabetic());
        assert!(!"732Conspiracy".is_alphabetic());
        assert!(!"the 732 Conspiracy".is_alphabetic());
        assert!(!".".is_alphabetic());
        assert!(!"_".is_alphabetic());
    }

    #[test]
    fn is_numeric() {
        assert!(!"".is_numeric());
        assert!(!"Joe".is_numeric());
        assert!("732".is_numeric());
        assert!(!"732Conspiracy".is_numeric());
        assert!(!"the 732 Conspiracy".is_numeric());
        assert!(!".".is_numeric());
        assert!(!"_".is_numeric());
    }
}
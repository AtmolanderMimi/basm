//! Random utilities.

use std::ops::Range;

/// Trait that implements operations which allows slicing string-like types with `Range<usize>`.
pub trait Sliceable: AsRef<str> {
    /// The type to which `Self` can be sliced.
    /// Should be &str for most uses.
    type SliceType: AsRef<str>;
    /// Creates a slice from a string.
    /// This should be equivalent to `get`, but for support for custom
    /// slice types, this method *exists*,
    /// also for parity with the char version.
    /// Returns `None`, if the range is out of bounds.
    fn slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType>;
}

impl<'a> Sliceable for &'a str {
    type SliceType = &'a str;

    fn slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        self.get(byte_range)
    }
}

impl<'a> Sliceable for &'a String {
    type SliceType = &'a str;

    fn slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        self.as_str().slice(byte_range)
    }
}

/// Trait implementing utilities to find the `(ln, col)` char position
/// of an element in a string.
pub trait FindLnCol: Sliceable {
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
            .next_back();
        // because \n is one byte long and we don't want to include it
        let start_line_byte_index = last_nl_byte_index.map_or(0, |b| b + 1);

        let mut column = string[start_line_byte_index..nth_byte].len();

        // line and column are zero-indexed right now, let's switch
        // them to the more common one-index
        line += 1;
        column += 1;

        Some((line, column))
    }
}

impl<T: Sliceable + ?Sized> FindLnCol for T {}

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
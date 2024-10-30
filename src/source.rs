use std::{borrow::Cow, fs, io, ops::Range, path::{Path, PathBuf}};

use thiserror::Error;

use crate::error::CompilerError;
use crate::utils::CharOps;

/// Represents a bfu source code file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceFile {
    contents: String,
    absolute_path: PathBuf, // do we really need the path?
}

impl SourceFile {
    pub fn from_raw_parts(absolute_path: PathBuf, contents: String) -> SourceFile {
        SourceFile {
            absolute_path,
            contents,
        }
    }

    /// Creates a new [SourceFile] from a file.
    /// 
    /// Errors out if `absolute_path` is not absolute.
    pub fn from_file(absolute_path: impl AsRef<Path>) -> Result<SourceFile, SourceFileError> {
        let absolute_path = absolute_path.as_ref();
        if !absolute_path.is_absolute() {
            return Err(SourceFileError::NotAbsPath(absolute_path.to_path_buf()));
        }

        let contents = match fs::read_to_string(&absolute_path) {
            Ok(c) => c,
            Err(error) => {
                return Err(SourceFileError::OpeningSourceFile {
                    error,
                    path: absolute_path.to_path_buf() 
                })
            }
        };

        Ok(SourceFile {
            contents,
            absolute_path: absolute_path.to_path_buf(),
        })
    }
}

impl<'a> CharOps<'a> for SourceFile {
    type SliceType = SfSlice<'a>;

    fn byte_slice(&'a self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        let char_range = self.byte_to_char_range(byte_range)?;
        SfSlice::from_source(self, char_range)
    }

    fn char_slice(&'a self, char_range: Range<usize>) -> Option<Self::SliceType> {
        SfSlice::from_source(self, char_range)
    }

    fn byte_to_char_range(&self, byte_range: Range<usize>) -> Option<Range<usize>> {
        self.contents.byte_to_char_range(byte_range)
    }

    fn char_to_byte_range(&self, char_range: Range<usize>) -> Option<Range<usize>> {
        self.contents.char_to_byte_range(char_range)
    }
}

/// A slice of a [SourceFile]. It contains information about its position.
#[derive(Debug, Clone, PartialEq)]
pub struct SfSlice<'a> {
    pub source: Cow<'a, SourceFile>,
    slice_char_range: Range<usize>,
    // will be none if source is owned, because we can't reference self in self
    slice: Option<&'a str>,
}

impl<'a> SfSlice<'a> {
    /// Creates a [SfSlice] from a [SourceFile].
    /// The `range` is by characters, not bytes.
    pub fn from_source(source: &SourceFile, char_range: Range<usize>) -> Option<SfSlice> {
        let slice = if let Some(s) = source.contents.char_slice(char_range.clone()) {
            s
        } else {
            return None;
        };

        Some(SfSlice {
            source: Cow::Borrowed(source),
            slice_char_range: char_range,
            slice: Some(slice),
        })
    }

    #[cfg(test)]
    /// Creates a new slice, just for testing purposes.
    pub fn new_bogus(contents: &str) -> SfSlice<'static> {
        let sf = SourceFile::from_raw_parts(PathBuf::new(), contents.to_string());
        let slice = sf.char_slice(0..(contents.chars().count())).unwrap();
        slice.to_owned()
    }

    /// Returns the offset from the start of the source file this slice is referencing.
    pub fn offset(&self) -> usize {
        self.slice_char_range.start
    }

    /// Transforms this type into its owned form ('static).
    pub fn to_owned(self) -> SfSlice<'static> {
        let owned_source = self.source.into_owned();
        let slice_char_range = self.slice_char_range;

        SfSlice {
            source: Cow::Owned(owned_source),
            slice_char_range,
            slice: None,
        }
    }

    /// Returns the range of characters into the source of
    /// this slice.
    /// This start char range is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn char_range(&self) -> Range<usize> {
        self.slice_char_range.clone()
    }

    /// Returns the start index of the range of characters 
    /// into the source of this slice.
    /// This start position is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn start(&self) -> usize {
        self.slice_char_range.start
    }

    /// Returns the end index of the range of characters 
    /// into the source of this slice.
    /// This end position is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn end(&self) -> usize {
        self.slice_char_range.end
    }

    /// Returns the equivalent string slice.
    pub fn inner_slice(&self) -> &str {
        match self.slice {
            Some(s) => s,
            None => {
                self.source.contents.char_slice(self.slice_char_range.clone())
                    .unwrap()
            }
        }
    }

    /// Transforms a relative range into this slice into an absolute range of the source file.
    /// `rel_range` is in **characters**.
    /// Returns `None` if  `rel_range` is not within the range of this slice.
    pub fn relative_char_to_absolute_range(&self, rel_range: Range<usize>) -> Option<Range<usize>> {
        let abs_end = rel_range.end + self.offset();
        if abs_end > self.slice_char_range.end {
            return None;
        }

        Some((rel_range.start+self.offset())..(rel_range.end+self.offset()))
    }

    /// Transforms a byte range to a char range and then transforms that
    /// into an absolute range relative to the source file.
    /// `rel_byte_range` is in bytes and **the return value is in chars**.
    pub fn relative_byte_to_absolute_range(
        &self,
        rel_byte_range: Range<usize>,
    ) -> Option<Range<usize>> {
        let rel_range = self.byte_to_char_range(rel_byte_range)?;
        let abs_end = rel_range.end + self.offset();
        if abs_end > self.slice_char_range.end {
            return None;
        }

        Some((rel_range.start+self.offset())..(rel_range.end+self.offset()))
    }
}

impl<'a> CharOps<'a> for SfSlice<'a> {
    type SliceType = SfSlice<'a>;

    fn byte_slice(&'a self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        let sub_range = self.relative_byte_to_absolute_range(byte_range)?;

        Some(SfSlice {
            source: self.source.clone(),
            slice_char_range: sub_range,
            slice: None,
        })
    }

    fn byte_to_char_range(&self, byte_range: Range<usize>) -> Option<Range<usize>> {
        self.inner_slice().byte_to_char_range(byte_range)
    }

    fn char_to_byte_range(&self, char_range: Range<usize>) -> Option<Range<usize>> {
        self.inner_slice().char_to_byte_range(char_range)
    }
}

#[derive(Debug, Error)]
pub enum SourceFileError {
    #[error("failed to open source file at \"{path}\" because {error}")]
    OpeningSourceFile {
        #[source] error: io::Error,
        path: PathBuf,
    },
    #[error("the path \"{0}\" is not absolute")]
    NotAbsPath(PathBuf),
}

// no lints, because these errors are not constrained to a token
impl CompilerError for SourceFileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{self, absolute};

    #[test]
    fn reading_from_source_file() {
        let abs_path = path::absolute(PathBuf::from("./test-resources/fib.bfu")).unwrap();
        let sf = SourceFile::from_file(&abs_path).unwrap();

        let contents = fs::read_to_string(&abs_path).unwrap();
        assert!(sf.contents.contains(&contents));
    }

    #[test]
    fn non_abs_path_check() {
        let not_abs_path = PathBuf::from("./test-resources/fib.bfu");
        if let Err(SourceFileError::NotAbsPath(_)) = SourceFile::from_file(not_abs_path) {
            // YAY ! :3
        } else {
            // wtf
            panic!("non abs path got accepted for creating a sourcefile")
        }
    }

    #[test]
    fn fail_on_opening_unexisting_file() {
        let abs_path = path::absolute(
            PathBuf::from("./test-resources/the-nicole-files.bfu")
        ).unwrap();
        SourceFile::from_file(&abs_path)
            .expect_err("fish");
    }

    fn test_file() -> SourceFile {
        let path = path::absolute(PathBuf::from("./test-resources/fib.bfu")).unwrap();
        let sf = SourceFile::from_file(path)
            .unwrap();

        sf
    }

    #[test]
    fn sfslice_offset() {
        let sf = test_file();
        
        // 012.345.6789
        let sfs = sf.char_slice(3..6).unwrap();
        assert_eq!(sfs.offset(), 3);

        let sfs = sf.char_slice(84..124).unwrap();
        assert_eq!(sfs.offset(), 84);

        let sfs = sf.char_slice(0..29).unwrap();
        assert_eq!(sfs.offset(), 0);
    }

    #[test]
    fn sfslice_slice() {
        let sf = SourceFile::from_raw_parts(
            PathBuf::new(),
            "【 ▯▯▯▯】 ∴  ╔▌▯▯⇭ ▕▚".to_string(),
        );

        let sfs = sf.char_slice(0..2).unwrap();
        assert_eq!(sfs.inner_slice(), "【 ");

        let sfs = sf.char_slice(8..16).unwrap();
        assert_eq!(sfs.inner_slice(), "∴  ╔▌▯▯⇭");
        let sfs = sf.char_slice(15..19).unwrap();
        assert_eq!(sfs.inner_slice(), "⇭ ▕▚");
    }

    #[test]
    fn sfslice_byte_to_char_range() {
        let sf = SourceFile::from_raw_parts(
            PathBuf::new(),
            "【 ▯▯▯▯】 ∴  ╔▌▯▯⇭ ▕▚".to_string(),
        );

        let sfs = sf.char_slice(0..3).unwrap();
        assert_eq!(sfs.byte_to_char_range(3..7), Some(1..3));
        assert_eq!(sf.contents.get(3..7), Some(" ▯"));
    }

    #[test]
    fn sfslice_relative_char_to_absolute_range() {
        let sf = SourceFile::from_raw_parts(
            PathBuf::new(),
            "【 ▯▯▯▯】 ∴  ╔▌▯▯⇭ ▕▚".to_string(),
        );

        let sfs = sf.char_slice(2..8).unwrap();
        assert_eq!(sfs.relative_char_to_absolute_range(1..2), Some(3..4));
        assert_eq!(sfs.relative_char_to_absolute_range(0..4), Some(2..6));
        assert_eq!(sfs.relative_char_to_absolute_range(0..10), None);
    }

    #[test]
    fn sfslice_char_slice() {
        let sf = SourceFile::from_raw_parts(
            PathBuf::new(),
            "【 ▯▯▯▯】 ∴  ╔▌▯▯⇭ ▕▚".to_string(),
        );

        let sfs = sf.char_slice(0..7)
            .unwrap();
        assert_eq!(sfs.inner_slice(), "【 ▯▯▯▯】");
        let ssfs = sfs.char_slice(2..7)
            .unwrap();
        assert_eq!(ssfs.inner_slice(), "▯▯▯▯】");

        let sfs = sf.char_slice(8..19)
            .unwrap();
        assert_eq!(sfs.inner_slice(), "∴  ╔▌▯▯⇭ ▕▚");
        let ssfs = sfs.char_slice(0..11)
            .unwrap();
        assert_eq!(ssfs.inner_slice(), "∴  ╔▌▯▯⇭ ▕▚");

        let opt_sfs = sf.char_slice(0..732);
        assert!(opt_sfs.is_none());

        let sfs = sf.char_slice(0..10)
            .unwrap();
        assert!(sfs.char_slice(25..12).is_none());
    }
}

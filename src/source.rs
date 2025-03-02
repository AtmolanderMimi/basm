//! Utilities to handle the provenance of string back to it orignal file (aka its source).

use std::{fmt::{Debug, Display}, fs, io, ops::Range, path::{Path, PathBuf}};

use thiserror::Error;

use crate::error::CompilerError;
use crate::utils::Sliceable;

/// Represents a bfu source code file.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceFile {
    contents: String,
    absolute_path: PathBuf, // do we really need the path?
}

impl SourceFile {
    /// Creates a new [`SourceFile`] from its raw parts.
    /// `absolute_path` is expected to be in its absolute form.
    pub fn from_raw_parts(absolute_path: PathBuf, contents: String) -> SourceFile {
        SourceFile {
            contents,
            absolute_path,
        }
    }

    /// Creates a new [`SourceFile`] from a file.
    /// 
    /// # Errors
    /// Errors out if `absolute_path` is not absolute.
    pub fn from_file(absolute_path: impl AsRef<Path>) -> Result<SourceFile, SourceFileError> {
        let absolute_path = absolute_path.as_ref();
        if !absolute_path.is_absolute() {
            return Err(SourceFileError::NotAbsPath(absolute_path.to_path_buf()));
        }

        let contents = match fs::read_to_string(absolute_path) {
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

    /// Leaks the [`SourceFile`] into the heap, returning a static reference to it.
    pub fn leak(self) -> &'static SourceFile {
        let heap = Box::new(self);
        Box::leak(heap)
    }

    /// returns the lenght in bytes of the file.
    pub fn lenght(&self) -> usize {
        self.contents.len()
    }

    /// Returns the absolute path of the source file.
    pub fn absolute_path(&self) -> &Path {
        &self.absolute_path
    }
}

impl Sliceable for &'static SourceFile {
    type SliceType = SfSlice;

    fn slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        SfSlice::from_source(self, byte_range)
    }
}

impl Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.contents)
    }
}

impl AsRef<str> for SourceFile {
    fn as_ref(&self) -> &str {
        &self.contents
    }
}

/// A slice of a [`SourceFile`]. It contains information about its position.
#[derive(Clone, PartialEq)]
pub struct SfSlice {
    source: &'static SourceFile,
    slice_byte_range: Range<usize>,
}

impl SfSlice {
    /// Creates a [`SfSlice`] from a [`SourceFile`].
    /// The `range` is by bytes, not characters.
    /// Returns `None` if the `byte_range` is not valid.
    pub fn from_source(source: &'static SourceFile, byte_range: Range<usize>) -> Option<SfSlice> {
        // checks if the byte_range is valid
        if byte_range.start > byte_range.end {
            return None;
        }

        let valid_bytes = 0..=(source.contents.len());
        if !valid_bytes.contains(&byte_range.start) {
            return None;
        } else if !valid_bytes.contains(&byte_range.end) {
            return None;
        }

        Some(SfSlice {
            source,
            slice_byte_range: byte_range,
        })
    }

    #[cfg(test)]
    /// Creates a new slice, just for testing purposes.
    pub fn new_bogus(contents: &str) -> SfSlice {
        let sf = SourceFile::from_raw_parts(PathBuf::new(), contents.to_string());
        let sf = sf.leak();
        let slice = sf.slice(0..(contents.len())).unwrap();

        slice
    }

    /// Returns the offset from the start of the source file this slice is referencing
    /// in bytes.
    pub fn offset(&self) -> usize {
        self.slice_byte_range.start
    }

    /// Returns the range of characters into the source of
    /// this slice.
    /// This start char range is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn range(&self) -> Range<usize> {
        self.slice_byte_range.clone()
    }

    /// Returns the start index of the range of bytes 
    /// into the source of this slice.
    /// This start position is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn start(&self) -> usize {
        self.slice_byte_range.start
    }

    /// Returns the end index of the range of bytes 
    /// into the source of this slice.
    /// This end position is absolute from the original source file,
    /// not from it's originating subslice.
    pub fn end(&self) -> usize {
        self.slice_byte_range.end
    }

    /// Returns the equivalent string slice.
    pub fn inner_slice(&self) -> &str {
        (&self.source.contents).slice(self.range())
            .expect("char_range should always be a valid substring of the source")
    }

    /// Returns the [`SourceFile`] from which this [`SfSlice`] was referenced.
    pub fn source(&self) -> &'static SourceFile {
        self.source
    }

    /// Transforms a relative range into this slice into an absolute range of the source file.
    /// `rel_range` is in **bytes**.
    /// The return value is in bytes.
    /// Returns `None` if  `rel_range` is not within the range of this slice.
    pub fn relative_to_absolute_range(&self, rel_range: Range<usize>) -> Option<Range<usize>> {
        let abs_end = rel_range.end + self.offset();
        if abs_end > self.slice_byte_range.end {
            return None;
        }

        Some((rel_range.start+self.offset())..(rel_range.end+self.offset()))
    }
}

impl Sliceable for SfSlice {
    type SliceType = SfSlice;

    fn slice(&self, byte_range: Range<usize>) -> Option<Self::SliceType> {
        let abs_range = self.relative_to_absolute_range(byte_range)?;
        SfSlice::from_source(self.source, abs_range)
    }
}

impl AsRef<str> for SfSlice {
    fn as_ref(&self) -> &str {
        self.inner_slice()
    }
}

impl Debug for SfSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SfSlice")
            .field("source", &self.source.absolute_path)
            .finish()
    }
}

impl Display for SfSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner_slice())
    }
}

/// An error that occurs while working with [`SourceFile`] and other related types.
#[derive(Debug, Error)]
pub enum SourceFileError {
    /// Failed to open the path to the source file.
    #[error("failed to open source file at \"{path}\" because {error}")]
    OpeningSourceFile {
        /// The inner error that caused the fail.
        #[source] error: io::Error,
        /// The path that it tried to open.
        path: PathBuf,
    },
    /// The provided path was not absolute, even if it was expected.
    /// `0` is the non absolute path.
    #[error("the path \"{0}\" is not absolute")]
    NotAbsPath(PathBuf),
}

// no lints, because these errors are not constrained to a token
impl CompilerError for SourceFileError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path;

    #[test]
    fn reading_from_source_file() {
        let abs_path = path::absolute(PathBuf::from("./test-resources/fib.basm")).unwrap();
        let sf = SourceFile::from_file(&abs_path).unwrap();

        let contents = fs::read_to_string(&abs_path).unwrap();
        assert!(sf.contents.contains(&contents));
    }

    #[test]
    fn non_abs_path_check() {
        let not_abs_path = PathBuf::from("./test-resources/fib.basm");
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
}

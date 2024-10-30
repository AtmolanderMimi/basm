use std::{error::Error, ops::Range};

use either::Either;

use crate::{source::{SfSlice, SourceFile}, utils::CharOps as _};

pub trait CompilerError: Error {
    /// Returns the range of the error.
    fn lint(&self) -> Option<Lint> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A lint in the source code
pub struct Lint<'a> {
    gravity: LintGravity,
    /// range in the source code file, or the whole file
    slice: Either<SfSlice<'a>, &'a SourceFile>
}

impl<'a> Lint<'a> {
    /// Creates a new [Lint] with the gravity of error.
    pub fn new_error(source: &SourceFile) -> Lint {
        Lint {
            gravity: LintGravity::Error,
            slice: Either::Right(source)
        }
    }

    /// Creates a new [Lint] with the gravity of error as the slice of the file.
    /// `range` is in characters, not bytes.
    pub fn new_error_range(source: &SourceFile, range: Range<usize>) -> Option<Lint> {
        let l = Lint {
            gravity: LintGravity::Error,
            slice: Either::Left(source.char_slice(range)?)
        };

        Some(l)
    }

    /// Creates a new [Lint] with the gravity of warning.
    pub fn new_warning(source: &SourceFile) -> Lint {
        Lint {
            gravity: LintGravity::Warning,
            slice: Either::Right(source)
        }
    }

    /// Creates a new [Lint] with the gravity of warning as the slice of the file.
    /// `range` is in characters, not bytes.
    pub fn new_warning_range(source: &SourceFile, range: Range<usize>) -> Option<Lint> {
        let l = Lint {
            gravity: LintGravity::Warning,
            slice: Either::Left(source.char_slice(range)?)
        };

        Some(l)
    }

    /// Creates a new [Lint] from a [SfSlice] with the gravity of error.
    pub fn from_slice_error(slice: SfSlice) -> Lint {
        Lint {
            slice: Either::Left(slice),
            gravity: LintGravity::Error,
        }
    }

    /// Creates a new [Lint] from a [SfSlice] with the gravity of warning.
    pub fn from_slice_warning(slice: SfSlice) -> Lint {
        Lint {
            slice: Either::Left(slice),
            gravity: LintGravity::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LintGravity {
    #[default]
    Error,
    Warning,
}
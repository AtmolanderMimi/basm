//! Error and linting utilities.

use std::{error::Error, ops::Range};

use colored::Colorize;
use either::Either;

use crate::{source::{SfSlice, SourceFile}, utils::{CharOps as _, FindLnCol}};

/// Number of characters around a lint for context in error display.
const CONTEXT_WINDOW: usize = 50;

/// Trait to add to all the errors within this crate.
/// Ensures and allows easy printing of errors with error the source and extra info.
pub trait CompilerError: Error {
    /// Returns the range of the error.
    fn lint(&self) -> Option<Lint> {
        None
    }

    /// Returns a fancy print-ready description
    /// of the error.
    fn description(&self) -> String {
        let mut out = String::new();

        let lint = self.lint();

        let gravity = lint.map_or(LintGravity::Error, |l| l.gravity);

        match gravity {
            LintGravity::Error => out.push_str(&"Error:".color(gravity.associated_color()).bold().to_string()),
            LintGravity::Warning => out.push_str(&"Warning:".color(gravity.associated_color()).bold().to_string()),
        }

        if let Some(l) = self.lint() {
            // -- position --
            match l.slice {
                Either::Left(ref slice) => {
                    let start = slice.char_range().start;
                    let (ln, col) = slice.source().char_find_ln_col(start).unwrap();
                    let abs_path = slice.source().absolute_path();

                    out.push_str(&format!(" from Ln {ln:?}, Col {col:?} in {abs_path:?}\n"));
                },
                Either::Right(sf) => {
                    let abs_path = sf.absolute_path();
                    
                    out.push_str(&format!(" in {abs_path:?}\n"));
                }
            }

            // -- err message --
            out.push_str(&format!(" → {}\n", &self.to_string().underline().bold()));

            // -- context --
            if let Either::Left(slice) = l.slice {
                let source = slice.source();
                let pre_context_range = slice.start_char().saturating_sub(CONTEXT_WINDOW)..slice.start_char();
                let post_context_range = if (slice.end_char() + CONTEXT_WINDOW) > source.char_lenght() {
                    slice.end_char()..source.char_lenght()
                } else {
                    slice.end_char()..(slice.end_char() + CONTEXT_WINDOW)
                };

                out.push_str(&"[...] ".black().to_string());
                out.push_str(source.slice_char(pre_context_range).unwrap().as_ref());

                out.push_str(&slice.as_ref().color(gravity.associated_color()).underline().bold().to_string());

                out.push_str(source.slice_char(post_context_range).unwrap().as_ref());
                out.push_str(&" [...]".black().to_string());
            }
        } else {
            out.push_str(&format!(" {}", self.to_string()).underline().to_string());
        }
        
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
/// A lint in the source code
pub struct Lint {
    gravity: LintGravity,
    /// range in the source code file, or the whole file
    slice: Either<SfSlice, &'static SourceFile>
}

impl Lint {
    /// Creates a new [`Lint`] with the gravity of error.
    pub fn new_error(source: &'static SourceFile) -> Lint {
        Lint {
            gravity: LintGravity::Error,
            slice: Either::Right(source)
        }
    }

    /// Creates a new [`Lint`] with the gravity of error as the slice of the file.
    /// `range` is in characters, not bytes.
    pub fn new_error_range(source: &'static SourceFile, range: Range<usize>) -> Option<Lint> {
        let l = Lint {
            gravity: LintGravity::Error,
            slice: Either::Left(source.slice_char(range)?)
        };

        Some(l)
    }

    /// Creates a new [`Lint`] with the gravity of warning.
    pub fn new_warning(source: &'static SourceFile) -> Lint {
        Lint {
            gravity: LintGravity::Warning,
            slice: Either::Right(source)
        }
    }

    /// Creates a new [`Lint`] with the gravity of warning as the slice of the file.
    /// `range` is in characters, not bytes.
    pub fn new_warning_range(source: &'static SourceFile, range: Range<usize>) -> Option<Lint> {
        let l = Lint {
            gravity: LintGravity::Warning,
            slice: Either::Left(source.slice_char(range)?)
        };

        Some(l)
    }

    /// Creates a new [`Lint`] from a [`SfSlice`] with the gravity of error.
    pub fn from_slice_error(slice: SfSlice) -> Lint {
        Lint {
            slice: Either::Left(slice),
            gravity: LintGravity::Error,
        }
    }

    /// Creates a new [`Lint`] from a [`SfSlice`] with the gravity of warning.
    pub fn from_slice_warning(slice: SfSlice) -> Lint {
        Lint {
            slice: Either::Left(slice),
            gravity: LintGravity::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
/// The gravity of a lint
enum LintGravity {
    #[default]
    Error,
    Warning,
}

impl LintGravity {
    pub fn associated_color(&self) -> colored::Color {
        match self {
            LintGravity::Error => colored::Color::Red,
            LintGravity::Warning => colored::Color::Yellow,
        }
    }
}
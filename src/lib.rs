//! # Brain Aneurysm
//! 
//! started as of 2024-10-12

#![feature(assert_matches)]

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::must_use_candidate,
    // messes up with colored, not all `to_string` are directly going to be displayed yakno
    clippy::to_string_in_format_args,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
)]

/// The number type, aka what does each cell on the tape hold
pub type Num = u8;

pub mod error;
pub use error::{CompilerError, Lint};
pub mod lexer;
pub use lexer::lex_file;
pub mod source;
pub mod utils;

/// Transpiles bfu source code into bf.
pub fn transpile(source: &str) -> &str {
    todo!()
}
#![feature(assert_matches)]

/// The number type, aka what does each cell on the tape hold
pub (self) type Num = u8;

pub mod error;
pub use error::{CompilerError, Lint};
pub mod lexer;
pub mod source;
pub mod utils;

/// Transpiles bfu source code into bf.
pub fn transpile(source: &str) -> &str {
    todo!()
}
#![feature(assert_matches)]

/// The number type, aka what does each cell on the tape hold
pub (self) type Num = u8;

mod error;
use error::{CompilerError, Lint};
mod lexer;
mod source;
mod utils;

/// Transpiles bfu source code into bf.
pub fn transpile(source: &str) -> &str {
    todo!()
}
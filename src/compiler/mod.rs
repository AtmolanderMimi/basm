//! The compiler part of the language,
//! takes a parsed file.

mod value;
mod scope;
mod expression;

use thiserror::Error;

use crate::parser::ParsedFile;
use crate::CompilerError as CompilerErrorTrait;

/// Imports an item from [crate::parser] with the `Parsed` prefix.
#[macro_export]
macro_rules! use_as_parsed {
    ($item:ident) => {
        use crate::parser::$item as ${ concat(Parsed, $item) };
    };
}

/// An error which occured during compilation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompilerError {

}

impl CompilerErrorTrait for CompilerError {

}

/// Compiles the file (and any other files that the given file may import).
pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
    todo!()
}


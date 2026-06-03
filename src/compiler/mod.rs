//! The compiler part of the language,
//! takes a parsed file.

mod value;
mod scope;
mod operators;
mod string_normalizer;
mod expression;
mod emplacement;
mod argument;
mod directive;
mod file;

use thiserror::Error;

use crate::compiler::directive::{Directive, DirectiveError};
use crate::compiler::emplacement::EmplacementNormalizationError;
use crate::compiler::expression::{Expression, ExpressionEvaluationError};
use crate::compiler::file::File;
use crate::compiler::scope::Scope;
use crate::parser::{LanguageItem, ParsedFile};
use crate::{CompilerError as CompilerErrorTrait, Lint};

// Creates a new #[repr(transparent)] newtype wrapper of the type with the specified name.
/// The newtype implements `From` for casting from the original type and 
#[macro_export]
macro_rules! newtype_wrapper {
    ($newtype:ident, $inner_type:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq)]
        pub struct $newtype(pub $inner_type);

        // -- direct value
        // value cast
        impl From<$inner_type> for $newtype {
            fn from(value: $inner_type) -> Self {
                unsafe { std::mem::transmute::<$inner_type, $newtype>(value) }
            }
        }

        // ref cast
        impl<'a> From<&'a $inner_type> for &'a $newtype {
            fn from(value: &'a $inner_type) -> Self {
                unsafe { std::mem::transmute::<&$inner_type, &$newtype>(value) }
            }
        }

        // mut ref cast
        impl<'a> From<&'a mut $inner_type> for &'a mut $newtype {
            fn from(value: &'a mut $inner_type) -> Self {
                unsafe { std::mem::transmute::<&mut $inner_type, &mut $newtype>(value) }
            }
        }
    };
}

/// An error which occured during compilation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompilerError {
    #[error("{inner}")]
    DirectiveError {
        directive: Directive,
        inner: DirectiveError,
    },
}

impl CompilerErrorTrait for CompilerError {
    fn lint(&self) -> Option<crate::Lint> {
        let lint = match self {
            Self::DirectiveError { directive, .. } => Lint::from_slice_error(directive.0.slice())
        };

        Some(lint)
    }
}

/// Compiles the file (and any other files that the given file may import).
pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
    let file: &File = program.into();

    let mut main_scope = Scope::new();
    file.interpret_directives(&mut main_scope)?;

    Ok(main_scope.get_output().to_string())
}

#[cfg(test)]
mod tests {
    use crate::parser::Pattern;

use super::*;

    #[test]
    fn minimal_program() {
        let file = ParsedFile::solve_str(include_str!("../../test-resources/small.basm"))
            .unwrap();

        let output = compile(&file).unwrap();
        assert_eq!(output, "hello, world! :D");
    }
}


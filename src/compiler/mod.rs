//! The compiler part of the language,
//! takes a parsed file.

mod value;
mod scope;
mod operators;
mod string_normalizer;
mod expression;
mod directive;

use either::Either;
use thiserror::Error;

use crate::compiler::directive::{Directive, DirectiveInlineError};
use crate::compiler::expression::Expression;
use crate::compiler::operators::OperationError;
use crate::parser::{CharLit, Ident, LanguageItem, ParsedFile, StrLit};
use crate::{CompilerError as CompilerErrorTrait, Lint};

/// Imports an item from [crate::parser] with the `Parsed` prefix
/// AND creates a new #[repr(transparent)] newtype wrapper of it under the original name.
/// The newtype implements `From` for casting from the original type and 
#[macro_export]
macro_rules! use_as_parsed {
    ($item:ident) => {
        use crate::parser::$item as ${ concat(Parsed, $item) };
        
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq)]
        pub struct $item(pub ${ concat(Parsed, $item) });

        // value cast
        impl From<${ concat(Parsed, $item) }> for $item {
            fn from(value: ${ concat(Parsed, $item) }) -> Self {
                unsafe { std::mem::transmute::<${ concat(Parsed, $item) }, $item>(value) }
            }
        }

        // ref cast
        impl<'a> From<&'a ${ concat(Parsed, $item) }> for &'a $item {
            fn from(value: &'a ${ concat(Parsed, $item) }) -> Self {
                unsafe { std::mem::transmute::<&${ concat(Parsed, $item) }, &$item>(value) }
            }
        }

        // mut ref cast
        impl<'a> From<&'a mut ${ concat(Parsed, $item) }> for &'a mut $item {
            fn from(value: &'a mut ${ concat(Parsed, $item) }) -> Self {
                unsafe { std::mem::transmute::<&mut ${ concat(Parsed, $item) }, &mut $item>(value) }
            }
        }
    };
}

/// An error which occured during compilation
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CompilerError {
    #[error("{inner}")]
    OperationError {
        inner: OperationError,
        expression: Expression,
    },
    #[error("variable {} does not exist in the current scope", ident.slice_str())]
    VariableDoesNotExist {
        ident: Ident
    },
    #[error("escape sequence {sequence} is invalid")]
    EscapeSequencesIsInvalid {
        lit: Either<StrLit, CharLit>,
        sequence: String,
    },
    #[error("{inner}")]
    DirectiveInlineError {
        directive: Directive,
        inner: DirectiveInlineError,
    },
}

impl CompilerErrorTrait for CompilerError {
    fn lint(&self) -> Option<crate::Lint> {
        let lint = match self {
            Self::VariableDoesNotExist { ident } => Lint::from_slice_error(ident.slice()),
            Self::EscapeSequencesIsInvalid { lit, .. } => {
                let slice = lit.as_ref().either(|s| s.slice(), |c| c.slice());
                Lint::from_slice_error(slice)
            },
            Self::OperationError { expression, .. } => {
                Lint::from_slice_error(expression.0.slice())
            },
            Self::DirectiveInlineError { directive, .. } => Lint::from_slice_error(directive.0.slice())
        };

        Some(lint)
    }
}

/// Compiles the file (and any other files that the given file may import).
pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
    todo!()
}


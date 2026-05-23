//! Tools used to parse a string of tokens into sensible a sensible structure (the [`ParsedProgram`] struct).

// because i want to share patterns, but users shouldn't have to use them directly
// so it is fine they can't because of private bounds.
#![allow(private_interfaces)]

mod components;
mod terminals;
mod operators;
mod list;
mod directive;
mod r#macro;
mod expression;
mod file;

use thiserror::Error;

use crate::{CompilerError, Lint, lexer::token::Token, parser::{expression::ExpressionItem, file::ParsedFile}, source::SfSlice};

/// Return type of trying to solve for a pattern.
/// The `Ok` variant contains the number of tokens taken to solve the pattern (the `usize`)
pub type PatternResult<T> = Result<(usize, T), ParseError>;

/// Defines a language pattern.
pub trait Pattern where Self: Sized {
    /// The type resulting from the parse of the pattern.
    type ParseResult: Sized = Self;

    /// Solves a pattern. See [PatternResult] for how to interpret result.
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult>;

    /// The name of the pattern, used for errors
    fn name() -> String;
}

/// Error happening during the parsing process when an unexpected token is encontered.
/// Language item parsers will throw this error
/// when encountering a pattern in the tokens that doesn't match their expectation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ParseError {
    /// When a pattern encounters a token which was not expected.
    #[error("expected {expected}, got {}", got.t_type.to_string())]
    UnexpectedTokenError {
        /// The expected pattern
        expected: String,
        /// The token that was had
        got: Token,
    },
    /// When an expression cannot be constructed.
    #[error("expression could not be properly parsed (probably because operators can't be linked to operands)")]
    UnparsedExpression {
        /// The items in the expression that could not be parsed
        items: Vec<ExpressionItem>
    },
    /// When the pattern has no more tokens to read, this should never happen.
    #[error("{0} could not be parsed before running out of tokens")]
    NoMoreTokens(String),
}

impl ParseError {
    /// Creates a new [Self::UnexpectedTokenError].
    pub fn new_unexpected_token(expected: impl Into<String>, got: Token) -> Self {
        Self::UnexpectedTokenError {
            expected: expected.into(),
            got,
        }
    }

    /// Creates a new [Self::UnparsedExpression].
    pub fn new_unparsed_expression(items: Vec<ExpressionItem>) -> Self {
        Self::UnparsedExpression { items }
    }

    /// Creates a new [Self::NoMoreTokens].
    pub fn new_no_more_tokens(pattern_name: String) -> Self {
        Self::NoMoreTokens(pattern_name)
    }
}

impl CompilerError for ParseError {
    fn lint(&self) -> Option<crate::Lint> {
        let lint = match self {
            Self::UnexpectedTokenError { got, .. } => Lint::from_slice_error(got.slice()),
            Self::UnparsedExpression { items } => {
                let start = items.first()?.slice().start();
                let end = items.last()?.slice().end();

                Lint::new_error_range(items.first()?.slice().source(), start..end)?
            },
            Self::NoMoreTokens(_) => return None,
        };

        Some(lint)
    }
}

/// A generic trait to be implemented onto each language item.
pub trait LanguageItem {
    /// A slice defining the position of the language item.
    fn slice(&self) -> SfSlice;
}

impl LanguageItem for Token {
    fn slice(&self) -> SfSlice {
        self.slice.clone()
    }
}

#[macro_export]
/// Used to implement the [`LanguageItem`] trait's `slice()` method in a generic method.
/// Creates an implementation which creates a slice from the start of `start` to the end of `end`. 
macro_rules! impl_language_item {
    ($type:ty, $start:ident, $end:ident) => {
        impl crate::parser::LanguageItem for $type {
            fn slice(&self) -> crate::source::SfSlice {
                let start_slice = self.$start.slice();
                let start = start_slice.start();
                let end = self.$end.slice().end();
                
                let source = start_slice.source();
                crate::utils::Sliceable::slice(&source, start..end)
                    .unwrap()
            }
        }
    };
}

/// Parses the tokens into a structured form ([`ParsedFile`]).
pub fn parse_tokens(tokens: &[Token]) -> Result<ParsedFile, ParseError> {
    ParsedFile::solve(&tokens)
        .map(|(_, file)| file)
}

/// The collection of patterns used to parse for structures.
/// (At least all the patterns for the structures which are public)
pub mod pattern {
    pub use super::{Pattern};

    use crate::parser::components;

    pub use components::*;
}

//! Tools used to parse a string of tokens into sensible a sensible structure (the [`ParsedProgram`] struct).

// because i want to share patterns, but users shouldn't have to use them directly
// so it is fine they can't because of private bounds.
#![allow(private_interfaces)]

mod components;
mod terminals;
pub use terminals::*;
mod operators;
pub use operators::*;
mod list;
pub use list::*;
mod directive;
pub use directive::*;
mod r#macro;
pub use r#macro::*;
mod expression;
pub use expression::*;
mod emplacement;
pub use emplacement::*;
mod file;
pub use file::*;

use thiserror::Error;

use crate::{CompilerError, Lint, lexer::token::Token, source::SfSlice};

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

    /// Lexes and then parses the string.
    /// 
    /// # Panics
    /// Panics if the lexing fails.
    #[cfg(test)]
    fn solve_str(string: &str) -> Result<Self::ParseResult, ParseError> {
        Self::solve(&crate::lexer::lex_string(string).unwrap())
            .map(|ok| ok.1)
    }
}

/// Error happening during the parsing process when an unexpected token is encontered.
/// Language item parsers will throw this error
/// when encountering a pattern in the tokens that doesn't match their expectation.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("{variant}")]
pub struct ParseError {
    tokens_before_error: usize,
    variant: ParseErrorVariant
}

/// A variant of [ParseError].
#[derive(Debug, Clone, PartialEq, Error)]
enum ParseErrorVariant {
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
    /// When an emplacement is invalid.
    #[error("emplacement expression does not represent an emplacement")]
    InvalidEmplacement(EmplacementSubExpression),
    /// When the pattern has no more tokens to read, this should never happen.
    #[error("{0} could not be parsed before running out of tokens")]
    NoMoreTokens(String),
    /// When a pattern is matched (through [Not]) when it should not.
    #[error("{0} was expected to be absent")]
    UnexpectedPattern(String, SfSlice),
}

impl ParseError {
    /// Creates a new [Self::UnexpectedTokenError].
    pub fn new_unexpected_token(tokens_before_error: usize, expected: impl Into<String>, got: Token) -> Self {
        ParseError {
            tokens_before_error,
            variant: ParseErrorVariant::UnexpectedTokenError {
                expected: expected.into(),
                got,
            },
        }   
    }

    /// Creates a new [Self::UnparsedExpression].
    pub fn new_unparsed_expression(tokens_before_error: usize, items: Vec<ExpressionItem>) -> Self {
        ParseError {
            tokens_before_error,
            variant: ParseErrorVariant::UnparsedExpression { items },
        }
    }

    /// Creates a new [Self::NoMoreTokens].
    pub fn new_no_more_tokens(tokens_before_error: usize, pattern_name: String) -> Self {
        ParseError {
            tokens_before_error,
            variant: ParseErrorVariant::NoMoreTokens(pattern_name),
        }
    }

    /// Creates a new [Self::UnexpectedPattern].
    pub fn new_unexpected_pattern(tokens_before_error: usize, pattern_name: String, pattern_slice: SfSlice) -> Self {
        ParseError {
            tokens_before_error,
            variant: ParseErrorVariant::UnexpectedPattern(pattern_name, pattern_slice),
        }
    }

    /// The number of tokens which had to be consumed (or tried to be consumed), before the parse was concluded as an error.
    pub fn tokens_before_error(&self) -> usize {
        self.tokens_before_error
    }

    /// Adds `nb_tokens` to the total of tokens consumed before the error occured.
    pub fn add_tokens_before_error(&mut self, nb_tokens: usize) {
        self.tokens_before_error += nb_tokens;
    }
}

impl CompilerError for ParseError {
    fn lint(&self) -> Option<crate::Lint> {
        let lint = match &self.variant {
            ParseErrorVariant::UnexpectedTokenError { got, .. } => Lint::from_slice_error(got.slice()),
            ParseErrorVariant::UnparsedExpression { items } => {
                let start = items.first()?.slice().start();
                let end = items.last()?.slice().end();

                Lint::new_error_range(items.first()?.slice().source(), start..end)?
            },
            ParseErrorVariant::InvalidEmplacement(e) => Lint::from_slice_error(e.slice()),
            ParseErrorVariant::NoMoreTokens(_) => return None,
            ParseErrorVariant::UnexpectedPattern(_, slice) => Lint::from_slice_error(slice.clone()),
        };

        Some(lint)
    }
}

/// A generic trait to be implemented onto each language item.
pub trait LanguageItem {
    /// A slice defining the position of the language item.
    fn slice(&self) -> SfSlice;

    /// A string slice, generated from `slice`
    fn slice_str(&self) -> &str {
        let slice = self.slice();
        slice.inner_slice()
    }
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

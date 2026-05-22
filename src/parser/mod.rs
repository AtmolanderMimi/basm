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

use std::fmt::Display;

use thiserror::Error;

use crate::{CompilerError, Lint, lexer::token::{Token}, parser::file::ParsedFile, source::SfSlice};

/// Return type of trying to solve for a pattern.
/// The `Ok` variant contains the number of tokens taken to solve the pattern (the `usize`)
pub type PatternResult<T> = Result<(usize, T), UnexpectedTokenError>;

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
pub struct UnexpectedTokenError {
    expected: String,
    got: Option<Token>,
}

impl UnexpectedTokenError {
    /// Creates a new [UnexpectedTokenError].
    pub fn new(expected: impl Into<String>, got: Token) -> Self {
        UnexpectedTokenError {
            expected: expected.into(),
            got: Some(got),
        }
    }

    // TODO: replace this error, as the error is quite cryptic when there is no "unexpected token"
    /// Creates a new [UnexpectedTokenError] without a gotten token.
    pub fn new_got_nothing(expected: impl Into<String>) -> Self {
        UnexpectedTokenError {
            expected: expected.into(),
            got: None,
        }
    }
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let got_string = self.got.as_ref()
            .map(|t| t.t_type.to_string())
            .unwrap_or("nothing".to_string());

        f.write_str(&format!("expected {}, got {got_string}", self.expected))
    }
}

impl CompilerError for UnexpectedTokenError {
    fn lint(&self) -> Option<crate::Lint> {
        self.got.as_ref().map(|t| Lint::from_slice_error(t.slice()))
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
pub fn parse_tokens(tokens: &[Token]) -> Result<ParsedFile, UnexpectedTokenError> {
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

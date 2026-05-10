//! Tools used to parse a string of tokens into sensible a sensible structure (the [`ParsedProgram`] struct).

// because i want to share patterns, but users shouldn't have to use them directly
// so it is fine they can't because of private bounds.
#![allow(private_interfaces)]

mod components;
mod terminals;

use std::fmt::Display;

use thiserror::Error;

use crate::{CompilerError, Lint, lexer::token::{Token, TokenType}, source::SfSlice};

// Return type of trying to solve for a pattern.
// The `Ok` variant contains the number of tokens taken to solve the pattern (the `usize`)
pub type PatternResult<T: Clone> = Result<(usize, T), UnexpectedTokenError>;

/// Defines a language pattern.
pub trait Pattern {
    type ParseResult: Clone;

    /// Solves a pattern. See [PatternResult] for how to interpret result.
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult>;
}

/// Error happening during the parsing process when an unexpected token is encontered.
/// Language item parsers will throw this error
/// when encountering a pattern in the tokens that doesn't match their expectation.
#[derive(Debug, Clone, PartialEq, Error)]
pub struct UnexpectedTokenError {
    expected_tokens: Vec<TokenType>,
    got: Option<Token>,
}

impl UnexpectedTokenError {
    pub fn new(expected: Vec<TokenType>, got: Token) -> Self {
        UnexpectedTokenError {
            expected_tokens: expected,
            got: Some(got),
        }
    }

    pub fn new_got_nothing(expected: Vec<TokenType>) -> Self {
        UnexpectedTokenError {
            expected_tokens: expected,
            got: None,
        }
    }
}

impl Display for UnexpectedTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // gets the first element or "..."
        let mut expected_list = self.expected_tokens.get(0)
            .map(|t| t.to_string())
            .unwrap_or("...".to_string());

        // creates the format list in coma seperated format
        for token_type in &self.expected_tokens[1..] {
            expected_list.push_str(", ");
            expected_list.push_str(&token_type.to_string());
        }

        let got_string = self.got.as_ref()
            .map(|t| t.t_type.to_string())
            .unwrap_or("nothing".to_string());

        f.write_str(&format!("expected {expected_list}, got {got_string}"))
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
        impl LanguageItem for $type {
            fn slice(&self) -> SfSlice {
                let start_slice = self.$start.slice();
                let start = start_slice.start();
                let end = self.$end.slice().end();
                
                start_slice.source().slice(start..end)
                .unwrap()
            }
        }
    };
}

// TODO:
/// Parses the tokens into a structured form ([`ParsedProgram`]).
//pub fn parse_tokens(tokens: &[Token]) -> Result<ParsedFile, PatternMatchingError> {
//    solve_pattern::<FilePattern>(tokens)
//}

/// The collection of patterns used to parse for structures.
/// (At least all the patterns for the structures which are public)
pub mod patterns {
    pub use super::{Pattern};

    use crate::parser::terminals;
    use crate::parser::components;

    pub use terminals::{AtPattern, EofPattern, StarPattern, IdentPattern, MinusPattern, NumLitPattern, StrLitPattern, CharLitPattern, SemicolonPattern, LeftSquarePattern, RightSquarePattern};
    pub use components::{Or, Then, Many};
}

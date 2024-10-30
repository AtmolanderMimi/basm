pub mod token;
use thiserror::Error;

use crate::{error::{CompilerError, Lint}, source::SfSlice};

struct Lexer<'a> {
    index: usize,
    source: &'a str,
    /// the token currently being built
    building_token: &'a str,

}

impl<'a> Lexer<'a> {
    pub fn new(source: &str) -> Lexer {
        Lexer {
            index: 0,
            source: source,
            building_token: "",
        }
    }

    //pub fn advance(&mut self) {
    //    let next = if let Some(n) = self.source.chars().nth(self.index) {
    //        n
    //    } else {
    //        return
    //    };
//
    //    if next {}
    //}

}

#[derive(Error, Debug, PartialEq)]
pub enum LexerError<'a> {
    #[error("literal contents are invalid {0}")]
    InvalidLiteral(LiteralError<'a>),
}

#[derive(Error, Debug, PartialEq)]
pub enum LiteralError<'a> {
    #[error("Num \"{1:?}\" is not within allowed range (tip: most likely 0-255)")]
    InvalidNumber(SfSlice<'a>, String),
    #[error("Char literals cannot be empty")]
    EmptyChar(SfSlice<'a>),
    #[error("Char '{1:?}' is invalid. Char literals can only hold one character (maybe you want a string: \"...\"?")]
    TooFullChar(SfSlice<'a>, String),
}

impl<'a> CompilerError for LiteralError<'a> {
    fn lint(&self) -> Option<Lint> {
        let slice = match self {
            Self::EmptyChar(s, ..) => s,
            Self::InvalidNumber(s, ..) => s,
            Self::TooFullChar(s, ..) => s,
        };

        Some(Lint::from_slice_error(slice.clone()))
    }
}
//! Defines tooling used to parse a string into a chain of language tokens.
//! As a user, you should probably be looking for [`lex_file`], all of this module's
//! code is put to use in there.

pub mod token;
use std::ops::Range;

use thiserror::Error;
use token::Token;

use crate::{error::{CompilerError, Lint}, source::{SfSlice, SourceFile}, utils::CharOps};

struct Lexer<'a> {
    range: Range<usize>,
    source: &'a SourceFile,
    tokens: Vec<Token<'a>>
}

impl<'a> Lexer<'a> {
    pub fn new(source_file: &SourceFile) -> Lexer {
        Lexer {
            range: 0..0,
            source: source_file,
            tokens: Vec::new(),
        }
    }

    pub fn advance(&mut self) -> Result<Advancement, LexerError<'a>> {
        self.range.end += 1;

        if self.range.end > self.source.char_lenght() {
            return Ok(Advancement::Finished);
        }

        let sf_slice = self.source.char_slice(self.range.clone())
            .unwrap();
        if let Some(non_lit) = Token::parse_token_non_lit(&sf_slice) {
            let possibly_lit_range = self.range.start..non_lit.slice.start();
            let possibly_lit_slice = self.source.char_slice(possibly_lit_range)
                .unwrap();

            self.range.start = non_lit.slice.end();

            if let Some(lit) = Token::parse_token_lit(&possibly_lit_slice)? {
                self.tokens.push(lit);
            }
            self.tokens.push(non_lit);
        }

        Ok(Advancement::Advancing)
    }
}

/// Turns a [`SourceFile`] into a list of syntactic tokens, by "lexing" them.
/// 
/// # Errors
/// Errors are not fatal and can be accumulated over the lexing process.
/// This is why the return type on this function is `(Vec<Token>, <Vec<LexerError>)`.
/// Although errors are not fatal, they should be investigated since an error means
/// that a token could not be correctly formed and as thus the token list is partially invalid.
pub fn lex_file(source_file: &SourceFile) -> (Vec<Token>, Vec<LexerError>) {
    let mut errors = Vec::new();
    let mut lexer = Lexer::new(source_file);
    loop {
        match lexer.advance() {
            Ok(Advancement::Finished) => break,
            Ok(Advancement::Advancing) => (),
            Err(e) => errors.push(e),
        }
    }

    (lexer.tokens, errors)
}

enum Advancement {
    Advancing,
    Finished,
}

/// An error that occured during the lexing process.
#[derive(Error, Debug)]
pub enum LexerError<'a> {
    /// There was an error while forming a literal.
    #[error("{0}")]
    InvalidLiteral(LiteralError<'a>),
}

impl CompilerError for LexerError<'_> {
    fn lint(&self) -> Option<Lint> {
        match self {
            LexerError::InvalidLiteral(e) => e.lint()
        }
    }
}

impl<'a> From<LiteralError<'a>> for LexerError<'a> {
    fn from(value: LiteralError<'a>) -> Self {
        LexerError::InvalidLiteral(value)
    }
}

const ITALIC_START: &str = "\x1B[3m";
const ITALIC_END: &str = "\x1B[23m";

#[derive(Error, Debug, PartialEq)]
/// An error encounted while trying to parse for a literal syntax token.à
/// 
/// *`"Yo... This is literally an error, man..."`*
pub enum LiteralError<'a> {
    /// Number is invalid, because it is not within the range that bf can store (0-255).
    #[error("Num {0} is not within allowed range (tip: most likely 0-255)")]
    InvalidNumber(SfSlice<'a>),
    /// Char is invalid, because it does not contain a character. It should look like this: ''.
    #[error("Char literals cannot be empty")]
    EmptyChar(SfSlice<'a>),
    /// Char is invalid, because it contains more than one character. Can be misleading, because an accented
    /// character is represented as two Rust char's. For example ë would look like ¨ e.
    #[error("Char {0} is invalid. Char literals can only hold one character (maybe you want a string: \"...\"?")]
    TooFullChar(SfSlice<'a>),
    /// No valid token type was found for this substring even ident,
    /// which are just alphanumeric sequences with underscores.
    #[error("could not parse this substring \"{ITALIC_START}{0}{ITALIC_END}\" (neither ident, nor other token)")]
    Unparseable(SfSlice<'a>),
}

impl CompilerError for LiteralError<'_> {
    fn lint(&self) -> Option<Lint> {
        let slice = match self {
            Self::EmptyChar(s, ..) |
            Self::InvalidNumber(s, ..) |
            Self::TooFullChar(s, ..) |
            Self::Unparseable(s, ..) => s,
        };

        Some(Lint::from_slice_error(slice.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{absolute, PathBuf};

    fn test_file() -> SourceFile {
        let path = PathBuf::from("./test-resources/small-fib.bfu");
        let abs_path = absolute(path).unwrap();
        SourceFile::from_file(abs_path).unwrap()
    }

    #[test]
    fn lexing_does_not_panic() {
        lex_file(&test_file());
    }

    #[test]
    fn lexing_does_not_error() {
        assert_eq!(lex_file(&test_file()).1.len(), 0);
    }
}
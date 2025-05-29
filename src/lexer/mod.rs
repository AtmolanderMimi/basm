//! Defines tooling used to parse a string into a chain of language tokens.
//! As a user, you should probably be looking for [`lex_file`], all of this module's
//! code is put to use in there.

pub mod token;
use std::{ops::Range, vec::IntoIter};

use thiserror::Error;
use token::{Token, TokenType};

use crate::{error::{CompilerError, Lint}, source::{SfSlice, SourceFile}, utils::Sliceable};

struct Lexer {
    range: Range<usize>,
    // TODO: not the most efficient, but definitly better than what i did before
    // (recalculate all chars for each slice into source)
    character_indexes_iter: IntoIter<usize>,
    source: &'static SourceFile,
    tokens: Vec<Token>,
    comment_mode: bool,
}

impl Lexer {
    pub fn new(source_file: &'static SourceFile) -> Lexer {
        let mut character_indexes = source_file.as_ref()
            .char_indices()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        character_indexes.push(source_file.lenght());
        let character_indexes_iter = character_indexes.into_iter();

        Lexer {
            range: 0..0,
            character_indexes_iter,
            source: source_file,
            tokens: Vec::new(),
            comment_mode: false,
        }
    }

    pub fn advance(&mut self) -> Result<Advancement, LexerError> {
        let Some(new_end) = self.character_indexes_iter.next() else {
            return Ok(Advancement::Finished)
        };

        self.range.end = new_end;

        let sf_slice = self.source.slice(self.range.clone())
            .unwrap();

        if self.comment_mode && sf_slice.inner_slice().contains('\n') {
            self.comment_mode = false;
            self.range.start = self.range.end;

            return Ok(Advancement::Advancing);
        } else if self.comment_mode {
            // we don't want to match any more tokens before reaching the end of a line
            return Ok(Advancement::Advancing);
        }

        if let Some(non_lit) = Token::parse_token_non_lit(&sf_slice) {
            let possibly_lit_range = self.range.start..non_lit.slice.start();
            let possibly_lit_slice = self.source.slice(possibly_lit_range)
                .unwrap();

            self.range.start = non_lit.slice.end();

            if let Some(lit) = Token::parse_token_lit(&possibly_lit_slice)? {
                self.tokens.push(lit);
            }

            // we don't want line comments into the ast
            if non_lit.t_type == TokenType::LineComment {
                self.comment_mode = true;
            } else {
                self.tokens.push(non_lit);
            }
        }

        // kind-of a cheat, instead of waiting for a non-lit, if there is a space: check
        let string = sf_slice.inner_slice();
        let in_string = string.chars().filter(|c| *c == '\"').count() % 2 == 1;
        let in_char = string.chars().filter(|c| *c == '\'').count() % 2 == 1;
        if (string.ends_with(' ') || string.ends_with('\n')) && !(in_string || in_char) {
            if let Some(lit) = Token::parse_token_lit(&sf_slice)? {
                self.tokens.push(lit);
                self.range.start = self.range.end;
            }
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
pub fn lex_file(source_file: &'static SourceFile) -> Result<Vec<Token>, (Vec<Token>, Vec<LexerError>)> {
    let mut errors = Vec::new();
    let mut lexer = Lexer::new(source_file);
    loop {
        match lexer.advance() {
            Ok(Advancement::Finished) => break,
            Ok(Advancement::Advancing) => (),
            Err(e) => errors.push(e),
        }
    }

    let file_lenght = source_file.lenght();
    let eof_slice = source_file.slice(file_lenght..file_lenght)
        .expect("slice should be valid");
    let eof = Token::new(TokenType::Eof, eof_slice);
    lexer.tokens.push(eof);

    if errors.is_empty() {
        Ok(lexer.tokens)
    } else {
        Err((lexer.tokens, errors))
    }
}

enum Advancement {
    Advancing,
    Finished,
}

/// An error that occured during the lexing process.
#[derive(Error, Debug)]
pub enum LexerError {
    /// There was an error while forming a literal.
    #[error("{0}")]
    InvalidLiteral(LiteralError),
}

impl CompilerError for LexerError {
    fn lint(&self) -> Option<Lint> {
        match self {
            LexerError::InvalidLiteral(e) => e.lint()
        }
    }
}

impl From<LiteralError> for LexerError {
    fn from(value: LiteralError) -> Self {
        LexerError::InvalidLiteral(value)
    }
}

const ITALIC_START: &str = "\x1B[3m";
const ITALIC_END: &str = "\x1B[23m";

#[derive(Error, Debug, PartialEq)]
/// An error encounted while trying to parse for a literal syntax token.
/// 
/// *`"Yo... This is literally an error, man..."`*
pub enum LiteralError {
    /// Number is invalid, because it is not within the range that bf can store.
    #[error("number {0} could not be parsed, probably too big")]
    InvalidNumber(SfSlice),
    /// Char is invalid, because it does not contain a character. It should look like this: ''.
    #[error("character literals cannot be empty")]
    EmptyChar(SfSlice),
    /// Char is invalid, because it contains more than one character. Can be misleading, because an accented
    /// character is represented as two Rust char's. For example ë would look like ¨ e.
    #[error("character {0} is invalid. Character literals can only hold one character (maybe you want a string: \"...\"?)")]
    TooFullChar(SfSlice),
    /// No valid token type was found for this substring even ident,
    /// which are just alphanumeric sequences with underscores.
    #[error("could not parse this substring \"{ITALIC_START}{0}{ITALIC_END}\" (neither ident, nor other token)")]
    Unparseable(SfSlice),
}

impl CompilerError for LiteralError {
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
        let path = PathBuf::from("./test-resources/fib.basm");
        let abs_path = absolute(path).unwrap();
        SourceFile::from_file(abs_path).unwrap()
    }

    #[test]
    fn lexing_does_not_panic() {
        let _ = lex_file(test_file().leak());
    }

    #[test]
    fn lexing_does_not_error() {
        assert!(lex_file(test_file().leak()).is_ok());
    }
}
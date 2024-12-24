//! Tools used to parse a string of tokens into sensible a sensible structure (the [`ParsedProgram`] struct).

mod terminals;
mod componants;
mod expression;
mod instruction;
mod scope;
mod fields;

use componants::{Many, Then};
use fields::{MainFieldPattern, MetaFieldPattern};
use thiserror::Error;

use crate::{lexer::token::{Token, TokenType}, source::SfSlice, CompilerError, Lint};

#[allow(unused_imports)]
pub use terminals::{Ident, NumLit, CharLit, Plus, Minus, Semicolon, LeftSquare, RightSquare, At, MainIdent};
#[allow(unused_imports)]
pub use expression::{Expression, ValueRepresentation};
#[allow(unused_imports)]
pub use fields::{MainField, MetaField};
#[allow(unused_imports)]
pub use instruction::Instruction;
#[allow(unused_imports)]
pub use scope::Scope;

/// Defines a language pattern.
trait Pattern<'a>: Default {
    type ParseResult: Clone;

    /// Advances a pattern.
    /// The patterns becomes invalid after returning `Done` or `NotExpected`.
    /// **Any calls of this method after this, are considered undefined behaviour.**
    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult>;
}

#[derive(Debug, Clone, PartialEq)]
enum AdvancementState<T> {
    Advancing,
    Done(T),
    Error(PatternMatchingError),
}

struct Advancement<T> {
    // The number of tokens that were not used to make a decision, but not included in the pattern.
    pub overeach: usize,
    pub state: AdvancementState<T>,
}

impl<T> Advancement<T> {
    pub fn new_no_overeach(state: AdvancementState<T>) -> Advancement<T> {
        Advancement {
            overeach: 0,
            state,
        }
    }

    pub fn new(state: AdvancementState<T>, overeach: usize) -> Advancement<T> {
        Advancement {
            overeach,
            state,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum PatternMatchingError {
    #[error("expected {expected:?} token, got {got:?}")] // got got :3
    UnexpectedToken {
        expected: TokenType,
        got: Token<'static>, // TODO: i didn't want to deal with more lifetimes
    },
}

impl CompilerError for PatternMatchingError {
    fn lint(&self) -> Option<crate::Lint> {
        let l = match self {
            Self::UnexpectedToken { got, .. } => {
                Lint::from_slice_error(got.slice.clone())
            }
        };

        Some(l)
    }
}

/// Feeds a pattern with tokens, implements backtracking with when overeaching.
#[derive(Debug, Clone, PartialEq)]
struct PatternFeeder<'a, 'b, T: Pattern<'a>> {
    pattern: T,
    tokens: &'b Vec<Token<'a>>,
    current_token: usize,
}

impl<'a, 'b: 'a, T: Pattern<'a>> PatternFeeder<'a, 'b, T> {
    fn new(tokens: &'b Vec<Token<'a>>) -> Self {
        PatternFeeder {
            pattern: T::default(),
            tokens,
            current_token: 0,
        }
    }

    // Advances by one token. Should be considered UB after getting Error or Done.
    fn advance(&mut self) -> Advancement<T::ParseResult> {
        let token = self.tokens.get(self.current_token)
            .expect("patterns should end before running out of tokens");

        let adv = self.pattern.advance(token);
        self.current_token += 1;
        self.current_token -= adv.overeach;

        adv
    }
}

fn solve_pattern<'a, 'b: 'a, T: Pattern<'a> + 'a>(tokens: &'b Vec<Token<'a>>) -> Result<T::ParseResult, PatternMatchingError> {
    let mut feeder: PatternFeeder<'_, '_, T> = PatternFeeder::new(tokens);

    loop {
        match feeder.advance().state {
            AdvancementState::Advancing => (),
            AdvancementState::Done(res) => return Ok(res),
            AdvancementState::Error(e) => return Err(e),
        }
    }
}

/// Pattern for constructing a [`ParsedProgram`].
#[derive(Debug, Clone, PartialEq, Default)]
struct ProgramPattern<'a>(
    Then<'a, Many<'a, MetaFieldPattern<'a>>, MainFieldPattern<'a>>
);

/// A whole, parsed, basm program.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedProgram<'a> {
    pub meta_instructions: Vec<MetaField<'a>>,
    pub main_field: MainField<'a>,
}

impl<'a> Pattern<'a> for ProgramPattern<'a> {
    type ParseResult = ParsedProgram<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
        use AdvancementState as AdvState;

        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = ParsedProgram {
                    meta_instructions: res.0,
                    main_field: res.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

/// A generic trait to be implemented onto each language item.
pub trait LanguageItem<'a> {
    type Owned;

    /// A slice defining the position of the language item.
    fn slice(&self) -> SfSlice<'a>;
    /// Turns the language item into an owned form.
    /// (usually by copying)
    fn into_owned(self) -> Self::Owned;
}

impl<'a> LanguageItem<'a> for Token<'a> {
    type Owned = Token<'static>;

    fn into_owned(self) -> Self::Owned {
        Token::into_owned(self)
    }

    fn slice(&self) -> SfSlice<'a> {
        self.slice.clone()
    }
}

/// Parses the tokens into a structured form ([`ParsedProgram`]).
pub fn parse_tokens<'a, 'b: 'a>(tokens: &'b Vec<Token<'a>>) -> Result<ParsedProgram<'a>, PatternMatchingError> {
    solve_pattern::<ProgramPattern>(tokens)
}

#[cfg(test)]
mod tests {
    use std::path::absolute;

    use crate::{lex_file, source::{SfSlice, SourceFile}};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token<'static> {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn parsed_file_token_pattern() {
        // normal
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(732),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<ProgramPattern>(&tokens).unwrap();
        assert_eq!(res.meta_instructions.len(), 1);

        // no [main]
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<ProgramPattern>(&tokens);
        assert!(res.is_err());

        // no meta
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(732),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<ProgramPattern>(&tokens).unwrap();
        assert_eq!(res.meta_instructions.len(), 0);
    }

    #[test]
    fn parsing_fib_example() {
        let abs_path = absolute("./test-resources/fib.basm").unwrap();
        let file = SourceFile::from_file(&abs_path).unwrap();
        let res = lex_file(&file);
        assert!(res.1.is_empty());
        let tokens = res.0;
        let program = solve_pattern::<ProgramPattern>(&tokens).unwrap();

        assert_eq!(program.meta_instructions.len(), 1);
        assert_eq!(program.main_field.contents.contents.len(), 7);
    }
}

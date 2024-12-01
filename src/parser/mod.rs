mod terminals;
mod componants;
mod expression;

use thiserror::Error;

use crate::{lexer::token::{Token, TokenType}, CompilerError, Lint};

/// Defines a language pattern.
pub trait Pattern<'a>: Default {
    type ParseResult: Clone;

    /// Advances a pattern.
    /// The patterns becomes invalid after returning `Done` or `NotExpected`.
    /// **Any calls of this method after this, are considered undefined behaviour.**
    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdvancementState<T> {
    Advancing,
    Done(T),
    Error(PatternMatchingError),
}

pub struct Advancement<T> {
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

pub fn solve_pattern<'a, 'b: 'a, T: Pattern<'a> + 'a>(tokens: &'b Vec<Token<'a>>) -> Result<T::ParseResult, PatternMatchingError> {
    let mut feeder: PatternFeeder<'_, '_, T> = PatternFeeder::new(tokens);

    loop {
        match feeder.advance().state {
            AdvancementState::Advancing => (),
            AdvancementState::Done(res) => return Ok(res),
            AdvancementState::Error(e) => return Err(e),
        }
    }
}
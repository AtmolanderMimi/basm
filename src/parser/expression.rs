//! Defines what is an expression and how to find it.

use either::Either;

use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::Sliceable;

use super::terminals::{CharLit, CharLitPattern, Ident, IdentPattern, Minus, MinusPattern, NumLit, NumLitPattern, Plus, PlusPattern};
use super::componants::{Many, Or, Then};
use super::Advancement;
use super::LanguageItem;
use super::Pattern;
use super::AdvancementState as AdvState;

// I can't quite decide if encoding the patterns is to best or the worst idea i've had in my programing career thus far.

/// Pattern for building a [`ValueRepresentation`].
#[derive(Debug, Clone, PartialEq, Default)]
struct ValuePattern(
    Or<IdentPattern, Or<NumLitPattern, CharLitPattern>>,
);

/// Something that represents a static value which can be gotten at compile-time.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueRepresentation {
    /// An ident, in this case it is an alias.
    Ident(Ident),
    /// A number literal.
    NumLit(NumLit),
    /// A character literal.
    CharLit(CharLit),
}

impl Pattern for ValuePattern {
    type ParseResult = ValueRepresentation;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = match res {
                    Either::Left(i) => ValueRepresentation::Ident(i),
                    Either::Right(Either::Left(n)) => ValueRepresentation::NumLit(n),
                    Either::Right(Either::Right(c)) => ValueRepresentation::CharLit(c),
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

/// Pattern for building a [`Mod`].
#[derive(Debug, Clone, PartialEq, Default)]
struct ModPattern(
    Then<Or<PlusPattern, MinusPattern>, ValuePattern>
);

impl Pattern for ModPattern {
    type ParseResult = Mod;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = match res.0 {
                    Either::Left(p) => Mod::Increment { plus_token: p, value: res.1 },
                    Either::Right(m) => Mod::Decrement { minus_token: m, value: res.1 },
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Represents a modification applied to a value. Like this: `+3`.
pub enum Mod {
    /// An increment modifyer.
    #[allow(missing_docs)]
    Increment {
        plus_token: Plus,
        value: ValueRepresentation,
    },
    /// An decrement modifyer.
    #[allow(missing_docs)]
    Decrement {
        minus_token: Minus,
        value: ValueRepresentation,
    },
}

/// Pattern for building a [`Expression`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExpressionPattern(
    Then<
        ValuePattern,
        Many<ModPattern>
    >
);

impl Pattern for ExpressionPattern {
    type ParseResult = Expression;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = Expression {
                    base: res.0,
                    mods: res.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

/// Represents an expression.
/// An expression is simply a [`ValueRepresentation`] which is can be offset by one or more others.
/// These offsets are [`Mod`]'s.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Expression {
    pub base: ValueRepresentation,
    pub mods: Vec<Mod>,
}

impl LanguageItem for ValueRepresentation {
    fn slice(&self) -> SfSlice {
        match self {
            Self::CharLit(c) => c.slice(),
            Self::NumLit(c) => c.slice(),
            Self::Ident(c) => c.slice(),
        }
    }
}

impl LanguageItem for Expression {
    fn slice(&self) -> SfSlice {
        let start = self.base.slice().start();
        let end = self.mods.last()
            .map_or(self.base.slice().end(), |l| l.slice().end());
        self.base.slice().source().slice(start..end)
            .unwrap()
    }
}

impl LanguageItem for Mod {
    fn slice(&self) -> SfSlice {
        match self {
            Self::Increment { plus_token, value } => {
                let start = plus_token.slice().start();
                let end = value.slice().end();
                plus_token.slice().source().slice(start..end)
                    .unwrap()
            }
            Self::Decrement { minus_token, value } => {
                let start = minus_token.slice().start();
                let end = value.slice().end();
                minus_token.slice().source().slice(start..end)
                    .unwrap()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{lexer::token::TokenType, parser::solve_pattern, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn expression_token_pattern() {
        let tokens = vec![
            TokenType::Ident("sp".to_string()),
            TokenType::Plus,
            TokenType::NumLit(72),
            TokenType::Minus,
            TokenType::Ident("i".to_string()),
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let expr = solve_pattern::<ExpressionPattern>(&tokens);
        let expr = expr.unwrap();
        assert_eq!(expr.mods.len(), 2);
        assert_matches!(expr.mods[0], Mod::Increment { .. });
        assert_matches!(expr.mods[1], Mod::Decrement { .. });


        let tokens = vec![
            TokenType::CharLit('d'),
            TokenType::Minus,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let expr = solve_pattern::<ExpressionPattern>(&tokens);
        let expr = expr.unwrap();
        assert_eq!(expr.mods.len(), 0);


        let tokens = vec![
            TokenType::InstructionDelimitor,
            TokenType::Minus,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let expr = solve_pattern::<ExpressionPattern>(&tokens);
        assert!(expr.is_err())
    }
}

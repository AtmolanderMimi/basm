//! Defines what is an expression and how to find it.

use either::Either;

use crate::compiler::ScopeContext;
use crate::lexer::token::Token;
use crate::lexer::token::TokenType;
use crate::source::SfSlice;

use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::LanguageItem;
use super::Pattern;
use super::AdvancementState as AdvState;

// I can't quite decide if encoding the patterns is to best or the worst idea i've had in my programing career thus far.

/// Pattern for building a [`ValueRepresentation`].
#[derive(Debug, Clone, PartialEq, Default)]
struct ValuePattern<'a>(
    Or<'a, IdentPattern, Or<'a, NumLitPattern, CharLitPattern>>,
);

/// Something that represents a static value which can be gotten at compile-time.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueRepresentation<'a> {
    /// An ident, in this case it is an alias.
    Ident(Ident<'a>),
    /// A number literal.
    NumLit(NumLit<'a>),
    /// A character literal.
    CharLit(CharLit<'a>),
}

impl<'a> Pattern<'a> for ValuePattern<'a> {
    type ParseResult = ValueRepresentation<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
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
struct ModPattern<'a>(
    Then<'a, Or<'a, PlusPattern, MinusPattern>, ValuePattern<'a>>
);

impl<'a> Pattern<'a> for ModPattern<'a> {
    type ParseResult = Mod<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
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
pub enum Mod<'a> {
    Increment {
        plus_token: Plus<'a>,
        value: ValueRepresentation<'a>,
    },
    Decrement {
        minus_token: Minus<'a>,
        value: ValueRepresentation<'a>,
    },
}

/// Pattern for building a [`Expression`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExpressionPattern<'a>(
    Then<'a,
        ValuePattern<'a>,
        Many<'a, ModPattern<'a>>
    >
);

impl<'a> Pattern<'a> for ExpressionPattern<'a> {
    type ParseResult = Expression<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
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
pub struct Expression<'a> {
    #[allow(missing_docs)]
    pub base: ValueRepresentation<'a>,
    #[allow(missing_docs)]
    pub mods: Vec<Mod<'a>>,
}

impl<'a> Expression<'a> {
    /// Evaluates the expression in the context.
    /// Returns `None` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &ScopeContext<'_>) -> Option<u32> {
        let mut base = self.base.evaluate(ctx)?;

        for m in &self.mods {
            match m {
                Mod::Increment { value, .. } => {
                    base = base.overflowing_add(value.evaluate(ctx)?).0;
                },
                Mod::Decrement { value, .. } => {
                    base = base.overflowing_sub(value.evaluate(ctx)?).0;
                },
            }
        }

        Some(base)
    }
}

impl<'a> ValueRepresentation<'a> {
    /// Evaluates the value in the context.
    /// Returns `None` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &ScopeContext<'_>) -> Option<u32> {
        match self {
            Self::NumLit(n) => {
                if let TokenType::NumLit(v) = n.0.t_type {
                    Some(v)
                } else {
                    panic!("NumLit should be a token of type NumLit")
                }
            },
            Self::CharLit(c) => {
                if let TokenType::CharLit(c) = c.0.t_type {
                    Some(c.into())
                } else {
                    panic!("CharLit should be a token of type CharLit")
                }
            },
            Self::Ident(i) => {
                if let TokenType::Ident(i) = &i.0.t_type {
                    // search if the ident exists
                    let value = ctx.find_alias(i)?;
                    Some(value)
                } else {
                    panic!("Ident should be a token of type Ident")
                }
            }
        }
    }
}

impl<'a> LanguageItem<'a> for ValueRepresentation<'a> {
    type Owned = ValueRepresentation<'static>;

    fn into_owned(self) -> Self::Owned {
        match self {
            Self::CharLit(c) => ValueRepresentation::CharLit(c.into_owned()),
            Self::NumLit(c) => ValueRepresentation::NumLit(c.into_owned()),
            Self::Ident(c) => ValueRepresentation::Ident(c.into_owned()),
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        match self {
            Self::CharLit(c) => c.slice(),
            Self::NumLit(c) => c.slice(),
            Self::Ident(c) => c.slice(),
        }
    }
}

impl<'a> LanguageItem<'a> for Expression<'a> {
    type Owned = Expression<'static>;

    fn into_owned(self) -> Self::Owned {
        Expression {
            base: self.base.into_owned(),
            mods: self.mods.into_iter().map(|m| m.into_owned()).collect(),
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        let start = self.base.slice().start();
        let end = self.mods.last()
            .map(|l| l.slice().end()).unwrap_or(self.base.slice().end());
        self.base.slice().reslice_char(start..end)
    }
}

impl<'a> LanguageItem<'a> for Mod<'a> {
    type Owned = Mod<'static>;

    fn into_owned(self) -> Self::Owned {
        match self {
            Self::Increment { plus_token, value } => Mod::Increment {
                plus_token: plus_token.into_owned(),
                value: value.into_owned(),
            },
            Self::Decrement { minus_token, value } => Mod::Decrement {
                minus_token: minus_token.into_owned(),
                value: value.into_owned(),
            },
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        match self {
            Self::Increment { plus_token, value } => {
                let start = plus_token.slice().start();
                let end = value.slice().end();
                plus_token.slice().reslice_char(start..end)
            }
            Self::Decrement { minus_token, value } => {
                let start = minus_token.slice().start();
                let end = value.slice().end();
                minus_token.slice().reslice_char(start..end)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{lexer::token::TokenType, parser::solve_pattern, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token<'static> {
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
        .map(|tt| bogus_token(tt)).collect();

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
        .map(|tt| bogus_token(tt)).collect();

        let expr = solve_pattern::<ExpressionPattern>(&tokens);
        let expr = expr.unwrap();
        assert_eq!(expr.mods.len(), 0);


        let tokens = vec![
            TokenType::InstructionDelimitor,
            TokenType::Minus,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let expr = solve_pattern::<ExpressionPattern>(&tokens);
        assert!(expr.is_err())
    }
}

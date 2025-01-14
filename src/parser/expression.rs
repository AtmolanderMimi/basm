//! Defines what is an expression and how to find it.

use either::Either;

use crate::compiler::AlisValue;
use crate::compiler::NormalizedScope;
use crate::compiler::ScopeContext;
use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::CharOps;

use super::terminals::*;
use super::componants::*;
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
    Increment {
        plus_token: Plus,
        value: ValueRepresentation,
    },
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
pub struct Expression {
    #[allow(missing_docs)]
    pub base: ValueRepresentation,
    #[allow(missing_docs)]
    pub mods: Vec<Mod>,
}

impl Expression {
    /// Evaluates the expression in the context.
    /// Returns `None` if an alias is not defined in the context or if the result is not numeric.
    pub fn evaluate_numeric(&self, ctx: &ScopeContext<'_>) -> Option<u32> {
        if let Either::Left(v) = self.evaluate(ctx)? {
            Some(v)
        } else {
            None
        }
    }

    /// Evaluates the expression in the context.
    /// Returns `None` if an alias is not defined in the context.
    pub fn evaluate<'a>(&self, ctx: &'a ScopeContext<'_>) -> Option<Either<u32, &'a NormalizedScope>> {
        match self.base.evaluate(ctx)? {
            Either::Left(mut base) => {
                for m in &self.mods {
                    match m {
                        Mod::Increment { value, .. } => {
                            base = base.overflowing_add(value.evaluate_numeric(ctx)?).0;
                        },
                        Mod::Decrement { value, .. } => {
                            base = base.overflowing_sub(value.evaluate_numeric(ctx)?).0;
                        },
                    }
                }

                Some(Either::Left(base))
            },
            Either::Right(scp) => {
                // we should not be able to add modifiers to a scope alis
                if !self.mods.is_empty() {
                    return None
                }

                Some(Either::Right(scp))
            }
        }
    }
}

impl ValueRepresentation {
    /// Evaluates the numeric value in the context.
    /// Returns `None` if an alias is not defined in the context or if the the alias is not numeric.
    pub fn evaluate_numeric(&self, ctx: &ScopeContext<'_>) -> Option<u32> {
        match self.evaluate(ctx)? {
            Either::Left(v) => Some(v),
            Either::Right(_) => None,
        }
    }

    /// Evaluates the value in the context, may be a scope if it is an alis.
    /// Returns `None` if an alias is not defined in the context.
    pub fn evaluate<'a>(&self, ctx: &'a ScopeContext<'_>) -> Option<Either<u32, &'a NormalizedScope>> {
        match self {
            Self::NumLit(n) => Some(Either::Left(n.value())),
            Self::CharLit(c) => Some(Either::Left(c.value().into())),
            Self::Ident(i) => {
                let alias = ctx.find_alias(i.value())?;
                Some(match alias {
                    AlisValue::Value(v) => Either::Left(*v),
                    AlisValue::Scope(s) => Either::Right(s)
                })
            },
        }
    }
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
        let start = self.base.slice().start_char();
        let end = self.mods.last()
            .map(|l| l.slice().end_char()).unwrap_or(self.base.slice().end_char());
        self.base.slice().source().slice_char(start..end)
            .unwrap()
    }
}

impl LanguageItem for Mod {
    fn slice(&self) -> SfSlice {
        match self {
            Self::Increment { plus_token, value } => {
                let start = plus_token.slice().start_char();
                let end = value.slice().end_char();
                plus_token.slice().source().slice_char(start..end)
                    .unwrap()
            }
            Self::Decrement { minus_token, value } => {
                let start = minus_token.slice().start_char();
                let end = value.slice().end_char();
                minus_token.slice().source().slice_char(start..end)
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

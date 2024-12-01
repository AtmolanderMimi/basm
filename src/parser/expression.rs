//! Defines what is an expression and how to find it.

use either::Either;

use crate::lexer::token::Token;

use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::Pattern;
use super::AdvancementState as AdvState;

// I can't quite decide if encoding the patterns is to best or the worst idea i've had in my programing career thus far.

/// Pattern for building a [`ValueRepresentation`].
#[derive(Debug, Clone, PartialEq, Default)]
struct ValuePattern<'a>(
    Or<'a, IdentPattern, Or<'a, NumLitPattern, CharLitPattern>>,
);

/// Something that represents a value.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueRepresentation<'a> {
    Ident(Ident<'a>),
    NumLit(NumLit<'a>),
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
#[derive(Debug, Clone, PartialEq)]
pub struct Expression<'a> {
    base: ValueRepresentation<'a>,
    mods: Vec<Mod<'a>>,
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

//! Defines operations, both binary and unary
//! Here is a table of precedence:
//! "(", ")"                         => 0
//! "==", "!=", ">", ">=", "<", "<=" => 1
//! "+", "-"                         => 2
//! "*", "/", "%"                    => 3
//! "!"                              => 4 // TODO: implement "!"
//! "@" (right-associative)          => 5

use crate::{lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, UnexpectedTokenError, patterns::{AtPattern, IdentPattern, LeftSquarePattern, MinusPattern, MultiplyPattern, NumLitPattern, Or, RightSquarePattern, Then}, terminals::{At, Divide, DividePattern, GreaterThan, GreaterThanEqual, GreaterThanEqualPattern, GreaterThanPattern, Ident, LeftSquare, LessThan, LessThanEqual, LessThanEqualPattern, LessThanPattern, LogicalEqual, LogicalEqualPattern, LogicalInequal, LogicalInequalPattern, LogicalNot, Minus, Modulo, ModuloPattern, Multiply, NumLit, Plus, PlusPattern, RightSquare}}};

pub trait Operator {
    const NB_PRECEDENCE_LEVELS: u32 = 6;
    const RIGHT_ASSOCIATIVE_LEVELS: &[u32] = &[5];

    /// The order the operations should be merged in (higher = earlier)
    fn precedence(&self) -> u32;
}

/// An operator with two operands on both of it's sides
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus(Plus),
    Minus(Minus),
    Multiply(Multiply),
    Divide(Divide),
    Modulo(Modulo),
    LogicalEqual(LogicalEqual),
    LogicalInequal(LogicalInequal),
    GreaterThan(GreaterThan),
    GreaterThanEqual(GreaterThanEqual),
    LessThan(LessThan),
    LessThanEqual(LessThanEqual),
    Index(At),
}

impl LanguageItem for BinaryOperator {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::Plus(t) => t.slice(),
            Self::Minus(t) => t.slice(),
            Self::Multiply(t) => t.slice(),
            Self::Divide(t) => t.slice(),
            Self::Modulo(t) => t.slice(),
            Self::LogicalEqual(t) => t.slice(),
            Self::LogicalInequal(t) => t.slice(),
            Self::GreaterThan(t) => t.slice(),
            Self::GreaterThanEqual(t) => t.slice(),
            Self::LessThan(t) => t.slice(),
            Self::LessThanEqual(t) => t.slice(),
            Self::Index(t) => t.slice(),
        }
    }
}

impl Operator for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::LogicalEqual(_)
            | Self::LogicalInequal(_)
            | Self::GreaterThan(_)
            | Self::GreaterThanEqual(_)
            | Self::LessThan(_)
            | Self::LessThanEqual(_) => 1,
            Self::Plus(_)
            | Self::Minus(_) => 2,
            Self::Multiply(_)
            | Self::Divide(_)
            | Self::Modulo(_) => 3,
            Self::Index(_) => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BinaryOperatorPattern;

impl Pattern for BinaryOperatorPattern {
    type ParseResult = BinaryOperator;

    // TODO: this implementation is better than by using a long chain of Or<>'s,
    // but is still very unelegant
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (nb_tokens, operator) = if let Ok((nb_tokens, parsed)) = PlusPattern::solve(&tokens) {
            (nb_tokens, BinaryOperator::Plus(parsed))
        } else if let Ok((nb_tokens, parsed)) = MinusPattern::solve(&tokens) {
            (nb_tokens, BinaryOperator::Minus(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = MultiplyPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::Multiply(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = DividePattern::solve(tokens) {
            (nb_tokens, BinaryOperator::Divide(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = ModuloPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::Modulo(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LogicalEqualPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalEqual(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LogicalInequalPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalInequal(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = GreaterThanPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThan(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = GreaterThanEqualPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThanEqual(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LessThanPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThan(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LessThanEqualPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThanEqual(parsed))
        } else if let Ok((nb_tokens, parsed)) = AtPattern::solve(tokens) {
            (nb_tokens, BinaryOperator::Index(parsed))
        } else {
            // TODO:
            return Err(UnexpectedTokenError::new_got_nothing(todo!("add all types")))
        };

        Ok((nb_tokens, operator))
    }
}

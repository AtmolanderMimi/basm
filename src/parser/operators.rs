//! Defines operations, both binary and unary
//! Here is a table of precedence:
//! "==", "!=", ">", ">=", "<", "<=" => 0
//! "+", "-"                         => 1
//! "*", "/", "%"                    => 2
//! "!"                              => 3 // TODO: implement "!"
//! "@" (right-associative)          => 4

use crate::{lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, UnexpectedTokenError, terminals::{At, Divide, GreaterThan, GreaterThanEqual, LessThan, LessThanEqual, LogicalEqual, LogicalInequal, Minus, Modulo, Multiply, Plus}}};

pub trait Operator {
    const NB_PRECEDENCE_LEVELS: u32 = 5;
    const RIGHT_ASSOCIATIVE_LEVELS: &[u32] = &[4];

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
            | Self::LessThanEqual(_) => 0,
            Self::Plus(_)
            | Self::Minus(_) => 1,
            Self::Multiply(_)
            | Self::Divide(_)
            | Self::Modulo(_) => 2,
            Self::Index(_) => 4,
        }
    }
}

impl Pattern for BinaryOperator {
    // TODO: this implementation is better than by using a long chain of Or<>'s,
    // but is still very unelegant
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (nb_tokens, operator) = if let Ok((nb_tokens, parsed)) = Plus::solve(&tokens) {
            (nb_tokens, BinaryOperator::Plus(parsed))
        } else if let Ok((nb_tokens, parsed)) = Minus::solve(&tokens) {
            (nb_tokens, BinaryOperator::Minus(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = Multiply::solve(tokens) {
            (nb_tokens, BinaryOperator::Multiply(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = Divide::solve(tokens) {
            (nb_tokens, BinaryOperator::Divide(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = Modulo::solve(tokens) {
            (nb_tokens, BinaryOperator::Modulo(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LogicalEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalEqual(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LogicalInequal::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalInequal(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = GreaterThan::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThan(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = GreaterThanEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThanEqual(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LessThan::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThan(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = LessThanEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThanEqual(parsed))
        } else if let Ok((nb_tokens, parsed)) = At::solve(tokens) {
            (nb_tokens, BinaryOperator::Index(parsed))
        } else {
            // TODO:
            return Err(UnexpectedTokenError::new_got_nothing(todo!("add all types")))
        };

        Ok((nb_tokens, operator))
    }
}

//! Defines operations, both binary and unary
//! Here is a table of precedence:
//! "||"                   => 0
//! "&&"                   => 1
//! ">", ">=", "<", "<="   => 2
//! "==", "!=",            => 3
//! "+", "-"               => 4
//! "*", "/", "%"          => 5
//! "!"                    => 6
//! "@",                   => 7
//! "."                    => 8

use crate::{lexer::token::Token, parser::{LanguageItem, ParseError, Pattern, PatternResult, terminals::{At, Divide, GreaterThan, GreaterThanEqual, LessThan, LessThanEqual, LogicalAnd, LogicalEqual, LogicalInequal, LogicalNot, LogicalOr, Minus, Modulo, Multiply, Period, Plus}}};

/// The number of precedence levels for all operators
pub const NB_PRECEDENCE_LEVELS: u32 = 9;

pub trait Operator {
    /// The order the operations should be merged in (higher = earlier)
    fn precedence(&self) -> u32;

    /// Wheter the operator is right associative.
    fn is_right_associative(&self) -> bool {
        false
    }

    /// Retruns true if it can modify an emplacement on it's left (first child of an expression).
    /// (Modifying an emplacement means that it takes an emplacement and returns one).
    fn modifies_an_emplacement(&self) -> bool;
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
    LogicalOr(LogicalOr),
    LogicalAnd(LogicalAnd),
    Index(At),
    Property(Period),
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
            Self::LogicalOr(t) => t.slice(),
            Self::LogicalAnd(t) => t.slice(),
            Self::Index(t) => t.slice(),
            Self::Property(t) => t.slice(),
        }
    }
}

impl Operator for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::LogicalOr(_) => 0,
            Self::LogicalAnd(_) => 1,
            Self::GreaterThan(_)
            | Self::GreaterThanEqual(_)
            | Self::LessThan(_)
            | Self::LessThanEqual(_) => 2,
            Self::LogicalEqual(_)
            | Self::LogicalInequal(_) => 3,
            Self::Plus(_)
            | Self::Minus(_) => 4,
            Self::Multiply(_)
            | Self::Divide(_)
            | Self::Modulo(_) => 5,
            Self::Index(_) => 7,
            Self::Property(_) => 8,
        }
    }

    fn modifies_an_emplacement(&self) -> bool {
        match self {
            Self::Index(_)
            | Self::Property(_) => true,
            _ => false,
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
        } else if let Ok((nb_tokens, parsed)) = Multiply::solve(tokens) {
            (nb_tokens, BinaryOperator::Multiply(parsed))
        } else if let Ok((nb_tokens, parsed)) = Divide::solve(tokens) {
            (nb_tokens, BinaryOperator::Divide(parsed))
        } else if let Ok((nb_tokens, parsed)) = Modulo::solve(tokens) {
            (nb_tokens, BinaryOperator::Modulo(parsed))
        } else if let Ok((nb_tokens, parsed)) = LogicalEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalEqual(parsed))
        } else if let Ok((nb_tokens, parsed)) = LogicalInequal::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalInequal(parsed))
        } else if let Ok((nb_tokens, parsed)) = GreaterThan::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThan(parsed))
        } else if let Ok((nb_tokens, parsed)) = GreaterThanEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::GreaterThanEqual(parsed))
        } else if let Ok((nb_tokens, parsed)) = LessThan::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThan(parsed))
        } else if let Ok((nb_tokens, parsed)) = LessThanEqual::solve(tokens) {
            (nb_tokens, BinaryOperator::LessThanEqual(parsed))
        } else if let Ok((nb_tokens, parsed)) = LogicalOr::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalOr(parsed))
        } else if let Ok((nb_tokens, parsed)) = LogicalAnd::solve(tokens) {
            (nb_tokens, BinaryOperator::LogicalAnd(parsed))
        } else if let Ok((nb_tokens, parsed)) = At::solve(tokens) {
            (nb_tokens, BinaryOperator::Index(parsed))
        } else if let Ok((nb_tokens, parsed)) = Period::solve(tokens) {
            (nb_tokens, BinaryOperator::Property(parsed))
        } else if let Some(token) = tokens.first() {
            // NOTE: this assumes that all binary operators only take one token
            return Err(ParseError::new_unexpected_token(1, Self::name(), token.clone()))
        } else {
            // NOTE: this too
            return Err(ParseError::new_no_more_tokens(1, Self::name()))
        };
        
        Ok((nb_tokens, operator))
    }

    fn name() -> String { "binaryop".to_string() }
}

/// An operator with with one operand, either on it's left or right
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    LogicalNot(LogicalNot),
}

impl LanguageItem for UnaryOperator {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::LogicalNot(t) => t.slice(),
        }
    }
}

impl Operator for UnaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::LogicalNot(_) => 6,
        }
    }

    fn is_right_associative(&self) -> bool {
        match self {
            Self::LogicalNot(_) => true,
        }
    }

    fn modifies_an_emplacement(&self) -> bool {
        false
    }
}

impl Pattern for UnaryOperator {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = LogicalNot::solve(&tokens)?;
        
        let operator = UnaryOperator::LogicalNot(res.1);

        Ok((res.0, operator))
    }

    fn name() -> String { "unaryop".to_string() }
}

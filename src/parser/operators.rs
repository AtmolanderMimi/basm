//! Defines operations, both binary and unary
//! Here is a table of precedence:
//! "||"                   => 0
//! "&&"                   => 1
//! ">", ">=", "<", "<="   => 2
//! "==", "!=",            => 3
//! "+", "-"               => 4
//! "*", "/", "%"          => 5
//! "!", "+", "-" (rev)    => 6
//! "[]", "."              => 7

use either::Either;

use crate::{lexer::token::Token, parser::{Expression, LanguageItem, ParseError, ParseErrorVariant, Pattern, PatternResult, pattern::{Or, Then}, terminals::*}, source::SfSlice, utils::Sliceable};

/// The number of precedence levels for all operators
pub const NB_PRECEDENCE_LEVELS: u32 = 8;
/// The precedence levels which are backwards, e.g: those which link operators end to start.
/// e.g: when there is a right-associative unary operator.
pub const REVERSE_PRECEDENCE_LEVELS: &[u32] = &[6];

pub trait Operator {
    /// The order the operations should be merged in (higher = earlier)
    fn precedence(&self) -> u32;

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
            Self::Property(_) => 7,
        }
    }

    fn modifies_an_emplacement(&self) -> bool {
        match self {
            Self::Property(_) => true,
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
pub enum RightAssociativeUnaryOperator {
    UnaryPlus(Plus),
    UnaryMinus(Minus),
    LogicalNot(LogicalNot),
}

impl LanguageItem for RightAssociativeUnaryOperator {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::UnaryPlus(t) => t.slice(),
            Self::UnaryMinus(t) => t.slice(),
            Self::LogicalNot(t) => t.slice(),
        }
    }
}

impl Operator for RightAssociativeUnaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::UnaryPlus(_)
            | Self::UnaryMinus(_)
            | Self::LogicalNot(_) => 6,
        }
    }

    fn modifies_an_emplacement(&self) -> bool {
        false
    }
}

impl Pattern for RightAssociativeUnaryOperator {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = Or::<
        Plus,
        Or<Minus,
        LogicalNot
        >>::solve(&tokens);
            
        // modifies the error to say unary op
        if let Err(ParseError { tokens_before_error, variant: ParseErrorVariant::UnexpectedTokenError{ got, ..} }) = res {
            return Err(ParseError::new_unexpected_token(tokens_before_error, Self::name(), got));
        };
        
        let (token_consumed, parsed) = res?;
        let operator = match parsed {
            Either::Left(op) => Self::UnaryPlus(op),
            Either::Right(Either::Left(op)) => Self::UnaryMinus(op),
            Either::Right(Either::Right(op)) => Self::LogicalNot(op),
        };

        Ok((token_consumed, operator))
    }

    fn name() -> String { "right-associative unaryop".to_string() }
}

/// An operator with with one operand which associates with the operand on its right.
#[derive(Debug, Clone, PartialEq)]
pub enum LeftAssociativeUnaryOperator {
    Index(LeftSquare, Box<Expression>, RightSquare),
}

impl LanguageItem for LeftAssociativeUnaryOperator {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::Index(lsquare, _, rsquare) => {
                let source = lsquare.slice().source();
                let start = lsquare.slice().start();
                let end = rsquare.slice().end();

                SfSlice::from_source(source, start..end)
                    .expect("from known byte slice")
            },
        }
    }
}

impl Operator for LeftAssociativeUnaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            Self::Index(..) => 7,
        }
    }

    fn modifies_an_emplacement(&self) -> bool {
        match self {
            Self::Index(..) => true,
        }
    }
}

impl Pattern for LeftAssociativeUnaryOperator {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        type IndexPattern = Then::<LeftSquare, Then<Expression, RightSquare>>;

        let res = IndexPattern::solve(tokens);
            
        // modifies the error to say unary op
        if let Err(ParseError { tokens_before_error, variant: ParseErrorVariant::UnexpectedTokenError{ got, ..} }) = res {
            return Err(ParseError::new_unexpected_token(tokens_before_error, Self::name(), got));
        };

        let (token_consumed, parsed) = res?;
        let operation = Self::Index(parsed.0, Box::new(parsed.1.0), parsed.1.1);

        Ok((token_consumed, operation))
    }

    fn name() -> String { "left-associative unaryop".to_string() }
}

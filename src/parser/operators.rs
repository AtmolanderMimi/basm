//! Defines operations, both binary and unary
//! Here is a table of precedence:
//! "(", ")"                         => 0
//! "==", "!=", ">", ">=", "<", "<=" => 1
//! "+", "-"                         => 2
//! "*", "/", "%"                    => 3
//! "!"                              => 4
//! "[]" (right-associative)         => 5

use either::Either;

use crate::{impl_language_item, lexer::token::Token, parser::{Pattern, PatternResult, patterns::{IdentPattern, LeftSquarePattern, NumLitPattern, Or, RightSquarePattern, Then}, terminals::{Divide, GreaterThan, GreaterThanEqual, Ident, LeftSquare, LessThan, LessThanEqual, LogicalEqual, LogicalInequal, LogicalNot, Minus, Modulo, Multiply, NumLit, Plus, RightSquare}}};

/// Info about operators and how they form an ast
trait Operator {
    /// The precedence of the operator
    fn precedence(&self) -> u32;

    // Whether the operator is left associative
    fn is_left_associative(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
struct IndexInto {
    pub left_bracket: LeftSquare,
    pub index: Either<Ident, NumLit>,
    pub right_bracket: RightSquare,
}
impl_language_item!(IndexInto, left_bracket, right_bracket);

#[derive(Debug, Clone, PartialEq, Default)]
struct IndexIntoPattern;

impl Pattern for IndexIntoPattern {
    type ParseResult = IndexInto;

    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        // FIXME: the index should just be an expression, but it does not exist yet
        type Pattern = Then<LeftSquarePattern, Then<Or<IdentPattern, NumLitPattern>, RightSquarePattern>>;

        let ok = Pattern::solve(tokens)?;

        Ok((
            ok.0,
            IndexInto {
                left_bracket: ok.1.0,
                index: ok.1.1.0,
                right_bracket: ok.1.1.1,
            }
        ))
    }
}

// defines the precedences
macro_rules! impl_operator {
    ($type:ty, $precedence:expr) => {
        impl Operator for $type {
            fn precedence(&self) -> u32 {
                $precedence
            }
        }
    };
}

impl_operator!(LogicalEqual, 1);
impl_operator!(LogicalInequal, 1);
impl_operator!(GreaterThan, 1);
impl_operator!(GreaterThanEqual, 1);
impl_operator!(LessThan, 1);
impl_operator!(LessThanEqual, 1);

impl_operator!(Plus, 2);
impl_operator!(Minus, 2);

impl_operator!(Multiply, 3);
impl_operator!(Divide, 3);
impl_operator!(Modulo, 3);

impl_operator!(LogicalNot, 4);

impl Operator for IndexInto {
    fn precedence(&self) -> u32 {
        5
    }

    fn is_left_associative(&self) -> bool {
        false
    }
}

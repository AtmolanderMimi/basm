//! Defines what is an expression is.

use crate::{lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, list::List, r#macro::Macro, operators::BinaryOperator, terminals::{CharLit, Ident, LeftParen, NumLit, RightParen, StrLit}}, source::SfSlice};

//// An expression. An expression is formed from one or more [ExpressionItem] being merged.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    node: ExpressionItem,
    children: Vec<Expression>,
}

impl LanguageItem for Expression {
    fn slice(&self) -> SfSlice {
        let source = self.node.slice().source();
        let start = self.node.slice().start();

        let end = if let Some(item) = self.children.last() {
            item.slice().end()
        } else {
            self.node.slice().end()
        };

        SfSlice::from_source(source, start..end)
            .expect("from known positions")
    }
}

impl Pattern for Expression {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        todo!()
    }
}

//// An item in an expression.
#[derive(Debug, Clone, PartialEq)]
enum ExpressionItem {
    /// This node has no meaning in an expression, this is only used for parsing
    LeftParen(LeftParen),
    /// This node has no meaning in an expression, this is only used for parsing
    RightParen(RightParen),
    Ident(Ident),
    NumLit(NumLit),
    CharLit(CharLit),
    StrLit(StrLit),
    Macro(Macro),
    List(List),
    BinaryOperator(BinaryOperator),
    // ... unary operators will go here if added
}

impl LanguageItem for ExpressionItem {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::LeftParen(t) => t.slice(),
            Self::RightParen(t) => t.slice(),
            Self::Ident(t) => t.slice(),
            Self::NumLit(t) => t.slice(),
            Self::CharLit(t) => t.slice(),
            Self::StrLit(t) => t.slice(),
            Self::List(t) => t.slice(),
            Self::Macro(t) => t.slice(),
            Self::BinaryOperator(t) => t.slice(),
        }
    }
}


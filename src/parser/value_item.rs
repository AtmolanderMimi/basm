//! Items representing a value, in themselves or in context without operations.

use crate::{lexer::token::Token, parser::{Block, Expression, LanguageItem, List, Pattern, PatternResult, pattern::*, terminals::*}, source::SfSlice};

/// Item representing a value, in themselves or in context without operations.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueItem {
    /// A group of items in parenthesises (or however it is spelled)
    ParenGroup(LeftParen, Box<Expression>, RightParen),
    Ident(Ident),
    NumLit(NumLit),
    CharLit(CharLit),
    StrLit(StrLit),
    Block(Block),
    List(List),
}

macro_rules! try_value_item_next {
    ($parsed:ident, $tokens_consumed:expr, $variant:ident) => {
        if $parsed.is_left() { return Ok(($tokens_consumed, ValueItem::$variant($parsed.unwrap_left()))); }
        let $parsed = $parsed.unwrap_right();
    };
}

impl Pattern for ValueItem {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        type ParenGroupPattern = Then<LeftParen, Then<Expression, RightParen>>;
        type Pattern = Or<
        ParenGroupPattern, Or<
        Ident, Or<
        NumLit, Or<
        CharLit, Or<
        StrLit, Or<
        Block,
        Then<List, Not<ThickArrow>>
        >>>>>>;

        let (token_consumed, item) = Pattern::solve(tokens)?;
        if item.is_left() {
            let paren = item.unwrap_left();
            return Ok((token_consumed, ValueItem::ParenGroup(paren.0, Box::new(paren.1.0), paren.1.1)));
        }
        let item = item.unwrap_right();
        try_value_item_next!(item, token_consumed, Ident);
        try_value_item_next!(item, token_consumed, NumLit);
        try_value_item_next!(item, token_consumed, CharLit);
        try_value_item_next!(item, token_consumed, StrLit);
        try_value_item_next!(item, token_consumed, Block);

        Ok((token_consumed, ValueItem::List(item.0)))
    }

    fn name() -> String {
        "value item".to_string()
    }
}

impl LanguageItem for ValueItem {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::ParenGroup(left, _, right) => {
                let source = left.slice().source();
                let start = left.slice().start();
                let end = right.slice().end();

                SfSlice::from_source(source, start..end)
                    .expect("from known items")
            },
            Self::Ident(t) => t.slice(),
            Self::NumLit(t) => t.slice(),
            Self::CharLit(t) => t.slice(),
            Self::StrLit(t) => t.slice(),
            Self::List(t) => t.slice(),
            Self::Block(t) => t.slice(),
        }
    }
}

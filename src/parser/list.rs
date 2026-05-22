//! Defines a list.

use crate::{impl_language_item, lexer::token::Token, parser::{Pattern, PatternResult, expression::Expression, pattern::{SeperatedMany, Then}, terminals::{Comma, LeftSquare, RightSquare}}};

/// A list literal. It is expressions seperated by commas in square brackets.
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub opening_bracket: LeftSquare,
    pub items: Vec<(Option<Comma>, Expression)>,
    pub closing_bracket: RightSquare,
}

impl_language_item!(List, opening_bracket, closing_bracket);

impl Pattern for List {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            LeftSquare,
            Then<
                SeperatedMany<Expression, Comma>,
                RightSquare
            >
        >::solve(&tokens)?;

        let list = List {
            opening_bracket: res.1.0,
            items: res.1.1.0,
            closing_bracket: res.1.1.1,
        };

        return Ok((res.0, list));
    }

    fn name() -> String { "listlit".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lexer::lex_string;

    #[test]
    fn list_parses_when_empty() {
        let tokens = lex_string("[]").unwrap();
        let (_, list) = List::solve(&tokens).unwrap();

        assert_eq!(list.items.len(), 0);
    }

    #[test]
    fn list_parses_one_item_int() {
        let tokens = lex_string("[3]").unwrap();
        let (_, list) = List::solve(&tokens).unwrap();

        assert_eq!(list.items.len(), 1);
    }

    #[test]
    fn list_parses_one_item_list() {
        let tokens = lex_string("[[]]").unwrap();
        let (_, list) = List::solve(&tokens).unwrap();

        assert_eq!(list.items.len(), 1);
    }

    #[test]
    fn list_parses_multiple_items() {
        let tokens = lex_string("[3, ['c'], \"732\"]").unwrap();
        let (_, list) = List::solve(&tokens).unwrap();

        assert_eq!(list.items.len(), 3);
    }

    #[test]
    fn list_does_not_parse_trailing_comma() {
        let tokens = lex_string("[3, ['c'], \"732\",]").unwrap();
        List::solve(&tokens).unwrap_err();
    }

    #[test]
    fn list_does_not_parse_empty_comma() {
        let tokens = lex_string("[,]").unwrap();
        List::solve(&tokens).unwrap_err();
    }

    #[test]
    fn list_does_not_parse_invalid_values() {
        let tokens = lex_string("[@]").unwrap();
        dbg!(List::solve(&tokens)).unwrap_err();
    }
}

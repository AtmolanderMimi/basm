//! Defines what is an expression is.

use either::Either;

use crate::{lexer::token::Token, parser::{LanguageItem, ParseError, ParseErrorVariant, Pattern, PatternResult, list::List, r#macro::Macro, operators::{BinaryOperator, NB_PRECEDENCE_LEVELS, Operator, UnaryOperator}, pattern::{Not, OneOrMore, Or, Then}, terminals::{CharLit, Ident, LeftParen, NumLit, RightParen, StrLit, ThickArrow}}, source::SfSlice};

//// An expression. An expression is formed from one or more [ExpressionItem] being merged.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub node: ExpressionItem,
    pub children: Vec<Expression>,
}

impl Expression {
    /// Creates a new expression with just the node.
    fn new_from_node(node: ExpressionItem) -> Self {
        Expression {
            node,
            children: Vec::new(),
        }
    }

    /// Creates a new expression from [ExpressionItem]'s.
    /// 
    /// # Panic
    /// Should always have at least 1 item in `items`.
    pub fn new_from_items(tokens_consumed: usize, items: &[ExpressionItem]) -> Result<Self, ParseError> {
        if items.is_empty() {
            panic!("should never be called empty")
        }

        let mut sub_exprs = items.iter()
            .map(|i| Expression::new_from_node(i.clone()))
            .collect::<Vec<_>>();

        for current_precedence in (0..NB_PRECEDENCE_LEVELS).rev() {
            // Links sub_exprs in a tree fashion for all the operators in the precedence level
            loop {
                // -- finds an operator to link
                let predicate = |(_, item): &(_, &Expression)| {
                    let Some(precedence) = item.node.precedence() else {
                        return false;
                    };
                    let has_correct_precedence = precedence == current_precedence;

                    let has_not_been_linked = item.is_leaf();

                    has_correct_precedence && has_not_been_linked
                };

                let maybe_operator_index = sub_exprs.iter()
                    .enumerate()
                    .find(predicate).map(|(i, _)| i);

                let Some(operator_index) = maybe_operator_index else {
                    // there are no more unmatched operators in the precedence level
                    break;
                };

                // -- link the operator
                match &sub_exprs[operator_index].node {
                    ExpressionItem::BinaryOperator(_) => {
                        // TODO: usize::MAX hack
                        let Some(item_after) = sub_exprs.try_remove(operator_index + 1) else {
                            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
                        };
                        let Some(item_before) = sub_exprs.try_remove(operator_index.checked_sub(1).unwrap_or(usize::MAX)) else {
                            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
                        };

                        let operator = &mut sub_exprs[operator_index-1];
                        operator.children = vec![item_before, item_after];
                    },
                    ExpressionItem::UnaryOperator(op) if op.is_right_associative() => {
                        let Some(item_after) = sub_exprs.try_remove(operator_index + 1) else {
                            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
                        };

                        let operator = &mut sub_exprs[operator_index];
                        operator.children = vec![item_after];
                    },
                    ExpressionItem::UnaryOperator(op) if !op.is_right_associative() => {
                        // TODO: usize::MAX hack
                        let Some(item_before) = sub_exprs.try_remove(operator_index.checked_sub(1).unwrap_or(usize::MAX)) else {
                            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
                        };

                        let operator = &mut sub_exprs[operator_index-1];
                        operator.children = vec![item_before];
                    },
                    _ => unimplemented!("an item with precedence that is not a binary operator"),
                }
            }
        }

        if sub_exprs.len() > 1 {
            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
        }
        Ok(sub_exprs.pop().expect("there should be one and only one item left"))
    }

    /// Does not have children
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl LanguageItem for Expression {
    fn slice(&self) -> SfSlice {
        let source = self.node.slice().source();

        let start = if let Some(item) = self.children.first() {
            // the min in case the operator is a prefix operator
            item.slice().start().min(self.node.slice().start())
        } else {
            self.node.slice().start()
        };

        let end = if let Some(item) = self.children.last() {
            // the max in case the operator is a postfix operator
            item.slice().end().max(self.node.slice().end())
        } else {
            self.node.slice().end()
        };

        SfSlice::from_source(source, start..end)
            .expect("from known positions")
    }
}

impl Pattern for Expression {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (consumed_tokens, items) = OneOrMore::<ExpressionItem>::solve(&tokens)?;

        Ok((consumed_tokens, Expression::new_from_items(consumed_tokens, &items)?))
    }

    fn name() -> String { "expr".to_string() }
}

//// An item in an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionItem {
    /// A group of items in parenthesises (or however it is spelled)
    ParenGroup(LeftParen, Box<Expression>, RightParen),
    Ident(Ident),
    NumLit(NumLit),
    CharLit(CharLit),
    StrLit(StrLit),
    Macro(Macro),
    List(List),
    BinaryOperator(BinaryOperator),
    UnaryOperator(UnaryOperator),
}

impl ExpressionItem {
    /// Returns the precedence of the item, if it has any
    pub fn precedence(&self) -> Option<u32> {
        match self {
            Self::BinaryOperator(op) => Some(op.precedence()),
            Self::UnaryOperator(op) => Some(op.precedence()),
            _ => None,
        }
    }
}

impl LanguageItem for ExpressionItem {
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
            Self::Macro(t) => t.slice(),
            Self::BinaryOperator(t) => t.slice(),
            Self::UnaryOperator(t) => t.slice(),
        }
    }
}

macro_rules! try_expression_item_next {
    ($parsed:ident, $tokens_consumed:expr, $variant:ident) => {
        if $parsed.is_left() { return Ok(($tokens_consumed, ExpressionItem::$variant($parsed.unwrap_left()))); }
        let $parsed = $parsed.unwrap_right();
    };
}

impl Pattern for ExpressionItem {
    // The order of these if statements has an impact on which item has priority
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        type ParenGroupPattern = Then<LeftParen, Then<Expression, RightParen>>;
        type ExpressionItemPattern = 
        Or<
        ParenGroupPattern, Or<
        Ident, Or<
        NumLit, Or<
        CharLit, Or<
        StrLit, Or<
        Macro, Or<
        Then<List, Not<ThickArrow>>,
        Or<BinaryOperator,
        UnaryOperator,
        >>>>>>>>;

        let res = ExpressionItemPattern::solve(tokens);
        let res = if let Err(ParseError { tokens_before_error, variant: ParseErrorVariant::UnexpectedTokenError{ got, ..} }) = res {
            return Err(ParseError::new_unexpected_token(tokens_before_error, Self::name(), got));
        } else {
            res?
        };

        let parsed = res.1;

        if parsed.is_left() { let p = parsed.unwrap_left(); return Ok((res.0, ExpressionItem::ParenGroup(p.0, Box::new(p.1.0), p.1.1))); }
        let parsed = parsed.unwrap_right();

        try_expression_item_next!(parsed, res.0, Ident);
        try_expression_item_next!(parsed, res.0, NumLit);
        try_expression_item_next!(parsed, res.0, CharLit);
        try_expression_item_next!(parsed, res.0, StrLit);
        try_expression_item_next!(parsed, res.0, Macro);
        if let Either::Left((list, _)) = parsed {
            return Ok((res.0, ExpressionItem::List(list)));
        }
        let parsed = parsed.unwrap_right();

        try_expression_item_next!(parsed, res.0, BinaryOperator);

        return Ok((res.0, ExpressionItem::UnaryOperator(parsed)));
    }

    fn name() -> String { "expression item".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::{lex_string, token::TokenType}, parser::Pattern};

    use std::assert_matches;

    #[test]
    fn expression_does_not_parse_nothing() {
        Expression::solve(&[]).unwrap_err();
    }

    #[test]
    fn expression_parses_single_item() {
        let tokens = lex_string("1").unwrap();

        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert!(expression.is_leaf());
    }

    #[test]
    fn expression_forms_operation() {
        let tokens = lex_string("1 + 3").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Plus(_))
        );
        assert_eq!(expression.children.len(), 2);
    }

    #[test]
    fn expression_cannot_parse_when_too_few_items() {
        let tokens = lex_string("1 + 3 + *").unwrap();
        dbg!(Expression::solve(&tokens)).unwrap_err();
    }

    #[test]
    fn expression_have_proper_expression_order() {
        let tokens = lex_string("1 + 2 * 3").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        // the addition (at the top)
        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Plus(_))
        );

        // the left argument of the multiplication
        assert_matches!(
            expression.children[0].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(1), .. })),
        );

        // the multiplication
        assert_matches!(
            expression.children[1].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Multiply(_))
        );

        // .. and it's arguments
        assert_matches!(
            expression.children[1].children[0].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(2), .. })),
        );

        assert_matches!(
            expression.children[1].children[1].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(3), .. })),
        );
    }

    #[test]
    fn expression_are_left_to_right_when_needed() {
        let tokens = lex_string("1 * 2 / 3").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        // the division (at the top)
        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Divide(_))
        );

        // the right argument of the division
        assert_matches!(
            expression.children[1].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(3), .. })),
        );

        // the multiplication
        assert_matches!(
            expression.children[0].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Multiply(_))
        );

        // .. and it's arguments
        assert_matches!(
            expression.children[0].children[0].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(1), .. })),
        );

        assert_matches!(
            expression.children[0].children[1].node,
            ExpressionItem::NumLit(NumLit(Token { t_type: TokenType::NumLit(2), .. })),
        );
    }

    #[test]
    fn expression_list_and_property() {
        let tokens = lex_string("list @ [1,2,3].len").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        // the division (at the top)
        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Index(_))
        );

        // the right argument of the division
        assert_matches!(
            expression.children[0].node,
            ExpressionItem::Ident(_),
        );

        // the multiplication
        assert_matches!(
            expression.children[1].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Property(_))
        );

        // .. and it's arguments
        assert_matches!(
            expression.children[1].children[0].node,
            ExpressionItem::List(_),
        );

        assert_matches!(
            expression.children[1].children[1].node,
            ExpressionItem::Ident(_),
        );
    }

    #[test]
    fn expression_parentheses_group_parses() {
        let tokens = lex_string("(([] + var) * 1)").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        let ExpressionItem::ParenGroup(_, inner_expression,_ ) = expression.node else {
            panic!()
        };

        assert_matches!(
            inner_expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Multiply(_))
        );

        assert_matches!(
            inner_expression.children[0].node,
            ExpressionItem::ParenGroup(..)
        );
        
        assert_matches!(
            inner_expression.children[1].node,
            ExpressionItem::NumLit(_),
        );
    }

    #[test]
    fn expression_empty_brackets_does_not_crash() {
        let tokens = lex_string("() + 3").unwrap();

        Expression::solve(&tokens).unwrap_err();
    }

    #[test]
    fn expression_item_no_confusion_between_argumented_macro_and_list() {
        let tokens = lex_string("[arg1, arg2] => {}").unwrap();

        let (tokens_consumed, item) = ExpressionItem::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            item,
            ExpressionItem::Macro(_),
        )
    }

    #[test]
    fn expression_unary_operator_parses() {
        let tokens = lex_string("!my_var").unwrap();

        let (_, item) = Expression::solve(&tokens).unwrap();
        assert_matches!(
            item.node,
            ExpressionItem::UnaryOperator(UnaryOperator::LogicalNot(_)),
        );
        assert_matches!(
            item.children[0].node,
            ExpressionItem::Ident(_),
        );
    }

    #[test]
    fn expression_unary_operator_respects_precedence() {
        let tokens = lex_string("1 && !my_var@[1,2]").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::LogicalAnd(_))
        );

        assert_matches!(
            expression.children[0].node,
            ExpressionItem::NumLit(_)
        );
        
        assert_matches!(
            expression.children[1].node,
            ExpressionItem::UnaryOperator(UnaryOperator::LogicalNot(_)),
        );

        assert_matches!(
            expression.children[1].children[0].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Index(_)),
        );
    }
}

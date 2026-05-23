//! Defines what is an expression is.

use crate::{lexer::token::Token, parser::{LanguageItem, ParseError, Pattern, PatternResult, list::List, r#macro::Macro, operators::{self, BinaryOperator, NB_PRECEDENCE_LEVELS, Operator}, pattern::{OneOrMore, Then}, terminals::{CharLit, Ident, LeftParen, NumLit, RightParen, StrLit}}, source::SfSlice};

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
    pub fn new_from_items(items: &[ExpressionItem]) -> Result<Self, ParseError> {
        if items.is_empty() {
            panic!("should never be called empty")
        }

        let mut sub_exprs = items.iter()
            .map(|i| Expression::new_from_node(i.clone()))
            .collect::<Vec<_>>();

        for current_precedence in (0..NB_PRECEDENCE_LEVELS).rev() {
            let is_right_associative = operators::RIGHT_ASSOCIATIVE_LEVELS.contains(&current_precedence);

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

                let maybe_operator_index = if is_right_associative {
                    sub_exprs.iter().enumerate().rfind(predicate).map(|(i, _)| i)
                } else {
                    sub_exprs.iter().enumerate().find(predicate).map(|(i, _)| i)
                };

                let Some(operator_index) = maybe_operator_index else {
                    // there are no more unmatched operators in the precedence level
                    break;
                };

                // -- link the operator
                match sub_exprs[operator_index].node {
                    ExpressionItem::BinaryOperator(_) => {
                        // TODO: usize::MAX hack
                        let Some(item_after) = sub_exprs.try_remove(operator_index.checked_add(1).unwrap_or(usize::MAX)) else {
                            return Err(ParseError::new_unparsed_expression(items.to_vec()));
                        };
                        let Some(item_before) = sub_exprs.try_remove(operator_index.checked_sub(1).unwrap_or(usize::MAX)) else {
                            return Err(ParseError::new_unparsed_expression(items.to_vec()));
                        };

                        let operator = &mut sub_exprs[operator_index-1];
                        operator.children = vec![item_before, item_after];
                    },
                    _ => unimplemented!("an item with precedence that is not a binary operator"),
                }
            }
        }

        if sub_exprs.len() > 1 {
            return Err(ParseError::new_unparsed_expression(items.to_vec()));
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

        Ok((consumed_tokens, Expression::new_from_items(&items)?))
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
    // ... unary operators will go here if added
}

impl ExpressionItem {
    /// Returns the precedence of the item, if it has any
    pub fn precedence(&self) -> Option<u32> {
        if let Self::BinaryOperator(op) = self {
            Some(op.precedence())
        } else {
            None
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
        }
    }
}

impl Pattern for ExpressionItem {
    // TODO: same thing as the solver for binary expressions,
    // to replace

    // The order of these if statements has an impact on which item has priority
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        type ParenGroupPattern = Then<LeftParen, Then<Expression, RightParen>>;

        let (nb_tokens, item) = if let Ok((nb_tokens, parsed)) = ParenGroupPattern::solve(&tokens) {
            let paren_group = ExpressionItem::ParenGroup(parsed.0, Box::new(parsed.1.0), parsed.1.1);

            (nb_tokens, paren_group)
        }
        else if let Ok((nb_tokens, parsed)) = Ident::solve(tokens) {
            (nb_tokens, ExpressionItem::Ident(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = NumLit::solve(tokens) {
            (nb_tokens, ExpressionItem::NumLit(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = CharLit::solve(tokens) {
            (nb_tokens, ExpressionItem::CharLit(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = StrLit::solve(tokens) {
            (nb_tokens, ExpressionItem::StrLit(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = Macro::solve(tokens) {
            (nb_tokens, ExpressionItem::Macro(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = List::solve(tokens) {
            (nb_tokens, ExpressionItem::List(parsed))
        }
        else if let Ok((nb_tokens, parsed)) = BinaryOperator::solve(tokens) {
            (nb_tokens, ExpressionItem::BinaryOperator(parsed))
        } else if let Some(token) = tokens.first() {
            return Err(ParseError::new_unexpected_token(Self::name(), token.clone()))
        } else {
            return Err(ParseError::new_no_more_tokens(Self::name()))
        };

        Ok((nb_tokens, item))
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
    fn expression_are_right_to_left_when_needed() {
        let tokens = lex_string("list_var @ [1,2,3] @ 2").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Index(_))
        );

        assert_matches!(
            expression.children[0].node,
            ExpressionItem::Ident(_),
        );

        assert_matches!(
            expression.children[1].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Index(_))
        );

        assert_matches!(
            expression.children[1].children[0].node,
            ExpressionItem::List(_),
        );

        assert_matches!(
            expression.children[1].children[1].node,
            ExpressionItem::NumLit(_),
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
}

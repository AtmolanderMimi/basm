//! Defines what is an expression is.

use std::collections::hash_set::Iter;

use crate::{lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, UnexpectedTokenError, list::List, r#macro::Macro, operators::{self, BinaryOperator, Operator}, pattern::{Many, Then}, terminals::{CharLit, Ident, LeftParen, NumLit, RightParen, StrLit}}, source::SfSlice};

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
    pub fn new_from_items(items: &[ExpressionItem]) -> Result<Self, UnexpectedTokenError> {
        if items.len() < 1 {
            return Err(UnexpectedTokenError::new_got_nothing(Vec::new())) // TODO: once again, redo error type
        }

        let mut items = items.iter()
            .map(|i| Expression::new_from_node(i.clone()))
            .collect::<Vec<_>>();

        for precedence_level in operators::NB_PRECEDENCE_LEVELS..0 {
            let is_right_associative = operators::RIGHT_ASSOCIATIVE_LEVELS.contains(&precedence_level);

            // Links items in a tree fashion for all the operators in the precedence level
            loop {
                // -- finds an operator to link
                let predicate = |(_, item): &(_, &Expression)| {
                    let has_correct_precedence = item.node.precedence().unwrap_or(0) == precedence_level;
                    let has_not_been_linked = item.is_leaf();

                    has_correct_precedence && has_not_been_linked
                };

                let maybe_operator_index = if is_right_associative {
                    items.iter().enumerate().rfind(predicate).map(|(i, _)| i)
                } else {
                    items.iter().enumerate().find(predicate).map(|(i, _)| i)
                };

                let Some(operator_index) = maybe_operator_index else {
                    // there are no more unmatched operators in the precedence level
                    break;
                };

                // -- link the operator
                match items[operator_index].node {
                    ExpressionItem::BinaryOperator(_) => {
                        // TODO: better error handling
                        let Some(item_after) = items.try_remove(operator_index+1) else {
                            return Err(UnexpectedTokenError::new_got_nothing(Vec::new()));
                        };
                        let Some(item_before) = items.try_remove(operator_index-1) else {
                            return Err(UnexpectedTokenError::new_got_nothing(Vec::new()));
                        };

                        let operator = &mut items[operator_index-1];
                        operator.children = vec![item_before, item_after];
                    },
                    _ => unimplemented!("an item with precedence that is not a binary operator"),
                }
            }
        }

        if items.len() > 1 {
            // TODO: better error handling
            return Err(UnexpectedTokenError::new_got_nothing(Vec::new()));
        }
        Ok(items.pop().expect("there should be one and only one item left"))
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
        let (consumed_tokens, items) = Many::<ExpressionItem>::solve(&tokens)?;

        Ok((consumed_tokens, Expression::new_from_items(&items)?))
    }
}

//// An item in an expression.
#[derive(Debug, Clone, PartialEq)]
enum ExpressionItem {
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
        } else {
            // TODO: redo error type
            return Err(UnexpectedTokenError::new_got_nothing(Vec::new()))
        };

        Ok((nb_tokens, item))
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex_string, parser::{Pattern, expression::Expression}};

    #[test]
    fn expression_does_not_parse_nothing() {
        Expression::solve(&[]).unwrap_err();
    }

    #[test]
    fn expression_parses_single_item() {
        let tokens = lex_string("1 ").unwrap();

        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, 1);
        assert!(expression.is_leaf());
    }
}

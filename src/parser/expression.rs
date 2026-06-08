//! Defines what is an expression is.

use either::Either;

use crate::{lexer::token::Token, parser::{LanguageItem, LeftAssociativeUnaryOperator, ParseError, Pattern, PatternResult, REVERSE_PRECEDENCE_LEVELS, ValueItem, operators::{BinaryOperator, NB_PRECEDENCE_LEVELS, Operator, RightAssociativeUnaryOperator}, pattern::Or}, source::SfSlice};

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

                let maybe_operator_index = if REVERSE_PRECEDENCE_LEVELS.contains(&current_precedence) {
                    sub_exprs.iter()
                        .enumerate()
                        .rev()
                        .find(predicate).map(|(i, _)| i)
                } else {
                    sub_exprs.iter()
                        .enumerate()
                        .find(predicate).map(|(i, _)| i)
                };

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
                    ExpressionItem::RightAssociativeUnaryOperator(_) => {
                        let Some(item_after) = sub_exprs.try_remove(operator_index + 1) else {
                            return Err(ParseError::new_unparsed_expression(tokens_consumed, items.to_vec()));
                        };

                        let operator = &mut sub_exprs[operator_index];
                        operator.children = vec![item_after];
                    },
                    ExpressionItem::LeftAssociativeUnaryOperator(_) => {
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
        // these two patterns switch over every time a value item is found or a binary operator is found
        // For example:
        // `!value[32] + 8`
        // Start with `RightAssociativeOrValue`.
        // -> `!` RightAssotive
        // -> `value` Value
        // we found a value, so we switch to `LeftAssociativeOrBinary`
        // -> `[32]` LeftAssociative 
        // -> `+` BinaryOperator
        // we found an operator, so we switch to `RightAssociativeOrValue`
        // This method allows us to prune the parsing of expression items which would be impossible,
        // in turn allowing otherwise ambiguous syntax like `[..]` for index which is the same a list literal.
        type RightAssociativeOrValue = Or<RightAssociativeUnaryOperator, ValueItem>;
        type LeftAssociativeOrBinary = Or<LeftAssociativeUnaryOperator, BinaryOperator>;

        let mut items = Vec::new();
        let mut tokens_consumed = 0;
        let mut parsing_right_associative_or_value = true;
        loop {
            let (new_tokens_consumed, new_item) = if parsing_right_associative_or_value {
                // -- right-associative or value
                let (new_tokens_consumed, parsed) = RightAssociativeOrValue::solve(&tokens[tokens_consumed..])?;

                let item = match parsed {
                    Either::Left(op) => ExpressionItem::RightAssociativeUnaryOperator(op),
                    Either::Right(val) => {
                        parsing_right_associative_or_value = false;
                        ExpressionItem::ValueItem(val)
                    },
                };

                (new_tokens_consumed, item)
            } else {
                // -- left-associative or binary operator
                let Ok((new_tokens_consumed, parsed)) = LeftAssociativeOrBinary::solve(&tokens[tokens_consumed..]) else {
                    // this is okay to fail, it just means that we are done
                    break;
                };

                let item = match parsed {
                    Either::Left(op) => ExpressionItem::LeftAssociativeUnaryOperator(op),
                    Either::Right(val) => {
                        parsing_right_associative_or_value = true;
                        ExpressionItem::BinaryOperator(val)
                    },
                };

                (new_tokens_consumed, item)
            };

            tokens_consumed += new_tokens_consumed;
            items.push(new_item);
        }

        Ok((tokens_consumed, Expression::new_from_items(tokens_consumed, &items)?))
    }

    fn name() -> String { "expr".to_string() }
}

//// An item in an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionItem {
    ValueItem(ValueItem),
    BinaryOperator(BinaryOperator),
    LeftAssociativeUnaryOperator(LeftAssociativeUnaryOperator),
    RightAssociativeUnaryOperator(RightAssociativeUnaryOperator),
}

impl ExpressionItem {
    /// Returns the precedence of the item, if it has any
    pub fn precedence(&self) -> Option<u32> {
        match self {
            Self::BinaryOperator(op) => Some(op.precedence()),
            Self::LeftAssociativeUnaryOperator(op) => Some(op.precedence()),
            Self::RightAssociativeUnaryOperator(op) => Some(op.precedence()),
            _ => None,
        }
    }
}

impl LanguageItem for ExpressionItem {
    fn slice(&self) -> crate::source::SfSlice {
        match self {
            Self::ValueItem(t) => t.slice(), 
            Self::BinaryOperator(t) => t.slice(),
            Self::LeftAssociativeUnaryOperator(t) => t.slice(),
            Self::RightAssociativeUnaryOperator(t) => t.slice(),
        }
    }
}

// ExpressionItem does not have a pattern definition, as its parsing is dependent on context.
// ExpressionItem's are parsed in the Expression Pattern implementation.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::lex_string, parser::Pattern};

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
    fn expression_are_left_to_right_when_needed() {
        let tokens = lex_string("\"hi\" * ident / 3").unwrap();
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
            ExpressionItem::ValueItem(ValueItem::NumLit(_)),
        );

        // the multiplication
        assert_matches!(
            expression.children[0].node,
            ExpressionItem::BinaryOperator(BinaryOperator::Multiply(_))
        );

        // .. and it's arguments
        assert_matches!(
            expression.children[0].children[0].node,
            ExpressionItem::ValueItem(ValueItem::StrLit(_)),
        );

        assert_matches!(
            expression.children[0].children[1].node,
            ExpressionItem::ValueItem(ValueItem::Ident(_)),
        );
    }

    #[test]
    fn expression_list_and_property() {
        let tokens = lex_string("list[[1,2,3].len]").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        // the division (at the top)
        let ExpressionItem::LeftAssociativeUnaryOperator(LeftAssociativeUnaryOperator::Index(_, inner_expr, _)) = expression.node else {
            panic!()
        };

        // the right argument of the division
        assert_matches!(
            expression.children[0].node,
            ExpressionItem::ValueItem(ValueItem::Ident(_)),
        );

        // the multiplication
        assert_matches!(
            inner_expr.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Property(_))
        );

        // .. and it's arguments
        assert_matches!(
            inner_expr.children[0].node,
            ExpressionItem::ValueItem(ValueItem::List(_)),
        );

        assert_matches!(
            inner_expr.children[1].node,
            ExpressionItem::ValueItem(ValueItem::Ident(_)),
        );
    }

    #[test]
    fn expression_parentheses_group_parses() {
        let tokens = lex_string("(([] + var) * 1)").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        let ExpressionItem::ValueItem(ValueItem::ParenGroup(_, inner_expression,_ )) = expression.node else {
            panic!()
        };

        assert_matches!(
            inner_expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::Multiply(_))
        );

        assert_matches!(
            inner_expression.children[0].node,
            ExpressionItem::ValueItem(ValueItem::ParenGroup(..))
        );
        
        assert_matches!(
            inner_expression.children[1].node,
            ExpressionItem::ValueItem(ValueItem::NumLit(_)),
        );
    }

    #[test]
    fn expression_empty_brackets_does_not_crash() {
        let tokens = lex_string("() + 3").unwrap();

        Expression::solve(&tokens).unwrap_err();
    }

    #[test]
    fn expression_no_confusion_between_argumented_block_and_list() {
        let tokens = lex_string("[arg1, arg2] => {}").unwrap();

        let (tokens_consumed, item) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            item.node,
            ExpressionItem::ValueItem(ValueItem::Block(_)),
        )
    }

    #[test]
    fn expression_unary_operator_parses() {
        let tokens = lex_string("!my_var").unwrap();

        let (_, item) = Expression::solve(&tokens).unwrap();
        assert_matches!(
            item.node,
            ExpressionItem::RightAssociativeUnaryOperator(RightAssociativeUnaryOperator::LogicalNot(_)),
        );
        assert_matches!(
            item.children[0].node,
            ExpressionItem::ValueItem(ValueItem::Ident(_)),
        );
    }

    #[test]
    fn expression_unary_operator_respects_precedence() {
        let tokens = lex_string("1 && !my_var[[1,2]]").unwrap();
        let (tokens_consumed, expression) = Expression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);

        assert_matches!(
            expression.node,
            ExpressionItem::BinaryOperator(BinaryOperator::LogicalAnd(_))
        );

        assert_matches!(
            expression.children[0].node,
            ExpressionItem::ValueItem(ValueItem::NumLit(_))
        );
        
        assert_matches!(
            expression.children[1].node,
            ExpressionItem::RightAssociativeUnaryOperator(RightAssociativeUnaryOperator::LogicalNot(_)),
        );

        assert_matches!(
            expression.children[1].children[0].node,
            ExpressionItem::LeftAssociativeUnaryOperator(LeftAssociativeUnaryOperator::Index(..)),
        );
    }
}

//! Defines how to get a value or emplacement out of an expression

use std::mem::transmute;

use either::Either;
use thiserror::Error;

use crate::{compiler::{block::Block, operators::{BinaryOperator, LeftAssociativeUnaryOperator, OperationError, RightAssociativeUnaryOperator}, scope::Scope, string_normalizer::{self, InvalidEscapeSequence}, value::Value}, newtype_wrapper, parser::{CharLit, Ident, LanguageItem, StrLit}};

use crate::parser::{Expression as ParsedExpression, ExpressionItem as ParsedExpressionItem, ValueItem as ParsedValueItem};
newtype_wrapper!(Expression, ParsedExpression);
newtype_wrapper!(ExpressionItem, ParsedExpressionItem);
newtype_wrapper!(ValueItem, ParsedValueItem);

/// An error occuring during the evaluation of an expression
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ExpressionEvaluationError {
    #[error("variable {} does not exist in the current scope", ident.slice_str())]
    VariableDoesNotExist {
        ident: Ident
    },
    #[error("{inner}")]
    OperationError {
        inner: OperationError,
        expression: Expression,
    },

    #[error("{inner}")]
    InvalidEscapeSequence {
        inner: InvalidEscapeSequence,
        lit: Either<StrLit, CharLit>,
    },
}

impl Expression {
    fn node(&self) -> &ExpressionItem {
        (&self.0.node).into()
    }

    fn children(&self) -> &[Expression] {
        // compiler::Expression is transparent to parser::Expression
        unsafe { transmute::<&[ParsedExpression], &[Expression]>(&self.0.children) } 
    }

    pub fn evaluate(&self, ctx: &Scope) -> Result<Value, ExpressionEvaluationError> {
        let value = match &self.0.node {
            ParsedExpressionItem::ValueItem(item) => <&ValueItem>::from(item).value(ctx)?,
            ParsedExpressionItem::BinaryOperator(op) => {
                let op = <&BinaryOperator>::from(op);

                let children = self.children();
                // once again, the pattern matching should guarenty that a binary operation has two children
                let input_values = (children[0].evaluate(ctx)?, children[1].evaluate(ctx)?);

                let value = op.evaluate(input_values.0, input_values.1)
                    .map_err(|err| ExpressionEvaluationError::OperationError { inner: err, expression: self.clone() })?;

                value
            },
            ParsedExpressionItem::RightAssociativeUnaryOperator(op) => {
                let op = <&RightAssociativeUnaryOperator>::from(op);

                let children = self.children();
                // once again, the pattern matching should guarenty that a unary operation has one child
                let input_value = children[0].evaluate(ctx)?;

                let value = op.evaluate(input_value)
                    .map_err(|err| ExpressionEvaluationError::OperationError { inner: err, expression: self.clone() })?;

                value
            },
            ParsedExpressionItem::LeftAssociativeUnaryOperator(op) => {
                let op = <&LeftAssociativeUnaryOperator>::from(op);

                let children = self.children();
                // once again, the pattern matching should guarenty that a unary operation has one child
                let input_value = children[0].evaluate(ctx)?;

                let value = op.evaluate(ctx, input_value)
                    .map_err(|err| ExpressionEvaluationError::OperationError { inner: err, expression: self.clone() })?;

                value
            },
        };

        Ok(value)
    }
}

impl ValueItem {
    /// Return the inherent value of the value item.
    /// This method will call Expression::evaluate if it has to evaluate Expressions
    /// (e.g: lists and parenthesis group).
    /// May fail if a variable is not defined in the scope.
    /// (e.g: NumLit() -> Value::Number, StrLit -> Value::List)
    pub fn value(&self, ctx: &Scope) -> Result<Value, ExpressionEvaluationError> {
        let maybe_value = match &self.0 {
            ParsedValueItem::ParenGroup(_, e, _) => {
                let expression = <&Expression>::from(e.as_ref());
                
                expression.evaluate(ctx)?
            },
            ParsedValueItem::Ident(ident) => {
                let Some(value) = ctx.get(ident.slice_str()) else {
                    return Err(ExpressionEvaluationError::VariableDoesNotExist { ident: ident.clone() })
                };

                value
            },
            ParsedValueItem::NumLit(num) => {
                let num = num.slice_str().parse()
                    .expect("the lexer should guarenty that NumLit's are valid i32");

                Value::Number(num)
            },
            ParsedValueItem::CharLit(charlit) => {
                let string_token_slice = charlit.slice_str();
                let inner_str = &string_token_slice[1..string_token_slice.len()-1];

                let normalized = string_normalizer::normalize_string_formatting(inner_str)
                    .map_err(|err| ExpressionEvaluationError::InvalidEscapeSequence {
                        inner: err,
                        lit: Either::Right(charlit.clone()),
                    })?;

                let first_char = normalized.chars().next()
                    .expect("chars should have at least one character");

                Value::Number(first_char as i32)
            },
            ParsedValueItem::StrLit(strlit) => {
                let string_token_slice = strlit.slice_str();
                let inner_str = &string_token_slice[1..string_token_slice.len()-1];
                let normalized = string_normalizer::normalize_string_formatting(inner_str)
                    .map_err(|err| ExpressionEvaluationError::InvalidEscapeSequence {
                        inner: err,
                        lit: Either::Left(strlit.clone()),
                    })?;

                Value::new_from_str(&normalized)
            },
            ParsedValueItem::Block(block) => {
                Value::Block(Box::new(Block::from(block.clone())))
            },
            ParsedValueItem::List(list) => {
                let mut values = Vec::new();

                for (_, item) in &list.items {
                    let value = <&Expression>::from(item).evaluate(ctx)?;
                    values.push(value);
                }

                Value::List(values)
            },
        };

        Ok(maybe_value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{compiler::operators::OperationError, parser::Pattern};

    use super::*;

    use std::assert_matches;

    fn expression_from_str(source: &str) -> Expression {
        let parsed = ParsedExpression::solve_str(source).unwrap();
        parsed.into()
    }

    fn value_item_from_str(source: &str) -> ValueItem {
        let parsed = ParsedValueItem::solve_str(source).unwrap();
        parsed.into()
    }

    #[test]
    fn expression_basic_arithmetic() {
        let expr = expression_from_str("9 + 10 - 21");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(-2));
    }

    #[test]
    fn expression_precedence_is_respected() {
        let expr = expression_from_str("(1 + 2) + 3 * 4");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(15));
    }

    #[test]
    fn expression_operations_on_lists() {
        let expr = expression_from_str("([1,2] + [3,4])[[0,3]]");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::List(vec![Value::Number(1), Value::Number(4)]));
    }

    #[test]
    fn expression_unary_operators() {
        let expr = expression_from_str("!(5 == 4 || 0 == 0)");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::FALSE);
    }

    #[test]
    fn expression_error_when_wrong_types_unary() {
        let expr = expression_from_str("![]");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_matches!(value.unwrap_err(), ExpressionEvaluationError::OperationError { inner: OperationError::InvalidTypeUnary { .. }, .. });
    }

    #[test]
    fn expression_error_when_wrong_types_binary() {
        let expr = expression_from_str("73[[1,2,3]]");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_matches!(value.unwrap_err(), ExpressionEvaluationError::OperationError { inner: OperationError::InvalidTypeBinary { .. }, .. });
    }

    #[test]
    fn paren_group_inherent_normal() {
        let ident = value_item_from_str("(my_var * 3)");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number(23 * 3))
    }

    #[test]
    fn paren_group_inherent_can_contain_other_groups() {
        let ident = value_item_from_str("((((my_var))))");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number(23))
    }

    #[test]
    fn ident_value_gets_from_scope() {
        let ident = value_item_from_str("my_var");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number(23))
    }

    #[test]
    fn ident_value_not_in_scope() {
        let ident = value_item_from_str("a_var");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.value(&scope);
        assert_matches!(value.unwrap_err(), ExpressionEvaluationError::VariableDoesNotExist { .. });
    }

    #[test]
    fn numlit_value_positive() {
        let ident = value_item_from_str("42");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number(42));
    }

    #[test]
    fn charlit_value_ascii() {
        let ident = value_item_from_str("'*'");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number(42));
    }

    #[test]
    fn charlit_value_unicode() {
        let ident = value_item_from_str("'↑'");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number('↑' as i32));
    }

    #[test]
    fn charlit_value_escape_sequence_valid() {
        let ident = value_item_from_str("'\\n'");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number('\n' as i32));
    }

    #[test]
    fn charlit_value_escape_sequence_single_quote() {
        let ident = value_item_from_str("'\\'''");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::Number('\'' as i32));
    }

    #[test]
    fn charlit_value_escape_sequence_invalid() {
        let ident = value_item_from_str("'\\3'");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_matches!(value.unwrap_err(), ExpressionEvaluationError::InvalidEscapeSequence { .. });
    }

    #[test]
    fn strlit_value_normal() {
        let ident = value_item_from_str("\"hi\"");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(vec![Value::Number('h' as i32), Value::Number('i' as i32)]));
    }

    #[test]
    fn strlit_value_escape_sequence() {
        let ident = value_item_from_str("\"\\ti\"");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(vec![Value::Number('\t' as i32), Value::Number('i' as i32)]));
    }

    #[test]
    fn strlit_value_empty() {
        let ident = value_item_from_str("\"\"");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(Vec::new()));
    }

    #[test]
    fn list_value_normal() {
        let ident = value_item_from_str("[700 + 32, (42)]");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(vec![Value::Number(732), Value::Number(42)]));
    }

    #[test]
    fn list_value_contains_other_lists() {
        let ident = value_item_from_str("[[1], []]");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(vec![Value::List(vec![Value::Number(1)]), Value::List(Vec::new())]));
    }

    #[test]
    fn list_value_empty() {
        let ident = value_item_from_str("[]");

        let scope = Scope::new();

        let value = ident.value(&scope);
        assert_eq!(value.unwrap(), Value::List(Vec::new()));
    }

    #[test]
    fn unary_plus_does_nothing() {
        let expr = expression_from_str("++++42");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(42));
    }

    #[test]
    fn unary_plus_with_binary_plus() {
        let expr = expression_from_str("3 + ++++42");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(45));
    }

    #[test]
    fn unary_minus_negates() {
        let expr = expression_from_str("-42");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(-42));
    }

    #[test]
    fn unary_minus_with_binary_minus() {
        let expr = expression_from_str("2 - -42");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_eq!(value.unwrap(), Value::Number(44));
    }
}

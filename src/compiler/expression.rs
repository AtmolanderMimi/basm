//! Defines how to get a value or emplacement out of an expression

use std::mem::transmute;

use crate::{compiler::{CompilerError, operators::{BinaryOperator, UnaryOperator}, scope::Scope, string_normalizer::normalize_string_literal, value::Value}, parser::LanguageItem, use_as_parsed};

use_as_parsed!(Expression);
use_as_parsed!(ExpressionItem);

impl Expression {
    fn node(&self) -> &ExpressionItem {
        (&self.0.node).into()
    }

    fn children(&self) -> &[Expression] {
        // compiler::Expression is transparent to parser::Expression
        unsafe { transmute::<&[ParsedExpression], &[Expression]>(&self.0.children) } 
    }

    pub fn evaluate(&self, ctx: &Scope) -> Result<Value, CompilerError> {
        if self.0.is_leaf() {
            let inherent_value = self.node().inherent_value(ctx)?
                .expect("expession is misconstructed, leaf nodes should always represent a value (i.e: not an operation)");

            return Ok(inherent_value);
        }

        let value = match &self.0.node {
            ParsedExpressionItem::BinaryOperator(op) => {
                let op = <&BinaryOperator>::from(op);

                let children = self.children();
                // once again, the pattern matching should guarenty that a binary operation has two children
                let input_values = (children[0].evaluate(ctx)?, children[1].evaluate(ctx)?);

                let value = op.evaluate(input_values.0, input_values.1)
                    .map_err(|err| CompilerError::OperationError { inner: err, expression: self.clone() })?;

                value
            },
            ParsedExpressionItem::UnaryOperator(op) => {
                let op = <&UnaryOperator>::from(op);

                let children = self.children();
                // once again, the pattern matching should guarenty that a unary operation has one child
                let input_value = children[0].evaluate(ctx)?;

                let value = op.evaluate(input_value)
                    .map_err(|err| CompilerError::OperationError { inner: err, expression: self.clone() })?;

                value
            },
            _ => panic!("all other expression item types should be leaves, and thus should have been caught"),
        };

        Ok(value)
    }
}

impl ExpressionItem {
    /// Return the inherent value of the expression item, if there is any.
    /// This method will call Expression::evaluate if it has to evaluate Expressions
    /// (e.g: lists and parenthesis group).
    /// May fail if an variable is not defined in the scope.
    /// (e.g: NumLit() -> Value::Number, StrLit -> Value::List, BinaryOperator -> None)
    pub fn inherent_value(&self, ctx: &Scope) -> Result<Option<Value>, CompilerError> {
        let maybe_value = match &self.0 {
            ParsedExpressionItem::ParenGroup(_, e, _) => {
                let expression = <&Expression>::from(e.as_ref());
                
                let value = expression.evaluate(ctx)?;
                Some(value)
            },
            ParsedExpressionItem::Ident(ident) => {
                let Some(value) = ctx.get(ident.slice_str()) else {
                    return Err(CompilerError::VariableDoesNotExist { ident: ident.clone() })
                };

                Some(value)
            },
            ParsedExpressionItem::NumLit(num) => {
                let num = num.slice_str().parse()
                    .expect("the lexer should guarenty that NumLit's are valid i32");

                Some(Value::Number(num))
            },
            ParsedExpressionItem::CharLit(charlit) => {
                let normalized = normalize_string_literal(either::Either::Right(charlit))?;

                let first_char = normalized.chars().next()
                    .expect("chars should have at least one character");

                Some(Value::Number(first_char as i32))
            },
            ParsedExpressionItem::StrLit(strlit) => {
                let normalized = normalize_string_literal(either::Either::Left(strlit))?;

                let mut list = Vec::new();
                for ch in normalized.chars() {
                    list.push(Value::Number(ch as i32));
                }

                Some(Value::List(list))
            },
            ParsedExpressionItem::Macro(macr) => {
                // TODO:
                todo!("implement macro value")
            },
            ParsedExpressionItem::List(list) => {
                let mut values = Vec::new();

                for (_, item) in &list.items {
                    let value = <&Expression>::from(item).evaluate(ctx)?;
                    values.push(value);
                }

                Some(Value::List(values))
            },
            _ => None,
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

    fn expression_item_from_str(source: &str) -> ExpressionItem {
        let parsed = ParsedExpressionItem::solve_str(source).unwrap();
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
        let expr = expression_from_str("([1,2] + [3,4])@[0,3]");

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
        assert_matches!(value.unwrap_err(), CompilerError::OperationError { inner: OperationError::InvalidTypeUnary { .. }, .. });
    }

    #[test]
    fn expression_error_when_wrong_types_binary() {
        let expr = expression_from_str("73@[1,2,3]");

        let scope = Scope::new();

        let value = expr.evaluate(&scope);
        assert_matches!(value.unwrap_err(), CompilerError::OperationError { inner: OperationError::InvalidTypeBinary { .. }, .. });
    }

    #[test]
    fn paren_group_inherent_normal() {
        let ident = expression_item_from_str("(my_var * 3)");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(23 * 3))
    }

    #[test]
    fn paren_group_inherent_can_contain_other_groups() {
        let ident = expression_item_from_str("((((my_var))))");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(23))
    }

    #[test]
    fn ident_inherent_value_gets_from_scope() {
        let ident = expression_item_from_str("my_var");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(23))
    }

    #[test]
    fn ident_inherent_value_not_in_scope() {
        let ident = expression_item_from_str("a_var");

        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(23));

        let value = ident.inherent_value(&scope);
        assert_matches!(value.unwrap_err(), CompilerError::VariableDoesNotExist { .. });
    }

    #[test]
    fn numlit_inherent_value_positive() {
        let ident = expression_item_from_str("42");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(42));
    }

    #[test]
    fn numlit_inherent_value_negative() {
        let ident = expression_item_from_str("-42");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(-42));
    }

    #[test]
    fn charlit_inherent_value_ascii() {
        let ident = expression_item_from_str("'*'");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number(42));
    }

    #[test]
    fn charlit_inherent_value_unicode() {
        let ident = expression_item_from_str("'↑'");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number('↑' as i32));
    }

    #[test]
    fn charlit_inherent_value_escape_sequence_valid() {
        let ident = expression_item_from_str("'\\n'");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number('\n' as i32));
    }

    #[test]
    fn charlit_inherent_value_escape_sequence_single_quote() {
        let ident = expression_item_from_str("'\\'''");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::Number('\'' as i32));
    }

    #[test]
    fn charlit_inherent_value_escape_sequence_invalid() {
        let ident = expression_item_from_str("'\\3'");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_matches!(value.unwrap_err(), CompilerError::EscapeSequencesIsInvalid { .. });
    }

    #[test]
    fn strlit_inherent_value_normal() {
        let ident = expression_item_from_str("\"hi\"");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(vec![Value::Number('h' as i32), Value::Number('i' as i32)]));
    }

    #[test]
    fn strlit_inherent_value_escape_sequence() {
        let ident = expression_item_from_str("\"\\ti\"");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(vec![Value::Number('\t' as i32), Value::Number('i' as i32)]));
    }

    #[test]
    fn strlit_inherent_value_empty() {
        let ident = expression_item_from_str("\"\"");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(Vec::new()));
    }

    #[test]
    fn list_inherent_value_normal() {
        let ident = expression_item_from_str("[700 + 32, (42)]");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(vec![Value::Number(732), Value::Number(42)]));
    }

    #[test]
    fn list_inherent_value_contains_other_lists() {
        let ident = expression_item_from_str("[[1], []]");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(vec![Value::List(vec![Value::Number(1)]), Value::List(Vec::new())]));
    }

    #[test]
    fn list_inherent_value_empty() {
        let ident = expression_item_from_str("[]");

        let scope = Scope::new();

        let value = ident.inherent_value(&scope);
        assert_eq!(value.unwrap().unwrap(), Value::List(Vec::new()));
    }
}

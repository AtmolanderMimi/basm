//! Defines emplacements (i.e: &var@1).

use std::mem;

use crate::{compiler::{expression::Expression, value::PropertyError}, parser::{BinaryOperator as ParsedBinaryOperator, Expression as ParsedExpression, ExpressionItem as ParsedExpressionItem}};
use thiserror::Error;
use crate::{compiler::{CompilerError, scope::Scope, value::{Value, ValueType}}, parser::LanguageItem, use_as_parsed};

use_as_parsed!(EmplacementExpression);
use_as_parsed!(EmplacementSubExpression);

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EmplacementNormalizationError {
    #[error("tried to index by {}, emplacement can only be indexed by number", invalid_type.name())]
    InvalidIndexValueType {
        invalid_type: ValueType,
    },
    #[error("tried to get the property with a value of type {}, can only get properties with valid strings", invalid_type.name())]
    InvalidPropertyValueType {
        invalid_type: ValueType,
    },
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EmplacementSetError {
    #[error("\"{variable_name}\" referenced in the emplacement does not exist in scope")]
    VariableDoesNotExist {
        variable_name: String,
    },
    #[error("failed to index because value is a {}", value_type.name())]
    VariableCannotBeIndexed {
        value_type: ValueType,
    },
    #[error("failed to index at {index}, it is out of bound (len: {len})")]
    IndexOutOfBound {
        index: i32,
        len: usize,
    },
    #[error("{inner}")]
    PropertyError {
        inner: PropertyError,
    }
}

/// A normalized emplacement (i.e: the expressions where replaced by their value)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NormalizedEmplacement {
    ident: String,
    operations: Vec<EmplacementOperation>,
}

#[derive(Debug, Clone, PartialEq)]
enum EmplacementOperation {
    Index(i32),
    Property(String),
}

impl EmplacementExpression {
    /// Gets the compiler version of the subexpression
    pub fn sub_expression(&self) -> &EmplacementSubExpression {
        <&EmplacementSubExpression>::from(&self.0.sub_expression)
    }

    /// Normalizes the emplacement. (i.e: transforms all expressions into their current value)
    pub fn normalize(&self, ctx: &Scope) -> Result<NormalizedEmplacement, CompilerError> {
        self.sub_expression().normalize(ctx)
    }  
}

impl EmplacementSubExpression {
    /// Normalizes the emplacement. (i.e: transforms all expressions into their current value)
    pub fn normalize(&self, ctx: &Scope) -> Result<NormalizedEmplacement, CompilerError> {
        let inner_expr = &self.0.0;
        if let ParsedExpressionItem::Ident(ident) = &inner_expr.node {
            let emplacement = NormalizedEmplacement {
                ident: ident.slice_str().to_string(),
                operations: Vec::new(),
            };

            return Ok(emplacement);
        }

        let operation = match &inner_expr.node {
            // -- indexing
            ParsedExpressionItem::BinaryOperator(ParsedBinaryOperator::Index(_)) => {
                let rhs = <&Expression>::from(&inner_expr.children[1]);
                let value = rhs.evaluate(ctx)?;

                // indexing by list
                let operation = if let Value::Number(number) = value {
                    EmplacementOperation::Index(number)
                } else {
                    let inner = EmplacementNormalizationError::InvalidIndexValueType { invalid_type: value.type_of() };

                    return Err(CompilerError::EmplacementNormalizationError { inner, expression: rhs.clone() });
                };

                operation
            },
            // -- property
            ParsedExpressionItem::BinaryOperator(ParsedBinaryOperator::Property(_)) => {
                let rhs = <&Expression>::from(&inner_expr.children[1]);
                let value = rhs.evaluate(ctx)?;

                let Some(property_name) = value.as_string() else {
                    let inner = EmplacementNormalizationError::InvalidPropertyValueType { invalid_type: value.type_of() };

                    return Err(CompilerError::EmplacementNormalizationError { inner, expression: rhs.clone() });
                };

                EmplacementOperation::Property(property_name)
            },
            // -- paren group
            ParsedExpressionItem::ParenGroup(_, expr, _) => {
                let emplacement = unsafe { mem::transmute::<&Box<ParsedExpression>, &Box<EmplacementSubExpression>>(expr) };
                return emplacement.normalize(ctx);
            },

            _ => {
                panic!("a properly formed emplacement should only have ident, index and property as nodes")
            }
        };

        let child_emplacement = unsafe { mem::transmute::<&ParsedExpression, &EmplacementSubExpression>(&inner_expr.children[0]) };
        let mut normalized_child_emplacement = child_emplacement.normalize(ctx)?;
        
        normalized_child_emplacement.operations.push(operation);

        Ok(normalized_child_emplacement)
    }
}

impl NormalizedEmplacement {
    pub fn set<'a>(&self, ctx: &'a mut Scope, value: Value) -> Result<(), EmplacementSetError> {
        let Some(original_value) = ctx.get(&self.ident).clone() else {
            return Err(EmplacementSetError::VariableDoesNotExist { variable_name: self.ident.clone() })
        };

        // -- goes searching for the value to modify
        let mut value_queue = vec![original_value];
        for operation in &self.operations {
            match operation {
                EmplacementOperation::Index(number) => {
                    let Value::List(list) = value_queue.last().unwrap() else {
                        return Err(EmplacementSetError::VariableCannotBeIndexed { value_type: value_queue.last().unwrap().type_of() });
                    };

                    let Ok(index): Result<usize, _> = (*number).try_into() else {
                        return Err(EmplacementSetError::IndexOutOfBound { index: *number, len: list.len() });
                    };

                    let Some(element): Option<&Value> = list.get(index) else {
                        return Err(EmplacementSetError::IndexOutOfBound { index: *number, len: list.len() });
                    };

                    value_queue.push(element.clone());
                },
                EmplacementOperation::Property(property) => {
                    let property = value_queue.last().unwrap().get_property(property)
                        .map_err(|err| EmplacementSetError::PropertyError { inner: err })?;

                    value_queue.push(property);
                }
            }
        }

        // -- modifies the value
        *value_queue.last_mut().unwrap() = value;

        // -- puts the modified value in place
        for operation in self.operations.iter().rev() {
            let value = value_queue.pop().unwrap();

            match operation {
                EmplacementOperation::Index(number) => {
                    let Value::List(list) = value_queue.last_mut().unwrap() else {
                        panic!("is list because we already checked when creating the queue");
                    };

                    // we know that the index is valid
                    list[*number as usize] = value;
                },
                EmplacementOperation::Property(property) => {
                    let parent = value_queue.last_mut().unwrap();

                    parent.set_property(property, value)
                        .map_err(|err| EmplacementSetError::PropertyError { inner: err })?;
                }
            }
        }

        // we now have only the original value left in the queue
        debug_assert_eq!(value_queue.len(), 1);
        let new_value = value_queue.pop().unwrap();
        // there should be no error
        ctx.set(&self.ident, new_value).unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Pattern;

    use super::*;

    use std::assert_matches;

    /// Parses and normalizes an emplacement from a string in an empty scope
    fn normalized_from_str(string: &str) -> Result<NormalizedEmplacement, CompilerError> {
        let parsed = ParsedEmplacementExpression::solve_str(string).unwrap();
        let emplacement = EmplacementExpression::from(parsed);

        emplacement.normalize(&Scope::new())
    }

    #[test]
    fn basic_normalized_emplacement() {
        let normalized = normalized_from_str("&ident").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert!(normalized.operations.is_empty());
    }

    #[test]
    fn non_emplacement_operator_normalized_emplacement() {
        normalized_from_str("&(ident@[[]])").unwrap_err();
    }

    #[test]
    fn paren_group_normalized_emplacement() {
        let normalized = normalized_from_str("&(ident)").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert!(normalized.operations.is_empty());
    }

    #[test]
    fn index_number_is_valid_normalized_emplacement() {
        let normalized = normalized_from_str("&ident@42").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 1);
        assert_eq!(normalized.operations[0], EmplacementOperation::Index(42));
    }

    #[test]
    fn index_list_is_invalid_normalized_emplacement() {
        normalized_from_str("&ident@[1,2]").unwrap_err();
    }

    #[test]
    fn property_normalized_emplacement() {
        let normalized = normalized_from_str("&ident.\"len\"").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 1);
        assert_eq!(normalized.operations[0], EmplacementOperation::Property("len".to_string()));
    }

    #[test]
    fn property_invalid_string_normalized_emplacement() {
        normalized_from_str("&ident.[[42]]").unwrap_err();
    }

    #[test]
    fn combined_operator_precedence_respected_normalized_emplacement() {
        let normalized = normalized_from_str("&ident.\"name\"@3").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 2);
        assert_eq!(normalized.operations[0], EmplacementOperation::Property("name".to_string()));
        assert_matches!(normalized.operations[1], EmplacementOperation::Index(_));
    }

    #[test]
    fn combined_operator_precedence_respected2_normalized_emplacement() {
        let normalized = normalized_from_str("&(ident@3).\"len\"").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 2);
        assert_matches!(normalized.operations[0], EmplacementOperation::Index(_));
        assert_eq!(normalized.operations[1], EmplacementOperation::Property("len".to_string()));
    }

    #[test]
    fn expressions_are_normalized_normalized_emplacement() {
        let normalized = normalized_from_str("&ident@(\"hello\"+\", world!\").\"len\"").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 1);
        assert_eq!(normalized.operations[0], EmplacementOperation::Index(13));
    }

    #[test]
    fn set_normalized_simple() {
        let normalized = normalized_from_str("&ident").unwrap();
        let mut scope = Scope::new();
        scope.declare("ident".to_string(), Value::Number(732));

        normalized.set(&mut scope, Value::Number(42)).unwrap();
        
        let new_value = scope.get("ident").unwrap();
        assert_eq!(new_value, Value::Number(42))
    }

    #[test]
    fn set_normalized_index_works() {
        let normalized = normalized_from_str("&ident@1").unwrap();
        let mut scope = Scope::new();
        scope.declare("ident".to_string(), Value::List(vec![Value::Number(732), Value::Number(143)]));

        normalized.set(&mut scope, Value::List(Vec::new())).unwrap();
        
        let new_value = scope.get("ident").unwrap();
        assert_eq!(new_value, Value::List(vec![Value::Number(732), Value::List(Vec::new())]));
    }

    #[test]
    fn set_normalized_chained_index_works() {
        let normalized = normalized_from_str("&ident@1@0").unwrap();
        let mut scope = Scope::new();
        scope.declare("ident".to_string(), Value::List(vec![Value::Number(732), Value::List(vec![Value::Number(143)])]));

        normalized.set(&mut scope, Value::Number(42)).unwrap();
        
        let new_value = scope.get("ident").unwrap();
        assert_eq!(new_value, Value::List(vec![Value::Number(732), Value::List(vec![Value::Number(42)])]));
    }
}

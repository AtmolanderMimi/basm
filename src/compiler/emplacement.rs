//! Defines emplacements (i.e: &var@1).

use std::mem;

use either::Either;

use crate::{compiler::expression::Expression, parser::{BinaryOperator as ParsedBinaryOperator, Expression as ParsedExpression, ExpressionItem as ParsedExpressionItem}};
use thiserror::Error;
use crate::{compiler::{CompilerError, scope::Scope, value::{Value, ValueType}}, parser::LanguageItem, use_as_parsed};

use_as_parsed!(EmplacementExpression);
use_as_parsed!(EmplacementSubExpression);

#[derive(Debug, Clone, PartialEq, Error)]
pub enum EmplacementNormalizationError {
    #[error("tried to index by {}, emplacement can only be index by number", invalid_type.name())]
    InvalidIndexValueType {
        invalid_type: ValueType,
    },
    #[error("tried to get the property with a value of type {}, can only get properties with valid strings", invalid_type.name())]
    InvalidPropertyValueType {
        invalid_type: ValueType,
    },
}

/// A normalized emplacement (i.e: the expressions where replaced by their value)
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NormalizedEmplacement {
    ident: String,
    operations: Vec<EmplacementOperation>,
}

#[derive(Debug, Clone, PartialEq)]
enum EmplacementOperation {
    Index(Either<i32, Vec<i32>>),
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
                let operation = if let Value::List(vec) = value {
                    // checks if the list is just numbers
                    let invalid_type = vec.iter()
                        .find(|v| !v.type_is_part_of(&ValueType::Number)).map(|v| v.type_of());
                    if let Some(invalid_type) = invalid_type {
                        let inner = EmplacementNormalizationError::InvalidIndexValueType { invalid_type };

                        return Err(CompilerError::EmplacementNormalizationError { inner, expression: rhs.clone() });
                    }

                    let indexes = vec.iter().map(|v| v.as_number().unwrap()).collect();

                    EmplacementOperation::Index(Either::Right(indexes))
                } else if let Value::Number(number) = value {
                    EmplacementOperation::Index(Either::Left(number))
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
    fn index_normalized_emplacement() {
        let normalized = normalized_from_str("&ident@[1,2]").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 1);
        assert_matches!(normalized.operations[0], EmplacementOperation::Index(Either::Right(_)));
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
        let normalized = normalized_from_str("&ident.\"name\"@[1,2,3]").unwrap();
        assert_eq!(normalized.ident, "ident");
        assert_eq!(normalized.operations.len(), 2);
        assert_eq!(normalized.operations[0], EmplacementOperation::Property("name".to_string()));
        assert_matches!(normalized.operations[1], EmplacementOperation::Index(_));
    }

    #[test]
    fn combined_operator_precedence_respected2_normalized_emplacement() {
        let normalized = normalized_from_str("&(ident@[1,2,3]).\"len\"").unwrap();
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
        assert_eq!(normalized.operations[0], EmplacementOperation::Index(Either::Left(13)));
    }
}

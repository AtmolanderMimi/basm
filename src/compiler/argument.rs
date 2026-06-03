//! Defines arguments to directives.

use thiserror::Error;

use crate::{compiler::{emplacement::EmplacementExpression, expression::{Expression, ExpressionEvaluationError}, scope::Scope, value::{Value, ValueType}}, parser::LanguageItem, newtype_wrapper};

use crate::parser::Argument as ParsedArgument;
newtype_wrapper!(Argument, ParsedArgument);

#[derive(Debug, PartialEq, Clone, Error)]
pub enum ArgumentError {
    #[error("{inner}")]
    ExpressionEvaluationError {
        inner: ExpressionEvaluationError,
        expression: Expression,
    },
    #[error("expected argument \"{}\" to be an {}, got {}",
        argument.0.slice_str(),
        expected.name(),
        got.name()
    )]
    InvalidArgumentType {
        argument: Argument,
        expected: ArgumentType,
        got: ArgumentType,
    },
    #[error("expected argument \"{}\"'s value to be of type {}, got {}",
        argument.0.slice_str(),
        expected.name(),
        got.name()
    )]
    InvalidValueType {
        argument: Argument,
        expected: ValueType,
        got: ValueType,
    },
}

impl Argument {
    /// Returns true if the argument is an emplacement
    pub fn is_emplacement(&self) -> bool {
        match self.0 {
            ParsedArgument::EmplacementExpression(_) => true,
            _ => false,
        }
    }

    // Returns the [ArgumentType] equivalent to the type of the argument
    pub fn get_type(&self) -> ArgumentType {
        match self.0 {
            ParsedArgument::Expression(_) => ArgumentType::Expression,
            ParsedArgument::EmplacementExpression(_) => ArgumentType::Emplacement,
        }
    }

    /// Gets the expression of the argument.
    pub fn get_as_expression(&self) -> &Expression {
        match &self.0 {
            ParsedArgument::Expression(e) => <&Expression>::from(e),
            ParsedArgument::EmplacementExpression(e) => <&Expression>::from(&e.sub_expression.0),
        }
    }

    /// Gets the emplacement of a emplacement argument
    /// or None if the argument is not an emplacement.
    pub fn get_emplacement(&self) -> Option<&EmplacementExpression> {
        match &self.0 {
            ParsedArgument::EmplacementExpression(e) => Some(<&EmplacementExpression>::from(e)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentType {
    Expression,
    Emplacement,
}

impl ArgumentType {
    fn name(&self) -> &str {
        match self {
            Self::Expression => "expression",
            Self::Emplacement => "emplacement",
        }
    }
}

/// A pair of the type of argument (expression/emplacement) and the type of it's resulting value.
/// Use for type checking both at the same type.
#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentValueTypePair {
    argument_type: ArgumentType,
    value_type: Option<ValueType>,
}

impl ArgumentValueTypePair {
    /// Creates a new pair.
    pub fn new(argument_type: ArgumentType, value_type: ValueType) -> Self {
        ArgumentValueTypePair { argument_type, value_type: Some(value_type), }
    }

    /// Creates a new pair without an associated value type,
    /// this means that, on evaluation, no value will be evaluated
    /// (useful to describe the type of arguments of directives like #set).
    pub fn new_without_value(argument_type: ArgumentType) -> Self {
        ArgumentValueTypePair { argument_type, value_type: None, }
    }

    /// Evaluates the value and checks if the argument and its value matches the type pair.
    /// Returns the value of the argument, if the argument was expected to have a value and there is a match,
    /// else returns the cause of the mismatch.
    pub fn evaluate_as(&self, ctx: &mut Scope, argument: &Argument) -> Result<Option<Value>, ArgumentError> {
        // checks the argument type
        if argument.get_type() != self.argument_type {
            return Err(ArgumentError::InvalidArgumentType {
                argument: argument.clone(), expected: self.argument_type.clone(), got: argument.get_type(),
            });
        }

        // checks the value type
        let Some(expected_value_type) = &self.value_type else {
            return Ok(None)
        };

        let argument_expression = argument.get_as_expression();
        let argument_value = argument_expression.evaluate(ctx)
            .map_err(|err| 
                ArgumentError::ExpressionEvaluationError { inner: err, expression: argument_expression.clone()
            })?;

        if !argument_value.type_is_part_of(&expected_value_type) {
            return Err(ArgumentError::InvalidValueType {
                argument: argument.clone(), expected: expected_value_type.clone(), got: argument_value.type_of(),
            });
        }

        Ok(Some(argument_value))
    }
}

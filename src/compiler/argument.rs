//! Defines arguments to directives.

use thiserror::Error;

use crate::{compiler::{emplacement::{EmplacementExpression, EmplacementNormalizationError, NormalizedEmplacement}, expression::{Expression, ExpressionEvaluationError}, scope::Scope, value::{Value, ValueType}}, newtype_wrapper, parser::LanguageItem};

use crate::parser::Argument as ParsedArgument;
newtype_wrapper!(Argument, ParsedArgument);

#[derive(Debug, PartialEq, Clone, Error)]
pub enum ArgumentError {
    #[error("{inner}")]
    ExpressionEvaluationError {
        inner: ExpressionEvaluationError,
        expression: Expression,
    },
    #[error("{inner}")]
    EmplacementNormalizationError {
        inner: EmplacementNormalizationError,
        emplacement: EmplacementExpression,
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

    /// Normalizes the argument and evaluates it's value.
    /// Fails if it could not evaluate an expression or emplacement.
    fn normalize_with_value(&self, ctx: &Scope) -> Result<NormalizedArgument, ArgumentError> {
        let normalized_argument = match &self.0 {
            ParsedArgument::Expression(e) => {
                let expression = <&Expression>::from(e);
                let value = expression.evaluate(ctx)
                    .map_err(|err| ArgumentError::ExpressionEvaluationError { inner: err, expression: expression.clone() })?;
                NormalizedArgument::Expression(Some(value))
            },
            ParsedArgument::EmplacementExpression(e) => {
                let emplacement_expression = <&EmplacementExpression>::from(e);
                let normalized_emplacement = emplacement_expression.normalize(ctx)
                    .map_err(|err| ArgumentError::EmplacementNormalizationError { inner: err, emplacement: emplacement_expression.clone() })?;

                let expression = <&Expression>::from(&e.sub_expression.0);
                let value = expression.evaluate(ctx)
                    .map_err(|err| ArgumentError::ExpressionEvaluationError { inner: err, expression: expression.clone() })?;
                NormalizedArgument::Emplacement(normalized_emplacement, Some(value))
            }
        };

        Ok(normalized_argument)
    }

    /// Normalizes the argument, does not evaluate the value or check for expression correctness.
    /// Fails if it could not evaluate an expression or emplacement.
    fn normalize_without_value(&self, ctx: &Scope) -> Result<NormalizedArgument, ArgumentError> {
        let normalized_argument = match &self.0 {
            ParsedArgument::Expression(_) => NormalizedArgument::Expression(None),
            ParsedArgument::EmplacementExpression(e) => {
                let emplacement_expression = <&EmplacementExpression>::from(e);
                let normalized_emplacement = emplacement_expression.normalize(ctx)
                    .map_err(|err| ArgumentError::EmplacementNormalizationError { inner: err, emplacement: emplacement_expression.clone() })?;
                NormalizedArgument::Emplacement(normalized_emplacement, None)
            }
        };

        Ok(normalized_argument)
    }
}

/// The values related to an argument which have been normalized.
/// May or may not have a value, depending on wether or not it was requested.
#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedArgument {
    Emplacement(NormalizedEmplacement, Option<Value>),
    Expression(Option<Value>),
}

impl NormalizedArgument {
    /// Returns the value, if any was evaluated.
    pub fn value(&self) -> Option<&Value> {
        match self {
            Self::Emplacement(_, v) => v.as_ref(),
            Self::Expression(v) => v.as_ref(),
        }
    }

    /// Returns the normalized emplacement, if the argument is an emplacement.
    pub fn emplacement(&self) -> Option<&NormalizedEmplacement> {
        match self {
            Self::Emplacement(e, _) => Some(e),
            Self::Expression(_) => None,
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
    pub fn normalize_as(&self, ctx: &Scope, argument: &Argument) -> Result<NormalizedArgument, ArgumentError> {
        // checks the argument type
        if argument.get_type() != self.argument_type {
            return Err(ArgumentError::InvalidArgumentType {
                argument: argument.clone(), expected: self.argument_type.clone(), got: argument.get_type(),
            });
        }

        // normalises with or withtout value depending on if we need the value
        let normalized_argument = if let Some(expected_value_type) = &self.value_type {
            let normalized_argument = argument.normalize_with_value(ctx)?;

            // .. and checks the type
            let argument_value = normalized_argument.value().unwrap();
            if !argument_value.type_is_part_of(&expected_value_type) {
                return Err(ArgumentError::InvalidValueType {
                    argument: argument.clone(), expected: expected_value_type.clone(), got: argument_value.type_of(),
                });
            }

            normalized_argument
        } else {
            argument.normalize_without_value(ctx)?
        };

        Ok(normalized_argument)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Pattern;

    use super::*;

    fn argument_from_str(string: &str) -> Argument {
        ParsedArgument::solve_str(string).unwrap().into()
    }

    #[test]
    fn argument_normalized_without_value_has_no_value() {
        let normalized_argument = argument_from_str("42").normalize_without_value(&Scope::new()).unwrap();

        assert!(normalized_argument.value().is_none());
    }

    #[test]
    fn argument_normalized_with_value_evaluates_value() {
        let normalized_argument = argument_from_str("3 * 4").normalize_with_value(&Scope::new()).unwrap();

        assert_eq!(normalized_argument.value().unwrap(), &Value::Number(12));
    }

    #[test]
    fn emplacement_argument_normalized_has_normalized_emplacement() {
        let normalized_argument = argument_from_str("&ident@3").normalize_without_value(&Scope::new()).unwrap();

        assert!(normalized_argument.emplacement().is_some());
    }

    #[test]
    fn argument_value_type_pair_normalize_as_expression_accepts_expressions() {
        let argument = argument_from_str("[34, 732]");
        let expression_type = ArgumentValueTypePair::new_without_value(ArgumentType::Expression);
        expression_type.normalize_as(&Scope::new(), &argument).unwrap();
    }

    #[test]
    fn argument_value_type_pair_normalize_as_expression_does_not_accept_emplacement() {
        let argument = argument_from_str("&my_var");
        let expression_type = ArgumentValueTypePair::new_without_value(ArgumentType::Expression);
        expression_type.normalize_as(&Scope::new(), &argument).unwrap_err();
    }

    #[test]
    fn argument_value_type_pair_normalize_as_emplacement_accepts_emplacements() {
        let argument = argument_from_str("&my_var");
        let emplacement_type = ArgumentValueTypePair::new_without_value(ArgumentType::Emplacement);
        emplacement_type.normalize_as(&Scope::new(), &argument).unwrap();
    }

    #[test]
    fn argument_value_type_pair_normalize_as_emplacement_does_not_accept_expression() {
        let argument = argument_from_str("[34, 732]");
        let emplacement_type = ArgumentValueTypePair::new_without_value(ArgumentType::Emplacement);
        emplacement_type.normalize_as(&Scope::new(), &argument).unwrap_err();
    }

    #[test]
    fn argument_value_type_pair_normalize_as_no_value_is_evaluated_when_none() {
        let argument = argument_from_str("[34, 732]");
        let emplacement_type = ArgumentValueTypePair::new_without_value(ArgumentType::Expression);
        let normalized = emplacement_type.normalize_as(&Scope::new(), &argument).unwrap();

        assert!(normalized.value().is_none());
    }

    #[test]
    fn argument_value_type_pair_normalize_as_value_is_evaluated_when_some() {
        let argument = argument_from_str("[34, 732]");
        let emplacement_type = ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::List);
        let normalized = emplacement_type.normalize_as(&Scope::new(), &argument).unwrap();

        assert!(normalized.value().is_some());
    }

    #[test]
    fn argument_value_type_pair_normalize_as_errors_when_value_type_does_not_match() {
        let argument = argument_from_str("[34, 732]");
        let emplacement_type = ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Number);
        emplacement_type.normalize_as(&Scope::new(), &argument).unwrap_err();
    }
}

//! Defines the execution of directives

use std::{collections::HashMap, sync::LazyLock};

use thiserror::Error;

use crate::{compiler::{CompilerError, expression::Expression, scope::Scope, value::{Value, ValueType}}, parser::{EmplacementExpression, LanguageItem}, use_as_parsed};

use_as_parsed!(Argument);
use_as_parsed!(Directive);

#[derive(Debug, PartialEq, Clone, Error)]
pub enum DirectiveInlineError {
    #[error("directive \"{0}\" does not exist")]
    NameDoesNotExist(String),
    #[error("expected {expected} arguments, got {got}")]
    InvalidNumberOfArguments {
        expected: usize,
        got: usize,
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

    pub fn get_type(&self) -> ArgumentType {
        match self.0 {
            ParsedArgument::Expression(_) => ArgumentType::Expression,
            ParsedArgument::EmplacementExpression(_) => ArgumentType::Emplacement,
        }
    }

    /// Gets the expression of a expression argument
    /// or None if the argument is not an expression
    pub fn get_as_expression(&self) -> &Expression {
        match &self.0 {
            ParsedArgument::Expression(e) => <&Expression>::from(e),
            ParsedArgument::EmplacementExpression(e) => <&Expression>::from(&e.sub_expression.0),
        }
    }

    /// Gets the emplacement of a emplacement argument
    /// or None if the argument is not an emplacement
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

impl Directive {
    /// Tries to inline the macro, returns an error on failure.
    pub fn inline(&self, ctx: &mut Scope) -> Result<(), CompilerError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                self.inline_generic(ctx, name.slice_str())
            }
            ParsedDirective::InlineMacro { .. } => todo!(),
        }
    }

    fn inline_generic(&self, ctx: &mut Scope, name: &str) -> Result<(), CompilerError> {
        let Some(directive_logic) = DIRECTIVES.get(name) else {
            let inner = DirectiveInlineError::NameDoesNotExist(name.to_string());

            return Err(CompilerError::DirectiveInlineError { directive: self.clone(), inner });
        };

        directive_logic.inline(ctx, self)
    }

    /// Returns the list of arguments.
    fn arguments(&self) -> Vec<&Argument> {
        match &self.0 {
            ParsedDirective::Generic { arguments, .. } => {
                arguments.iter().map(|(_, a)| a.into()).collect()
            },
            ParsedDirective::InlineMacro { arguments: Some(arguments), .. } => {
                arguments.1.iter().map(|(_, a)| a.into()).collect()
            },
            ParsedDirective::InlineMacro { arguments: None, .. } => {
                Vec::new()
            },
        }
    }
}

struct GenericDirectiveLogic {
    arguments: Vec<(ArgumentType, Option<ValueType>)>,
    /// function that takes in the values of arguments,
    /// returns a list where emplacement argument are Some and Expression are None.
    function: fn(&mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveInlineError>,
}

impl GenericDirectiveLogic {
    pub fn new(
        func: fn(&mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveInlineError>,
        argument_types: Vec<(ArgumentType, Option<ValueType>)>,
    ) -> Self {
        GenericDirectiveLogic {
            arguments: argument_types,
            function: func,
        }
    }

    /// Checks the type of arguments, will call `function`
    /// with the values and update the variables passed with emplacement.
    pub fn inline(&self, ctx: &mut Scope, directive: &Directive) -> Result<(), CompilerError> {
        let arguments = directive.arguments();        

        // check argument count
        if arguments.len() != self.arguments.len() {
            let inner = DirectiveInlineError::InvalidNumberOfArguments {
                expected: self.arguments.len(),
                got: arguments.len(),
            };

            return Err(CompilerError::DirectiveInlineError {
                inner,
                directive: directive.clone(),
            });
        }

        // checks argument type
        for ((expected_arg_type, _), &argument) in self.arguments.iter().zip(&arguments) {
            if *expected_arg_type != argument.get_type() {
                let inner = DirectiveInlineError::InvalidArgumentType {
                    argument: argument.clone(),
                    expected: expected_arg_type.clone(),
                    got: argument.get_type(),
                };

                return Err(CompilerError::DirectiveInlineError {
                    inner,
                    directive: directive.clone(),
                });
            }
        }

        // checks value type
        let mut values = Vec::new();
        for ((_, expected_val_type), &argument) in self.arguments.iter().zip(&arguments) {
            // if we don't expect a value, we don't need to evaluate the argument
            if expected_val_type.is_none() {
                values.push(None);
            }

            let expected_val_type = expected_val_type.as_ref().unwrap();
            let value = argument.get_as_expression().evaluate(ctx)?;
            if !value.type_is_part_of(expected_val_type) {
                let inner = DirectiveInlineError::InvalidValueType {
                    argument: argument.clone(),
                    expected: expected_val_type.clone(),
                    got: value.type_of(),
                };

                return Err(CompilerError::DirectiveInlineError {
                    inner,
                    directive: directive.clone(),
                });
            }

            values.push(Some(value));
        }

        let arguments = arguments.into_iter().zip(values)
            .collect::<Vec<_>>();
        (self.function)(ctx, &arguments)
            .map_err(|inner| CompilerError::DirectiveInlineError {
                    inner,
                    directive: directive.clone(),
            })?;

        Ok(())
    }
}

static DIRECTIVES: LazyLock<HashMap<String, GenericDirectiveLogic>> = LazyLock::new(|| {
    let mut hash_map: HashMap<String, GenericDirectiveLogic> = HashMap::new();
    
    hash_map.insert("raw".to_string(), GenericDirectiveLogic::new(
        directive_raw,
        vec![(ArgumentType::Expression, Some(ValueType::String))],
    ));

    hash_map
});

fn directive_raw(ctx: &mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveInlineError> {
    let msg = arguments[0].1.as_ref().unwrap()
        .as_string().unwrap();

    ctx.write_output(&msg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parser::Pattern;

    use super::*;

    fn directive_from_str(string: &str) -> Directive {
        ParsedDirective::solve_str(string).unwrap().into()
    }

    #[test]
    fn generic_directive_too_many_arguments() {
        let directive = directive_from_str("#raw \"hi :D\", 732;");
        directive.inline(&mut Scope::new()).unwrap_err();
    }

    #[test]
    fn generic_directive_not_enough_arguments() {
        let directive = directive_from_str("#raw;");
        directive.inline(&mut Scope::new()).unwrap_err();
    }

    #[test]
    fn generic_directive_invalid_type() {
        let directive = directive_from_str("#raw 732;");
        directive.inline(&mut Scope::new()).unwrap_err();
    }

    #[test]
    fn raw_generic_adds_to_output() {
        let directive = directive_from_str("#raw \"hi\" + \" :D\";");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), "hi :D");
    }

    #[test]
    fn raw_generic_works_with_lists() {
        let directive = directive_from_str("#raw [32, 42];");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), " *");
    }
}


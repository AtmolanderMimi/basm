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
    arguments: Vec<(ArgumentType, ValueType)>,
    /// function that takes in the values of arguments,
    /// returns a list where emplacement argument are Some and Expression are None.
    function: fn(&mut Scope, arguments: Vec<Value>) -> Result<Vec<Value>, DirectiveInlineError> ,
}

impl GenericDirectiveLogic {
    pub fn new(
        func: fn(&mut Scope, arguments: Vec<Value>) -> Result<Vec<Value>, DirectiveInlineError>,
        argument_types: Vec<(ArgumentType, ValueType)>,
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

            values.push(value);
        }

        let output = (self.function)(ctx, values)
            .map_err(|inner| CompilerError::DirectiveInlineError {
                    inner,
                    directive: directive.clone(),
            })?;

        let returned_values = output.into_iter()
            .enumerate();

        for (i, new_value) in returned_values {
            todo!("write back the emplacements");
            //arguments[i].get_emplacement()
        }

        Ok(())
    }
}

static DIRECTIVES: LazyLock<HashMap<String, GenericDirectiveLogic>> = LazyLock::new(|| {
    let mut hash_map: HashMap<String, GenericDirectiveLogic> = HashMap::new();
    
    hash_map.insert("raw".to_string(), GenericDirectiveLogic::new(
        directive_raw,
        vec![(ArgumentType::Expression, ValueType::String)],
    ));

    hash_map
});

fn directive_raw(ctx: &mut Scope, arguments: Vec<Value>) -> Result<Vec<Value>, DirectiveInlineError> {
    let msg = arguments[0].as_string().unwrap();

    ctx.write_output(&msg);
    Ok(arguments)
}

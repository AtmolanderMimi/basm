//! Defines the execution of directives

use std::{collections::HashMap, sync::LazyLock};

use thiserror::Error;

use crate::{compiler::{CompilerError, expression::Expression, scope::Scope}, parser::{EmplacementExpression, LanguageItem}, use_as_parsed};

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
    #[error("expected argument \"{}\" to be an {}, got {}", argument.0.slice_str(), expected.name(), got.name())]
    InvalidArgumentType {
        argument: Argument,
        expected: ArgumentType,
        got: ArgumentType,
    },
    #[error("expected argument \"{}\"'s value to be of type {expected}, got {got}", argument.0.slice_str())]
    InvalidValueType {
        argument: Argument,
        expected: String,
        got: String,
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

    /// Returns true if the argument is an expression
    pub fn is_expression(&self) -> bool {
        match self.0 {
            ParsedArgument::Expression(_) => true,
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
    pub fn get_expression(&self) -> Option<&Expression> {
        match &self.0 {
            ParsedArgument::Expression(e) => Some(<&Expression>::from(e)),
            _ => None,
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
                self.inline_generic(ctx, name.slice_str(), &self.arguments())
                    .map_err(|err| CompilerError::DirectiveInlineError { directive: self.clone(), inner: err })
            }
            ParsedDirective::InlineMacro { .. } => todo!(),
        }
    }

    fn inline_generic(&self, ctx: &mut Scope, name: &str, arguments: &[&Argument]) -> Result<(), DirectiveInlineError> {
        let Some(directive_fn) = DIRECTIVES.get(name) else {
            return Err(DirectiveInlineError::NameDoesNotExist(name.to_string()))
        };

        directive_fn(ctx, arguments)
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

static DIRECTIVES: LazyLock<HashMap<String, fn(&mut Scope, &[&Argument]) -> Result<(), DirectiveInlineError>>> = LazyLock::new(|| {
    let mut hash_map: HashMap<String, fn(&mut Scope, &[&Argument]) -> Result<(), DirectiveInlineError>> = HashMap::new();
    
    hash_map.insert("raw".to_string(), directive_raw);

    hash_map
});

fn directive_raw(ctx: &mut Scope, arguments: &[&Argument]) -> Result<(), DirectiveInlineError> {
    todo!()
    // TODO: declarative type checking system for directives
    //if arguments.len() != 1 {
    //    return Err(DirectiveInlineError::InvalidNumberOfArguments { expected: 1, got: arguments.len() }));
    //}
    //
    //if !arguments[0].is_expression() {
    //    return Err(DirectiveInlineError::InvalidArgumentType { argument: argument[0], expected: ArgumentType::Expression, got: arguments[0].get_type() })
    //}
    //
    //let value = arguments[0].get_expression().unwrap().evaluate(ctx)
}

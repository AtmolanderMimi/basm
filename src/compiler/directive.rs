//! Defines the execution of directives

use std::{collections::HashMap, sync::LazyLock};

use thiserror::Error;

use crate::{compiler::{argument::{Argument, ArgumentError, ArgumentType, ArgumentValueTypePair}, scope::Scope, value::{Value, ValueType}}, parser::LanguageItem, use_as_parsed};

use_as_parsed!(Directive);

#[derive(Debug, PartialEq, Clone, Error)]
pub enum DirectiveError {
    #[error("directive \"{0}\" does not exist")]
    DirectiveDoesNotExist(String),
    #[error("{inner}")]
    ArgumentError {
        inner: ArgumentError,
        argument: Argument,
    },
    #[error("expected {expected} arguments, got {got} arguments")]
    InvalidArgumentCount {
        expected: usize,
        got: usize,
    }
}

impl Directive {
    /// Tries to inline the macro, returns an error on failure.
    pub fn inline(&self, ctx: &mut Scope) -> Result<(), DirectiveError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                self.inline_generic(ctx, name.slice_str())
            }
            ParsedDirective::InlineMacro { .. } => todo!(),
        }
    }

    fn inline_generic(&self, ctx: &mut Scope, name: &str) -> Result<(), DirectiveError> {
        let Some(directive_data) = DIRECTIVES.get(name) else {
            return Err(DirectiveError::DirectiveDoesNotExist(name.to_string()));
        };

        let arguments = self.validated_argument_value(ctx)?;

        (directive_data.function)(ctx, &arguments)
    }

    /// Returns the list of arguments and their values (or None if the argument is not expected to have a value).
    /// Returns an error if the arguments to the directive are not what they were expected.
    fn validated_argument_value(&self, ctx: &mut Scope) -> Result<Vec<(&Argument, Option<Value>)>, DirectiveError> {
        let actual_arguments = self.arguments();
        let expected_argument_types = self.expected_argument_value_type_pair()?;

        if actual_arguments.len() != expected_argument_types.len() {
            return Err(DirectiveError::InvalidArgumentCount {
                expected: expected_argument_types.len(),
                got: actual_arguments.len(),
            });
        }

        let mut values = Vec::new();
        for (i, argument) in actual_arguments.into_iter().enumerate() {
            let value = expected_argument_types[i].evaluate_as(ctx, argument)
                .map_err(|err| DirectiveError::ArgumentError { inner: err, argument: argument.clone() })?;

            values.push((argument, value));
        }

        Ok(values)
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

    /// Returns the list of argument-value type pair expected for the directive.
    fn expected_argument_value_type_pair(&self) -> Result<&[ArgumentValueTypePair], DirectiveError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                let name = name.slice_str();

                let Some(directive_data) = DIRECTIVES.get(name) else {
                    return Err(DirectiveError::DirectiveDoesNotExist(name.to_string()));
                };

                Ok(&directive_data.argument_types)
            },
            ParsedDirective::InlineMacro { macro_expression, .. } => todo!("implement macro type"),
        }
    }
}

struct GenericDirectiveData {
    pub argument_types: Vec<ArgumentValueTypePair>,
    /// function that takes in the values of arguments,
    /// returns a list where emplacement argument are Some and Expression are None.
    pub function: fn(&mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveError>,
}

impl GenericDirectiveData {
    pub fn new(
        func: fn(&mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveError>,
        argument_types: Vec<ArgumentValueTypePair>,
    ) -> Self {
        GenericDirectiveData {
            argument_types,
            function: func,
        }
    }
}

static DIRECTIVES: LazyLock<HashMap<String, GenericDirectiveData>> = LazyLock::new(|| {
    let mut hash_map: HashMap<String, GenericDirectiveData> = HashMap::new();
    
    hash_map.insert("raw".to_string(), GenericDirectiveData::new(
        directive_raw,
        vec![ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::String)],
    ));

    hash_map
});

fn directive_raw(ctx: &mut Scope, arguments: &[(&Argument, Option<Value>)]) -> Result<(), DirectiveError> {
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


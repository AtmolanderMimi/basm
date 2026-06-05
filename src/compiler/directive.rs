//! Defines the execution of directives

use std::{collections::HashMap, sync::LazyLock};

use thiserror::Error;

use crate::{compiler::{argument::{Argument, ArgumentError, ArgumentType, ArgumentValueTypePair, NormalizedArgument}, emplacement::{EmplacementDeclareError, EmplacementSetError}, scope::Scope, value::ValueType}, newtype_wrapper, parser::LanguageItem};

use crate::parser::Directive as ParsedDirective;
newtype_wrapper!(Directive, ParsedDirective);

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
    },
    #[error("{inner}")]
    FailedToDeclare {
        inner: EmplacementDeclareError,
        argument: Argument,
    },
    #[error("{inner}")]
    FailedToSet {
        inner: EmplacementSetError,
        argument: Argument,
    },
}

impl Directive {
    /// Tries to inline the block, returns an error on failure.
    pub fn inline(&self, ctx: &mut Scope) -> Result<(), DirectiveError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                self.inline_generic(ctx, name.slice_str())
            }
            ParsedDirective::InlineBlock { .. } => todo!(),
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
    fn validated_argument_value(&self, ctx: &mut Scope) -> Result<Vec<(&Argument, NormalizedArgument)>, DirectiveError> {
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
            let value = expected_argument_types[i].normalize_as(ctx, argument)
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
            ParsedDirective::InlineBlock { arguments: Some(arguments), .. } => {
                arguments.1.iter().map(|(_, a)| a.into()).collect()
            },
            ParsedDirective::InlineBlock { arguments: None, .. } => {
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
            ParsedDirective::InlineBlock { block_expression, .. } => todo!("implement block type"),
        }
    }
}

struct GenericDirectiveData {
    pub argument_types: Vec<ArgumentValueTypePair>,
    /// function that takes in the values of arguments,
    /// returns a list where emplacement argument are Some and Expression are None.
    pub function: fn(&mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError>,
}

impl GenericDirectiveData {
    pub fn new(
        func: fn(&mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError>,
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
    
    // raw <string>
    hash_map.insert("raw".to_string(), GenericDirectiveData::new(
        directive_raw,
        vec![ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::String)],
    ));

    // decl <&ident>, <val>
    hash_map.insert("decl".to_string(), GenericDirectiveData::new(
        directive_decl,
        vec![
            ArgumentValueTypePair::new_without_value(ArgumentType::Emplacement),
            ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Any),
        ],
    ));

    // set <&emplacement>, <val>
    hash_map.insert("set".to_string(), GenericDirectiveData::new(
        directive_set,
        vec![
            ArgumentValueTypePair::new_without_value(ArgumentType::Emplacement),
            ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Any),
        ],
    ));

    hash_map
});

/// `raw <string>`
fn directive_raw(ctx: &mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError> {
    let msg = arguments[0].1.value().unwrap()
        .as_string().unwrap();

    ctx.write_output(&msg);
    Ok(())
}

/// `decl <&ident>, <val>`
fn directive_decl(ctx: &mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError> {
    let var_name_emplacement = arguments[0].1.emplacement().unwrap();

    let var_value = arguments[1].1.value().unwrap();
    var_name_emplacement.declare(ctx, var_value.clone())
        .map_err(|err| DirectiveError::FailedToDeclare {
            inner: err,
            argument: arguments[0].0.clone()
        })?;

    Ok(())
}

/// `set <&emplacement>, <val>`
fn directive_set(ctx: &mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError> {
    let var_name_emplacement = arguments[0].1.emplacement().unwrap();

    let var_value = arguments[1].1.value().unwrap();
    var_name_emplacement.set(ctx, var_value.clone())
        .map_err(|err| DirectiveError::FailedToSet {
            inner: err,
            argument: arguments[0].0.clone()
        })?;

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
        let directive = directive_from_str("#raw \"hello\" + \", world!\" + \" :D\";");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), "hello, world! :D");
    }

    #[test]
    fn raw_generic_works_with_lists() {
        let directive = directive_from_str("#raw [32, 42];");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), " *");
    }
}


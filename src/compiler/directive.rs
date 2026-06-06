//! Defines the execution of directives

use std::{collections::HashMap, sync::LazyLock};

use thiserror::Error;

use crate::{compiler::{argument::{Argument, ArgumentError, ArgumentType, ArgumentValueTypePair, NormalizedArgument}, block::Block, emplacement::{EmplacementDeclareError, EmplacementSetError}, expression::{Expression, ExpressionEvaluationError}, scope::Scope, value::{Value, ValueType}}, newtype_wrapper, parser::LanguageItem};

use crate::parser::Directive as ParsedDirective;
newtype_wrapper!(Directive, ParsedDirective);

#[derive(Debug, PartialEq, Clone, Error)]
pub enum DirectiveError {
    #[error("directive \"{0}\" does not exist")]
    DirectiveDoesNotExist(String),
    #[error("expected a block, got a {}", got_type.name())]
    InlineBlockValueIsNotBlock {
        expression: Expression,
        got_type: ValueType,
    },
    /// error occuring while inlining a block
    #[error("{inner}")]
    InlineBlockError {
        inner: Box<DirectiveError>,
        block: Block,
    },
    #[error("{inner}")]
    ExpressionEvaluationError {
        inner: ExpressionEvaluationError,
        expression: Expression,
    },
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
    /// Returns a Value::Block, if the directive is an inline.
    /// If the directive is an inline, but the expression is not of type Block, returns an error.
    fn block_value(&self, ctx: &Scope) -> Result<Option<Block>, DirectiveError> {
        let ParsedDirective::InlineBlock { block_expression, .. } = &self.0 else {
            return Ok(None);
        };

        let expression = <&Expression>::from(block_expression);
        let value = expression.evaluate(ctx)
            .map_err(|err| DirectiveError::ExpressionEvaluationError {
                inner: err, expression: expression.clone(),
            })?;

        let Some(block) = value.as_block() else {
            return Err(DirectiveError::InlineBlockValueIsNotBlock { expression: expression.clone(), got_type: value.type_of() });
        };

        Ok(Some(block))
    }

    /// Tries to inline the block, returns an error on failure.
    pub fn inline<'a>(&self, ctx: &mut Scope) -> Result<(), DirectiveError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                self.inline_generic(ctx, name.slice_str())
            }
            ParsedDirective::InlineBlock { .. } => self.inline_block(ctx),
        }
    }

    fn inline_generic(&self, ctx: &mut Scope, name: &str) -> Result<(), DirectiveError> {
        let Some(directive_data) = DIRECTIVES.get(name) else {
            return Err(DirectiveError::DirectiveDoesNotExist(name.to_string()));
        };

        let arguments = self.validated_argument_value(ctx)?;

        (directive_data.function)(ctx, &arguments)
    }

    fn inline_block(&self, ctx: &mut Scope) -> Result<(), DirectiveError> {
        let arguments = self.validated_argument_value(ctx)?;
        let block = self.block_value(ctx)?.unwrap();

        block.inline(ctx, arguments)
    }

    /// Returns the list of arguments and their values (or None if the argument is not expected to have a value).
    /// Returns an error if the arguments to the directive are not what they were expected.
    fn validated_argument_value(&self, ctx: &mut Scope) -> Result<Vec<(&Argument, NormalizedArgument)>, DirectiveError> {
        let actual_arguments = self.arguments();
        let expected_argument_types = self.expected_argument_value_type_pair(ctx)?;

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
    fn expected_argument_value_type_pair(&self, ctx: &Scope) -> Result<Vec<ArgumentValueTypePair>, DirectiveError> {
        match &self.0 {
            ParsedDirective::Generic { name, .. } => {
                let name = name.slice_str();

                let Some(directive_data) = DIRECTIVES.get(name) else {
                    return Err(DirectiveError::DirectiveDoesNotExist(name.to_string()));
                };

                Ok(directive_data.argument_types.clone())
            },
            ParsedDirective::InlineBlock { .. } => {
                // all of these unwraps are invariants
                let block = self.block_value(ctx)?.unwrap();

                let argument_value_types = block.argument_types().into_iter()
                    .map(|arg_type| ArgumentValueTypePair::new(arg_type, ValueType::Any))
                    .collect();

                Ok(argument_value_types)
            },
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

    // if <condition>, <block []>
    hash_map.insert("if".to_string(), GenericDirectiveData::new(
        directive_if,
        vec![
            ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Number),
            ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Block(vec![])),
        ],
    ));

    // to_string <&string> <number>
    hash_map.insert("to_string".to_string(), GenericDirectiveData::new(
        directive_to_string,
        vec![
            ArgumentValueTypePair::new_without_value(ArgumentType::Emplacement),
            ArgumentValueTypePair::new(ArgumentType::Expression, ValueType::Number),
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

/// `if <condition>, <block []>`
fn directive_if(ctx: &mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError> {
    let condition_fufiled = arguments[0].1.value().unwrap().is_true().unwrap();
    if condition_fufiled {
        let block = arguments[1].1.value().unwrap().as_block().unwrap();
        block.inline(ctx, Vec::new())?;
    }

    Ok(())
}

/// `to_string <&string> <number>`
fn directive_to_string(ctx: &mut Scope, arguments: &[(&Argument, NormalizedArgument)]) -> Result<(), DirectiveError> {
    let number = arguments[1].1.value().unwrap().as_number().unwrap();
    let number_string = number.to_string();
    let emplacement = arguments[0].1.emplacement().unwrap();
    emplacement.set(ctx, Value::new_from_str(&number_string))
        .map_err(|err| DirectiveError::FailedToSet {
            inner: err,
            argument: arguments[0].0.clone()
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{compiler::value::Value, parser::Pattern};

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
    fn inline_directive_empty() {
        let directive = directive_from_str("{};");
        directive.inline(&mut Scope::new()).unwrap();
    }

    #[test]
    fn inline_directive_arguments() {
        let directive = directive_from_str("[arg1, &arg2] => {}: 32, &my_var;");
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(15));

        directive.inline(&mut scope).unwrap();
    }

    #[test]
    fn inline_directive_modifies_output_arguments() {
        let directive = directive_from_str("[&arg] => { #set &arg, 42; }: &my_var@1;");
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::List(vec![Value::Number(1), Value::Number(2)]));

        directive.inline(&mut scope).unwrap();
        assert_eq!(scope.get("my_var").unwrap(), Value::List(vec![Value::Number(1), Value::Number(42)]));
    }

    #[test]
    fn inline_directive_cannot_modify_invalidated_emplacements() {
        let directive = directive_from_str("[&arg1, &arg2] => { #set &arg1, 0; }: &my_var, &my_var@0;");
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::List(vec![Value::Number(0)]));

        directive.inline(&mut scope).unwrap_err();
    }

    #[test]
    fn inline_directive_local_variables_are_not_accessible_outside() {
        let directive = directive_from_str("{ #decl &inner_var, 0; };");
        let mut scope = Scope::new();

        directive.inline(&mut scope).unwrap();

        assert!(scope.get("inner_var").is_none());
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

    #[test]
    fn decl_generic_adds_variable_to_scope() {
        let directive = directive_from_str("#decl &var, 42;");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get("var").unwrap(), Value::Number(42));
    }

    #[test]
    fn decl_generic_can_only_declare_ident_emplacement() {
        let directive = directive_from_str("#decl &var@0, 42;");
        let mut scope = Scope::new();

        directive.inline(&mut scope).unwrap_err();
    }

    #[test]
    fn set_generic_modifies_value_in_scope() {
        let directive = directive_from_str("#set &var, 42;");
        let mut scope = Scope::new();
        scope.declare("var".to_string(), Value::Number(0));

        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get("var").unwrap(), Value::Number(42));
    }

    #[test]
    fn set_generic_modifies_complex_emplacement() {
        let directive = directive_from_str("#set &(var@0).\"len\", 1;");
        let mut scope = Scope::new();
        scope.declare("var".to_string(), Value::List(vec![Value::List(vec![])]));

        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get("var").unwrap(), Value::List(vec![Value::List(vec![Value::default()])]));
    }

    #[test]
    fn set_generic_errors_if_variable_does_not_exist() {
        let directive = directive_from_str("#set &var, 42;");
        let mut scope = Scope::new();

        directive.inline(&mut scope).unwrap_err();
    }

    #[test]
    fn if_generic_inlines_the_scope_when_true() {
        let directive = directive_from_str("#if 1, { #raw \"inlined\"; };");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), "inlined");
    }

    #[test]
    fn if_generic_does_not_inlines_the_scope_when_false() {
        let directive = directive_from_str("#if 0, { #raw \"inlined\"; };");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap();

        assert_eq!(scope.get_output(), "");
    }

    #[test]
    fn if_generic_does_accept_argumented_block() {
        let directive = directive_from_str("#if 0, [arg] => { #raw \"inlined\"; };");
        let mut scope = Scope::new();
        directive.inline(&mut scope).unwrap_err();
    }

    #[test]
    fn to_string_generic_positive_number() {
        let directive = directive_from_str("#to_string &my_var, 42;");
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(0));

        directive.inline(&mut scope).unwrap();

        let string = scope.get("my_var").unwrap().as_string().unwrap();
        assert_eq!(string, "42");
    }

    #[test]
    fn to_string_generic_negative_number() {
        let directive = directive_from_str("#to_string &my_var, 0-732;");
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(0));

        directive.inline(&mut scope).unwrap();

        let string = scope.get("my_var").unwrap().as_string().unwrap();
        assert_eq!(string, "-732");
    }
}


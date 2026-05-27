//! The scope in which variables can be defined.

use std::collections::HashMap;

use crate::compiler::value::Value;

/// The scdope in which variables can be defined.
/// This can also be a sub-scope, aka a scope which is an extension of a higher one.
#[derive(Debug, PartialEq, Default)]
pub struct Scope<'a, 'b: 'a> {
    local_variables: HashMap<String, Value>,
    parent: Option<&'a mut Scope<'b, 'b>>,
}

impl<'a, 'b: 'a> Scope<'a, 'b> {
    /// Creates a new scope that is not the child of anyone.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares a new local variable,
    /// overwrites any already existing local variable of the same name if there were any.1
    pub fn declare(&mut self, variable_name: String, value: Value) {
        self.local_variables.insert(variable_name, value);
    }

    /// Sets the value of a variable,
    /// returns `Err` if the variable is not declared in this scope.
    /// This method will search for the variable in super-scopes if it is not found in the current scope.
    pub fn set(&mut self, variable_name: &str, value: Value) -> Result<(), ()> {
        let local_entry = self.local_variables.get_mut(variable_name);

        if let Some(local_entry) = local_entry {
            *local_entry = value;
            Ok(())
        } else {
            self.parent.as_mut()
                .map(|p| p.set(variable_name, value))
                .unwrap_or(Err(()))
        }
    }

    /// Gets the value of a variable,
    /// returns `Err` if the variable is not declared in this scope.
    /// This method will search for the variable in super-scopes if it is not found in the current scope.
    pub fn get(&self, variable_name: &str) -> Result<Value, ()> {
        let local_entry = self.local_variables.get(variable_name);

        if let Some(local_entry) = local_entry {
            Ok(local_entry.clone())
        } else {
            self.parent.as_ref()
                .map(|p| p.get(variable_name))
                .unwrap_or(Err(()))
        }
    }
}

impl<'a: 'c, 'c> Scope<'a, 'a> {
    /// Creates a new scope which is the child this one.
    pub fn sub_scope(&'c mut self) -> Scope<'c, 'a> {
        Scope {
            parent: Some(self),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_scope_declarations_shadow_super_scope_declarations() {
        let mut scope = Scope::new();

        // declaring in your own scope
        scope.declare("my_var".to_string(), Value::Number(42));
        assert_eq!(scope.get("my_var"), Ok(Value::Number(42)));

        // declaring in a sub-scope shadows the variable of super-scopes
        let mut sub_scope = scope.sub_scope();
        sub_scope.declare("my_var".to_string(), Value::Number(732));
        assert_eq!(sub_scope.get("my_var"), Ok(Value::Number(732)));
    }

    #[test]
    fn scope_double_declaration_causes_overwrites_value() {
        let mut scope = Scope::new();

        scope.declare("my_var".to_string(), Value::Number(42));
        scope.declare("my_var".to_string(), Value::List(Vec::new()));

        assert_eq!(scope.get("my_var"), Ok(Value::List(Vec::new())));
    }

    #[test]
    fn scope_set_fails_when_the_variable_is_not_declared() {
        let mut scope = Scope::new();

        scope.set("my_var", Value::Number(42)).unwrap_err();
    }

    #[test]
    fn scope_set_modifies_variables_in_super_scope_when_required() {
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(42));

        let mut sub_scope = scope.sub_scope();
        sub_scope.set("my_var", Value::Number(732)).unwrap();

        assert_eq!(scope.get("my_var"), Ok(Value::Number(732)));
    }

    #[test]
    fn scope_get_errors_when_the_variable_does_not_exist() {
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(42));

        let sub_scope = scope.sub_scope();
        assert_eq!(sub_scope.get("not_var"), Err(()));
    }

    #[test]
    fn scope_get_gets_the_first_variable_in_scope_order() {
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(42));

        let mut sub_scope = scope.sub_scope();
        assert_eq!(sub_scope.get("my_var"), Ok(Value::Number(42)));

        sub_scope.set("my_var", Value::Number(732)).unwrap();
        assert_eq!(scope.get("my_var"), Ok(Value::Number(732)));
    }
}

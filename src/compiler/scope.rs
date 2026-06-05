//! The scope in which variables can be defined.

use std::collections::HashMap;

use either::Either;

use crate::compiler::value::Value;

/// The main scope (aka the topmost scope) specific data.
/// It stores data about the program as a whole
#[derive(Debug, Clone, PartialEq, Default)]
struct MainScopeData {
    program_output: String,
}

/// The scope in which variables can be defined.
/// This can also be a sub-scope, aka a scope which is an extension of a higher one.
#[derive(Debug, PartialEq)]
pub struct Scope {
    local_variables: HashMap<String, Value>,
    parent: Either<Box<Scope>, MainScopeData>,
}

impl Scope {
    /// Creates a new scope that is not the child of anyone.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a mutable reference to the main scope data.
    fn main_scope_data(&self) -> &MainScopeData {
        self.parent.as_ref().right_or_else(|s| s.main_scope_data())
    }

    /// Gets a mutable reference to the main scope data.
    fn main_scope_data_mut(&mut self) -> &mut MainScopeData {
        self.parent.as_mut().right_or_else(|s| s.main_scope_data_mut())
    }

    /// Writes output of the program (i.e: the resulting bf).
    pub fn write_output(&mut self, output: &str) {
        self.main_scope_data_mut().program_output.push_str(output);
    }

    /// Gets the output of the program, thus far.
    pub fn get_output(&self) -> &str {
        &self.main_scope_data().program_output
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
                .map_left(|p| p.set(variable_name, value))
                .left_or(Err(()))
        }
    }

    /// Gets the value of a variable,
    /// returns `None` if the variable is not declared in this scope.
    /// This method will search for the variable in super-scopes if it is not found in the current scope.
    pub fn get(&self, variable_name: &str) -> Option<Value> {
        let local_entry = self.local_variables.get(variable_name);

        if let Some(local_entry) = local_entry {
            Some(local_entry.clone())
        } else {
            self.parent.as_ref()
                .map_left(|p| p.get(variable_name))
                .left_or(None)
        }
    }
}

impl Scope {
    /// Creates a new scope which is the child this one.
    pub fn sub_scope(&mut self) {
        replace_with::replace_with_or_default(self, |self_| {
            Scope {
                parent: Either::Left(Box::new(self_)),
                ..Default::default()
            }
        });
    }

    /// Unwraps itself into it's parent scope or does nothing.
    pub fn parent_scope(&mut self) {
        replace_with::replace_with_or_default(self, |self_| {
            if let Either::Left(parent) = self_.parent {
                *parent
            } else {
                self_
            }
        });
    }
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            local_variables: HashMap::default(),
            parent: Either::Right(MainScopeData::default()),
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
        assert_eq!(scope.get("my_var"), Some(Value::Number(42)));

        // declaring in a sub-scope shadows the variable of super-scopes
        scope.sub_scope();
        scope.declare("my_var".to_string(), Value::Number(732));
        assert_eq!(scope.get("my_var"), Some(Value::Number(732)));
    }

    #[test]
    fn scope_double_declaration_causes_overwrites_value() {
        let mut scope = Scope::new();

        scope.declare("my_var".to_string(), Value::Number(42));
        scope.declare("my_var".to_string(), Value::List(Vec::new()));

        assert_eq!(scope.get("my_var"), Some(Value::List(Vec::new())));
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

        scope.sub_scope();
        scope.set("my_var", Value::Number(732)).unwrap();
        
        scope.parent_scope();
        assert_eq!(scope.get("my_var"), Some(Value::Number(732)));
    }

    #[test]
    fn scope_get_errors_when_the_variable_does_not_exist() {
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(42));

        scope.sub_scope();
        assert_eq!(scope.get("not_var"), None);
    }

    #[test]
    fn scope_get_gets_the_first_variable_in_scope_order() {
        let mut scope = Scope::new();
        scope.declare("my_var".to_string(), Value::Number(42));

        assert_eq!(scope.get("my_var"), Some(Value::Number(42)));

        scope.set("my_var", Value::Number(732)).unwrap();

        scope.parent_scope();
        assert_eq!(scope.get("my_var"), Some(Value::Number(732)));
    }
}

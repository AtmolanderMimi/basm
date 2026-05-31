//! Defines what is a value and it's types.

/// A value either coming from an expression or a variable.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    List(Vec<Value>),
    Macro() // TODO macro goes here
}

impl Value {
    pub const TRUE: Value = Value::Number(1);
    pub const FALSE: Value = Value::Number(0);

    // Returns the inner value of a Value::Number,
    // if this instance of value is Value::Number.
    pub fn as_number(&self) -> Option<i32> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    // Returns the inner value of a Value::List,
    // if this instance of value is Value::List.
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    /// Tries to convert a list of values to a String, fails if the list contains non Number values
    /// or if the value is not a list.
    pub fn as_string(&self) -> Option<String> {
        let list = self.as_list()?;
        if let Some(_) = list.iter().find(|v| !matches!(v, Value::Number(_))) {
            return None;
        }

        let chars = list.iter().map(|item| char::from_u32(u32::from_ne_bytes(item.as_number().unwrap().to_ne_bytes())));

        // if one of the chars is invalid
        if let Some(_) = chars.clone().find(|char| char.is_none()) {
            return None;
        }

        // we know that there is no none
        let string = chars.map(|c| c.unwrap()).collect();

        Some(string)
    }

    pub fn type_of(&self) -> ValueType {
        match self {
            Self::Number(_) => ValueType::Number,
            Self::List(_) if self.as_string().is_some() => ValueType::String,
            Self::List(_) => ValueType::List,
            Self::Macro(..) => ValueType::Macro,
        }
    }

    pub fn type_is_part_of(&self, vtype: &ValueType) -> bool {
        self.type_of().is_part_of(vtype)
    }

    pub fn type_name(&self) -> &'static str {
        self.type_of().name()
    }

    #[cfg(test)]
    /// Parses and evaluates an expression from a string with an empty scope.
    /// Panics on fail of parsing or evaluating.
    pub fn new_from_str(string: &str) -> Self {
        use crate::{compiler::{expression::Expression, scope::Scope}, parser::{self, Pattern}};

        let expr = Expression::from(parser::Expression::solve_str(string).unwrap());

        let scope = Scope::new();
        expr.evaluate(&scope).unwrap()
    }
}

/// The type of a value.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    /// Any value type
    Any,
    /// Value::Number(_)
    Number,
    /// Value::List(_)
    List,
    /// A list of numbers that can be transfered to string
    String,
    /// Value::Macro
    Macro,
}

impl ValueType {
    pub fn name(&self) -> &'static str {
        match self {
            ValueType::Any => "any",
            ValueType::Number => "number",
            ValueType::List => "list",
            ValueType::String => "string list",
            ValueType::Macro => "macro",
        }
    }

    /// Checks wheter the type is part of the ensemble type `vtype`
    pub fn is_part_of(&self, vtype: &ValueType) -> bool {
        match (self, vtype) {
            (_, Self::Any) => true,
            (Self::Number, Self::Number) => true,
            (Self::List, Self::List) => true,
            (Self::String, Self::List) => true,
            (Self::String, Self::String) => true,
            (Self::Macro, Self::Macro) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_types_are_part_of_any_type() {
        assert!(ValueType::Any.is_part_of(&ValueType::Any));
        assert!(ValueType::Number.is_part_of(&ValueType::Any));
        assert!(ValueType::List.is_part_of(&ValueType::Any));
        assert!(ValueType::String.is_part_of(&ValueType::Any));
        assert!(ValueType::Macro.is_part_of(&ValueType::Any));
    }

    #[test]
    fn all_types_are_part_of_themselves() {
        assert!(ValueType::Any.is_part_of(&ValueType::Any));
        assert!(ValueType::Number.is_part_of(&ValueType::Number));
        assert!(ValueType::List.is_part_of(&ValueType::List));
        assert!(ValueType::String.is_part_of(&ValueType::String));
        assert!(ValueType::Macro.is_part_of(&ValueType::Macro));
    }

    #[test]
    fn string_type_is_part_of_list() {
        assert!(ValueType::String.is_part_of(&ValueType::List));
    }

    #[test]
    fn value_as_string_is_valid() {
        let value = Value::new_from_str("\"\\nHéllo ↑ & rŭnnin.\"");

        assert_eq!(value.type_of(), ValueType::String);
        assert_eq!(value.as_string().unwrap(), "\nHéllo ↑ & rŭnnin.");
    }

    #[test]
    fn value_as_string_from_list_is_valid() {
        let value = Value::new_from_str("['A', '*']");

        assert_eq!(value.type_of(), ValueType::String);
        assert_eq!(value.as_string().unwrap(), "A*");
    }
}

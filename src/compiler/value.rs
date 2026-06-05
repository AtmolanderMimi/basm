//! Defines what is a value and it's types.

use thiserror::Error;

use crate::compiler::{argument::ArgumentType, block::Block};

/// Error relating to properties.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PropertyError {
    #[error("the property \"{property_name}\" does not exist for value of type {}", value_type.name())]
    PropertyDoesNotExist {
        property_name: String,
        value_type: ValueType,
    },
    #[error("the property \"{0}\" is read-only")]
    PropertyCannotBeSet(String),
    #[error("the property value is invalid because {0}")]
    PropertyValueIsInvalid(String),
}

/// A value either coming from an expression or a variable.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    List(Vec<Value>),
    Block(Box<Block>),
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

    // Returns the inner value of a Value::Block,
    // if this instance of value is Value::Block.
    pub fn as_block(&self) -> Option<Block> {
        match self {
            Value::Block(b) => Some(*b.clone()),
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
            Self::Block(b) => ValueType::Block(b.argument_types()),
        }
    }

    pub fn type_is_part_of(&self, vtype: &ValueType) -> bool {
        self.type_of().is_part_of(vtype)
    }

    pub fn type_name<'a>(&self) -> String {
        self.type_of().name()
    }

    pub fn set_property(&mut self, name: &str, value: Value) -> Result<(), PropertyError> {
        match (name, self) {
            // len property
            ("len", Self::List(list)) => {
                let Some(Ok(new_len)) = value.as_number().map(|num| num.try_into()) else {
                    return Err(PropertyError::PropertyValueIsInvalid("\"len\" needs to be a non-negative number".to_string()));
                };

                // new elements in the list are default
                list.resize(new_len, Value::default());
            },
            (_, myself) if myself.get_property(name).is_ok() => return Err(PropertyError::PropertyCannotBeSet(name.to_string())),
            (_, myself)=> return Err(PropertyError::PropertyDoesNotExist { property_name: name.to_string(), value_type: myself.type_of() })
        };

        Ok(())
    }

    pub fn get_property(&self, name: &str) -> Result<Value, PropertyError> {
        let property = match (name, self) {
            // len property
            ("len", Self::List(l)) => {
                let len = l.len().try_into()
                    .expect("there should be no way to excede i32::MAX, unless the user somehow creates a list literal bigger than that");
                Value::Number(len)
            },
            _ => return Err(PropertyError::PropertyDoesNotExist { property_name: name.to_string(), value_type: self.type_of() })
        };

        Ok(property)
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

impl Default for Value {
    fn default() -> Self {
        // the value that will be replicated when a list is expanded
        Self::Number(0)
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
    /// Value::Block(_), contains the types of the arguments
    Block(Vec<ArgumentType>),
}

impl ValueType {
    pub fn name(&self) -> String {
        match self {
            ValueType::Any => "any".to_string(),
            ValueType::Number => "number".to_string(),
            ValueType::List => "list".to_string(),
            ValueType::String => "string list".to_string(),
            ValueType::Block(args) => {
                let mut string_acc = "code block [".to_string();

                // adds each argument
                for (i, arg_type) in args.iter().enumerate() {
                    match arg_type {
                        ArgumentType::Expression => (),
                        ArgumentType::Emplacement => string_acc.push('&'),
                    };

                    string_acc.push_str(&format!("arg{i}, "));
                }

                // returns the trailing ", " if there were items
                if !args.is_empty() {
                    string_acc.pop();
                    string_acc.pop();
                }

                string_acc.push(']');
                string_acc
            },
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
            (Self::Block(args1), Self::Block(args2)) if args1 == args2 => true,
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
        assert!(ValueType::Block(Vec::new()).is_part_of(&ValueType::Any));
    }

    #[test]
    fn all_types_are_part_of_themselves() {
        assert!(ValueType::Any.is_part_of(&ValueType::Any));
        assert!(ValueType::Number.is_part_of(&ValueType::Number));
        assert!(ValueType::List.is_part_of(&ValueType::List));
        assert!(ValueType::String.is_part_of(&ValueType::String));
        assert!(ValueType::Block(Vec::new()).is_part_of(&ValueType::Block(Vec::new())));
    }

    #[test]
    fn string_type_is_part_of_list() {
        assert!(ValueType::String.is_part_of(&ValueType::List));
    }

    #[test]
    fn blocks_with_different_argument_types_are_not_part_of() {
        let block1 = ValueType::Block(vec![ArgumentType::Expression]);
        let block2 = ValueType::Block(vec![ArgumentType::Expression, ArgumentType::Expression]);

        assert!(!block1.is_part_of(&block2));
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

    #[test]
    fn get_property_that_does_not_exist_errors() {
        let list = Value::new_from_str("[1,2,3]");
        list.get_property("not_a_property").unwrap_err();
    }

    #[test]
    fn get_len_property_reflects_lenght_of_list() {
        let list = Value::new_from_str("[1,2,3]");
        let lenght = list.get_property("len").unwrap();

        assert_eq!(lenght, Value::Number(3));
    }

    #[test]
    fn set_len_property_reflects_shortens_list() {
        let mut list = Value::new_from_str("[1,2,3]");
        list.set_property("len", Value::Number(2)).unwrap();

        assert_eq!(list, Value::List(vec![Value::Number(1), Value::Number(2)]));
    }

    #[test]
    fn set_len_property_reflects_expands_list_with_zero() {
        let mut list = Value::new_from_str("[1,2,3]");
        list.set_property("len", Value::Number(5)).unwrap();

        assert_eq!(list, Value::List(vec![Value::Number(1), Value::Number(2), Value::Number(3), Value::Number(0), Value::Number(0)]));
    }

    #[test]
    fn set_len_property_to_negative_errors() {
        let mut list = Value::new_from_str("[1,2,3]");
        list.set_property("len", Value::Number(-2)).unwrap_err();
    }
}

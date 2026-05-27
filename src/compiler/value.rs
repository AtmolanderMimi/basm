//! Defines what is a value and it's types.

/// A value either coming from an expression or a variable.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(i32),
    List(Vec<Value>),
    Macro() // TODO macro goes here
}

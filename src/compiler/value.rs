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

    pub const NUMBER_TYPE: &'static str = "number";
    pub const LIST_TYPE: &'static str = "list";
    pub const MACRO_TYPE: &'static str = "macro";

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Number(_) => Value::NUMBER_TYPE,
            Self::List(_) => Value::LIST_TYPE,
            Self::Macro(..) => Value::MACRO_TYPE,
        }
    }
}

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

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Number(_) => "number",
            Self::List(_) => "list",
            Self::Macro(..) => "macro",
        }
    }
}

//! Defines how to get a value or emplacement out of an expression

use crate::{compiler::{CompilerError, value::Value}, use_as_parsed};

use_as_parsed!(Expression);
use_as_parsed!(ExpressionItem);

/// An expression, wrapper over [crate::parser::Expression].
pub struct Expression(ParsedExpression);

//impl Expression {
//    fn value(&self, ctx: Scope) -> Result<Value, CompilerError> {
//        if self.0.is_leaf() {
//            match self.0.node {
//                ParsedExpressionItem::Ident()
//            }
//        }
//    }
//}
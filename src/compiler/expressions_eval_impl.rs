//! Implements evaluation logic for [`Expression`].

use crate::parser::{Expression, Mod, ValueRepresentation};

use super::{AliasesTrait, CompilerError, ScopeContext};

impl Expression {
    /// Evaluates the expression in the context.
    /// Returns `Err` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &ScopeContext<'_>) -> Result<u32, CompilerError> {
        let mut base = self.base.evaluate(ctx)?;

        for m in &self.mods {
            match m {
                Mod::Increment { value, .. } => {
                    base = base.overflowing_add(value.evaluate(ctx)?).0;
                },
                Mod::Decrement { value, .. } => {
                    base = base.overflowing_sub(value.evaluate(ctx)?).0;
                },
            }
        }

        Ok(base)
    }
}

impl ValueRepresentation {
    /// Evaluates the value in the context.
    /// Returns `Err` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &ScopeContext<'_>) -> Result<u32, CompilerError> {
        let value = match self {
            Self::NumLit(n) => n.value(),
            Self::CharLit(c) => c.value().into(),
            Self::Ident(i) => {
                if let Some(v) = ctx.find_numeric_alias(i.value()) {
                    v
                } else {
                    return Err(CompilerError::AliasNotDefined(i.clone()))
                }
            },
        };

        Ok(value)
    }
}
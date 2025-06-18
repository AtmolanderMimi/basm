//! Implements evaluation logic for [`Expression`].

use crate::parser::{Expression, Mod, ValueRepresentation};

use super::{AliasesTrait, CompilerError};

impl Expression {
    /// Evaluates the expression in the context.
    /// Returns `Err` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &impl AliasesTrait) -> Result<u32, CompilerError> {
        let mut base = self.base.evaluate(ctx)?;

        for m in &self.mods {
            match m {
                Mod::Increment { value, .. } => {
                    base = base.overflowing_add(value.evaluate(ctx)?).0;
                },
                Mod::Decrement { value, .. } => {
                    base = base.overflowing_sub(value.evaluate(ctx)?).0;
                },
                Mod::Multiply { value, .. } => {
                    base = base.wrapping_mul(value.evaluate(ctx)?);
                },
                Mod::Divide { value, .. } => {
                    let num_value = value.evaluate(ctx)?;
                    if num_value == 0 {
                        return Err(CompilerError::DivisionByZero(self.clone()))
                    }

                    base = base / num_value;
                },
            }
        }

        Ok(base)
    }
}

impl ValueRepresentation {
    /// Evaluates the value in the context.
    /// Returns `Err` if an alias is not defined in the context.
    pub fn evaluate(&self, ctx: &impl AliasesTrait) -> Result<u32, CompilerError> {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{compiler::MainContext, lex_file, parser::patterns::{solve_pattern, ExpressionPattern}, source::SourceFile};

    use super::*;

    fn eval_expr_string(string: &str) -> Result<u32, CompilerError> {
        let mut contents = string.to_string();
        // i don't know why, but the pattern doesn't terminate if there isn't a margin like so
        // there the EOF token is there for that too...
        contents.push(' '); 

        let sf = SourceFile::from_raw_parts(
            PathBuf::default(),
            contents,
        ).leak();

        let tokens = lex_file(sf).unwrap();
        let expr = solve_pattern::<ExpressionPattern>(&tokens).unwrap();

        dbg!(expr).evaluate(&MainContext::new())
    }

    #[test]
    fn expression_evalution() {
        assert_eq!(eval_expr_string("3").unwrap(), 3);
        assert_eq!(eval_expr_string("3+40-1").unwrap(), 42);

        // left to right is still right
        // 3+2*5 = (3+2)*5 = 25 and not 3+(2*5) = 13
        assert_eq!(eval_expr_string("3+2*5").unwrap(), 25);
        assert_eq!(eval_expr_string("1+1-2*356").unwrap(), 0);
        assert_eq!(eval_expr_string("2*9-2*4").unwrap(), 64);

        // -- integer division does not break the universe --
        assert_eq!(eval_expr_string("0/8219").unwrap(), 0);
        assert_eq!(eval_expr_string("21/7").unwrap(), 3);

        // 5 / 3 = 1.6666666.. which in integer div should truncate to 1
        assert_eq!(eval_expr_string("5/3").unwrap(), 1);
        assert_eq!(eval_expr_string("2/3").unwrap(), 0);
        assert_eq!(eval_expr_string("589624/9/58").unwrap(), 1129);
        assert_eq!(eval_expr_string("10/3*3").unwrap(), 9); // loss of precision
        assert_eq!(eval_expr_string("12+3*6/31+6").unwrap(), 8);

        // we should never divide by 0
        let res = eval_expr_string("31+3*12/0+15").unwrap_err();
        if let CompilerError::DivisionByZero { .. } = res {
            // good
        } else {
            panic!("Should error from div by 0")
        }
    }
}

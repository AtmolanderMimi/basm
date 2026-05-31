//! Defines how to describe the position of a value.
//! e.g: `&var@3` defines the position of the third value in the list `var`.
//! [EmplacementExpression] simply parses an & + expression and then checks if the expression is an emplacement.

use std::mem;

use crate::{lexer::token::Token, parser::{Expression, ExpressionItem, LanguageItem, Operator, Output, ParseError, ParseErrorVariant::InvalidEmplacement, Pattern, PatternResult, pattern::Then}, source::SfSlice};

/// Defines the position of a value.
#[derive(Debug, Clone, PartialEq)]
pub struct EmplacementExpression {
    pub output: Output,
    pub sub_expression: EmplacementSubExpression,
}

impl LanguageItem for EmplacementExpression {
    fn slice(&self) -> SfSlice {
        let source = self.output.slice().source();
        let start = self.output.slice().start();
        let end = self.output.slice().end();

        SfSlice::from_source(source, start..end)
            .expect("from known positions")
    }
}

impl Pattern for EmplacementExpression {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (tokens_consumed, emplacement) = Then::<Output, EmplacementSubExpression>::solve(tokens)?;

        let emplacement = EmplacementExpression {
            output: emplacement.0,
            sub_expression: emplacement.1
        };

        Ok((tokens_consumed, emplacement))
    }

    fn name() -> String {
        "emplacement expr".to_string()
    }
}

/// An element of an emplacement expression (i.e: a wrapper over expression with constructor checks).
/// (an emplacement without the "&").
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct EmplacementSubExpression(pub Expression);

impl EmplacementSubExpression {
    /// Creates a new EmplacementSubExpression (an emplacement without the "&"),
    /// errors if the expression is not an emplacement.
    pub fn new_from_expression(expression: Expression, tokens_taken: usize) -> Result<Self, ParseError> {
        let emplacement = unsafe { mem::transmute::<Expression, EmplacementSubExpression>(expression) };

        if !emplacement.convey_emplacement() {
            return Err(ParseError { tokens_before_error: tokens_taken, variant: InvalidEmplacement(emplacement) });
        }

        return Ok(emplacement);
    }

    fn convey_emplacement(&self) -> bool {
        match &self.0.node {
            ExpressionItem::Ident(_) => true,
            ExpressionItem::ParenGroup(_, g, _) => {
                unsafe { mem::transmute::<&Box<Expression>, &Box<EmplacementSubExpression>>(g) }
                    .convey_emplacement()
            },
            ExpressionItem::BinaryOperator(op) if op.modifies_an_emplacement() => {
                unsafe { mem::transmute::<&Expression, &EmplacementSubExpression>(&self.0.children[0]) }
                    .convey_emplacement()
            },
            ExpressionItem::UnaryOperator(op) if op.modifies_an_emplacement() => {
                unsafe { mem::transmute::<&Expression, &EmplacementSubExpression>(&self.0.children[0]) }
                    .convey_emplacement()
            },
            _ => false
        }
        
    }
}

impl LanguageItem for EmplacementSubExpression {
    fn slice(&self) -> SfSlice {
        self.0.slice()
    }
}

impl Pattern for EmplacementSubExpression {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let (tokens_consumed, expression) = Expression::solve(tokens)?;

        let sub_emplacement = EmplacementSubExpression::new_from_expression(expression, tokens_consumed)?;

        Ok((tokens_consumed, sub_emplacement))
    }

    fn name() -> String {
        "emplacement subexpr".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex_string;

use super::*;

    #[test]
    fn ident_is_emplacement() {
        let tokens = lex_string("&my_var").unwrap();

        let (tokens_consumed, _) = EmplacementExpression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
    }

    #[test]
    fn without_ampersand_is_not_emplacement() {
        let tokens = lex_string("my_var").unwrap();

        EmplacementExpression::solve(&tokens).unwrap_err();
    }

    #[test]
    fn numlit_is_not_emplacement() {
        let tokens = lex_string("&3").unwrap();

        EmplacementExpression::solve(&tokens).unwrap_err();
    }

    #[test]
    fn parenthesis_group_of_emplacement_is_emplacement() {
        let tokens = lex_string("&(my_var)").unwrap();

        let (tokens_consumed, _) = EmplacementExpression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
    }

    #[test]
    fn non_emplacement_op_on_emplacement_is_not_emplacement() {
        let tokens = lex_string("&my_var + 3").unwrap();

        EmplacementExpression::solve(&tokens).unwrap_err();
    }

    #[test]
    fn index_on_emplacement_is_emplacement() {
        let tokens = lex_string("&my_var@3").unwrap();

        let (tokens_consumed, _) = EmplacementExpression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
    }

    #[test]
    fn property_on_emplacement_is_emplacement() {
        let tokens = lex_string("&my_var.\"len\"").unwrap();

        let (tokens_consumed, _) = EmplacementExpression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
    }

    #[test]
    fn complex_emplacement() {
        let tokens = lex_string("&(my_var@(4+3))@[1 || 0, 2 * my_var, another_index].\"len\"").unwrap();

        let (tokens_consumed, _) = EmplacementExpression::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
    }
}

//! Describes what a directive is.
//! A directive equals in most cases to a single line of text.
//! It has two forms:
//! * Generic: `#Name arg1, arg2, arg3;`
//! * Inline block: `expression: arg1, arg2, arg3;`

use either::Either;

use crate::{lexer::token::Token, parser::{EmplacementExpression, LanguageItem, Pattern, PatternResult, expression::Expression, pattern::{Or, TerminatedSeperatedMany, Then}, terminals::{Colon, Comma, Ident, Pound, Semicolon}}, source::SfSlice};

/// An argument passed to a directive.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Expression(Expression),
    EmplacementExpression(EmplacementExpression),
}

impl LanguageItem for Argument {
    fn slice(&self) -> SfSlice {
        match self {
            Self::Expression(e) => e.slice(),
            Self::EmplacementExpression(e) => e.slice(),
        }
    }
}

impl Pattern for Argument {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = Or::<Expression, EmplacementExpression>::solve(tokens)?;

        let argument = match res.1 {
            Either::Left(e) => Argument::Expression(e),
            Either::Right(e) => Argument::EmplacementExpression(e),
        };

        Ok((res.0, argument))
    }

    fn name() -> String { "directive".to_string() }
}

/// A parsed directive.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Generic {
        pound: Pound,
        name: Ident,
        arguments: Vec<(Option<Comma>, Argument)>,
        semicolon: Semicolon,
    },
    InlineBlock {
        block_expression: Expression,
        arguments: Option<(Colon, Vec<(Option<Comma>, Argument)>)>,
        semicolon: Semicolon,
    }
}

impl LanguageItem for Directive {
    fn slice(&self) -> SfSlice {
        let (source, start) = match self {
            Self::Generic { pound, .. } => {
                let slice = pound.slice();
                (slice.source(), slice.start())
            },
            Self::InlineBlock { block_expression, .. } => {
                let slice = block_expression.slice();
                (slice.source(), slice.start())
            }
        };

        let end = match self {
            Self::Generic { semicolon, .. } => semicolon.slice().end(),
            Self::InlineBlock { semicolon, .. } => semicolon.slice().end(),
        };

        SfSlice::from_source(source, start..end)
            .expect("from known sources")
    }
}

impl Pattern for Directive {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        type GenericPattern = Then<Ident, TerminatedSeperatedMany<Argument, Comma, Semicolon>>;
        type InlineBlock = Then<Expression, Or<Then<Colon, TerminatedSeperatedMany<Argument, Comma, Semicolon>>, Semicolon>>;

        let (tokens_consumed, directive) = if let Ok((pound_token_consumed, pound)) = Pound::solve(tokens) {
            let res = GenericPattern::solve(&tokens[pound_token_consumed..])?;
            let dir = Directive::Generic {
                pound: pound,
                name: res.1.0,
                arguments: res.1.1.0,
                semicolon: res.1.1.1,
            };

            (res.0 + pound_token_consumed, dir)
        } else {
            let res = InlineBlock::solve(&tokens)?;
            let dir = match res.1.1 {
                
                Either::Left((colon, (arguments, semicolon))) => {
                    Directive::InlineBlock {
                        block_expression: res.1.0,
                        arguments: Some((colon, arguments)),
                        semicolon,
                    }
                }
                Either::Right(semicolon) => Directive::InlineBlock {
                    block_expression: res.1.0,
                    arguments: None,
                    semicolon,
                }
            };

            (res.0, dir)
        };

        Ok((tokens_consumed, directive))
    }

    fn name() -> String { "directive".to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex_string;

    use std::assert_matches;

    #[test]
    fn generic_directive_minimal() {
        let tokens = lex_string("#name;").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::Generic { .. }
        );
    }

    #[test]
    fn generic_directive_normal_usecase() {
        let tokens = lex_string("#decl twenty_one, 9 + 10;").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::Generic { .. }
        );
    }

    #[test]
    fn generic_directive_with_a_block() {
        let tokens = lex_string("#decl IncrementA, { #set a, a + 1; };").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::Generic { .. }
        );
    }

    #[test]
    fn generic_directive_does_not_match_without_pound() {
        let tokens = lex_string("decl twenty_one, 9 + 10;").unwrap();

        Directive::solve(&tokens).unwrap_err();
    }

    #[test]
    fn generic_directive_with_emplacement_argument() {
        let tokens = lex_string("#set &my_var[(my_var.\"len\"-1)], 9 + 10;").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::Generic { .. }
        );
    }

    #[test]
    fn inline_directive_minimal() {
        let tokens = lex_string("Macro;").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::InlineBlock { .. }
        );
    }

    #[test]
    fn inline_directive_with_block_literal() {
        let tokens = lex_string("[] => {};").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::InlineBlock { .. }
        );
    }

    #[test]
    fn inline_directive_with_arguments() {
        let tokens = lex_string("Macro: arg1, [\"arg2\"], [] => {};").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::InlineBlock { .. }
        );
    }

    #[test]
    fn inline_directive_with_reference() {
        let tokens = lex_string("Macro: &arg1, &arg2[[1,2,3]];").unwrap();

        let (tokens_consumed, directive) = Directive::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_matches!(
            directive,
            Directive::InlineBlock { .. }
        );
    }
}

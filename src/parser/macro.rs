//! Defines a macro.

use crate::{impl_language_item, lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, directive::Directive, pattern::{Maybe, TerminatedMany, TerminatedSeperatedMany, Then}, terminals::{Comma, Ident, LeftCurly, LeftSquare, RightCurly, RightSquare, ThickArrow}}, source::SfSlice};

/// A macro literal.
#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub arguments: Option<(MacroArguments, ThickArrow)>,
    pub body: MacroBody,
}

impl LanguageItem for Macro {
    fn slice(&self) -> SfSlice {
        let start_index = if let Some((arguments, _)) = &self.arguments {
            arguments.slice().start()
        } else {
            self.body.slice().start()
        };

        SfSlice::from_source(
            self.body.slice().source(),
            start_index..self.body.slice().end()
        ).expect("should always be valid since it slices at the positions of known elements")
    }
}

impl Pattern for Macro {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            Maybe<Then<MacroArguments, ThickArrow>>,
            MacroBody
        >::solve(tokens)?;

        let macr = Macro {
            arguments: (res.1.0),
            body: res.1.1,
        };

        return Ok((res.0, macr));
    }

    fn name() -> String { "macrolit".to_string() }
}

/// The arguments of a macro literal.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroArguments {
    pub opening_bracket: LeftSquare,
    pub arguments: Vec<(Option<Comma>, Ident)>,
    pub closing_bracket: RightSquare,
}

impl_language_item!(MacroArguments, opening_bracket, closing_bracket);

impl Pattern for MacroArguments {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            LeftSquare,
            TerminatedSeperatedMany<
                Ident,
                Comma,
                RightSquare
            >
        >::solve(tokens)?;

        let arguments = MacroArguments {
            opening_bracket: res.1.0,
            arguments: res.1.1.0,
            closing_bracket: res.1.1.1,
        };

        return Ok((res.0, arguments));
    }

    fn name() -> String { "macro arguments".to_string() }
}

/// A the body of a macro literal.
#[derive(Debug, Clone, PartialEq)]
pub struct MacroBody {
    pub opening_bracket: LeftCurly,
    pub directives: Vec<Directive>,
    pub closing_bracket: RightCurly,
}

impl_language_item!(MacroBody, opening_bracket, closing_bracket);

impl Pattern for MacroBody {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            LeftCurly,
            TerminatedMany<
                Directive,
                RightCurly
            >
        >::solve(tokens)?;

        let body = MacroBody {
            opening_bracket: res.1.0,
            directives: res.1.1.0,
            closing_bracket: res.1.1.1,
        };

        return Ok((res.0, body));
    }

    fn name() -> String { "macro body".to_string() }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex_string;

use super::*;

    #[test]
    fn macro_parse_minimal() {
        let tokens = lex_string("{}").unwrap();

        let (tokens_consumed, macr) = Macro::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert!(macr.arguments.is_none());
        assert_eq!(macr.body.directives.len(), 0);
    }

    #[test]
    fn macro_parse_minimal_with_arguments() {
        let tokens = lex_string("[] => {}").unwrap();

        let (tokens_consumed, macr) = Macro::solve(&tokens).unwrap();
        let arguments = macr.arguments.unwrap().0.arguments;

        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_eq!(arguments.len(), 0);
        assert_eq!(macr.body.directives.len(), 0);
    }

    #[test]
    fn macro_parse_normal_usecase() {
        let tokens = lex_string("
        [arg1, arg2, arg3] => {
            #directive_name arg1, [34, arg2, \"hello\"];
            #if '*' == 42, { InAnotherMacro; };
            Macro: arg2;
        }
        ").unwrap();

        let (tokens_consumed, macr) = Macro::solve(&tokens).unwrap();
        let arguments = macr.arguments.unwrap().0.arguments;
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_eq!(arguments.len(), 3);
        assert_eq!(macr.body.directives.len(), 3);
    }

    #[test]
    fn macro_parse_only_accepts_idents_as_args() {
        let tokens = lex_string("[32] => {}").unwrap();

        Macro::solve(&tokens).unwrap_err();
    }
}

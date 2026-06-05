//! Defines a block.

use crate::{impl_language_item, lexer::token::Token, parser::{LanguageItem, Pattern, PatternResult, directive::Directive, pattern::{Maybe, TerminatedMany, TerminatedSeperatedMany, Then}, terminals::{Comma, Ident, LeftCurly, LeftSquare, Output, RightCurly, RightSquare, ThickArrow}}, source::SfSlice};

/// A block literal.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub arguments: Option<(BlockArguments, ThickArrow)>,
    pub body: BlockBody,
}

impl LanguageItem for Block {
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

impl Pattern for Block {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            Maybe<Then<BlockArguments, ThickArrow>>,
            BlockBody
        >::solve(tokens)?;

        let macr = Block {
            arguments: (res.1.0),
            body: res.1.1,
        };

        return Ok((res.0, macr));
    }

    fn name() -> String { "blocklit".to_string() }
}

/// The arguments of a block literal.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockArguments {
    pub opening_bracket: LeftSquare,
    pub arguments: Vec<(Option<Comma>, Option<Output>, Ident)>,
    pub closing_bracket: RightSquare,
}

impl_language_item!(BlockArguments, opening_bracket, closing_bracket);

impl Pattern for BlockArguments {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            LeftSquare,
            TerminatedSeperatedMany<
                Then<Maybe<Output>, Ident>,
                Comma,
                RightSquare
            >
        >::solve(tokens)?;

        // flattens the tuple
        let arguments = res.1.1.0.into_iter()
            .map(|(comma, (refe, ident))| (comma, refe, ident))
            .collect();
        let arguments = BlockArguments {
            opening_bracket: res.1.0,
            arguments,
            closing_bracket: res.1.1.1,
        };

        return Ok((res.0, arguments));
    }

    fn name() -> String { "block arguments".to_string() }
}

/// A the body of a block literal.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockBody {
    pub opening_bracket: LeftCurly,
    pub directives: Vec<Directive>,
    pub closing_bracket: RightCurly,
}

impl_language_item!(BlockBody, opening_bracket, closing_bracket);

impl Pattern for BlockBody {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = 
        Then::<
            LeftCurly,
            TerminatedMany<
                Directive,
                RightCurly
            >
        >::solve(tokens)?;

        let body = BlockBody {
            opening_bracket: res.1.0,
            directives: res.1.1.0,
            closing_bracket: res.1.1.1,
        };

        return Ok((res.0, body));
    }

    fn name() -> String { "block body".to_string() }
}

#[cfg(test)]
mod tests {
    use crate::lexer::lex_string;

use super::*;

    #[test]
    fn block_parse_minimal() {
        let tokens = lex_string("{}").unwrap();

        let (tokens_consumed, macr) = Block::solve(&tokens).unwrap();
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert!(macr.arguments.is_none());
        assert_eq!(macr.body.directives.len(), 0);
    }

    #[test]
    fn block_parse_minimal_with_arguments() {
        let tokens = lex_string("[] => {}").unwrap();

        let (tokens_consumed, macr) = Block::solve(&tokens).unwrap();
        let arguments = macr.arguments.unwrap().0.arguments;

        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_eq!(arguments.len(), 0);
        assert_eq!(macr.body.directives.len(), 0);
    }

    #[test]
    fn block_parse_normal_usecase() {
        let tokens = lex_string("
        [arg1, arg2, arg3] => {
            #directive_name arg1, [34, arg2, \"hello\"];
            #if '*' == 42, { InAnotherMacro; };
            Macro: arg2;
        }
        ").unwrap();

        let (tokens_consumed, macr) = Block::solve(&tokens).unwrap();
        let arguments = macr.arguments.unwrap().0.arguments;
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_eq!(arguments.len(), 3);
        assert_eq!(macr.body.directives.len(), 3);
    }

    #[test]
    fn block_parse_output_argument() {
        let tokens = lex_string("
        [&arg1] => {
            #directive_name arg1, [34, \"hello\"];
            #if '*' == 42, { InAnotherMacro; };
            Macro: &arg1;
        }
        ").unwrap();

        let (tokens_consumed, macr) = Block::solve(&tokens).unwrap();
        let arguments = macr.arguments.unwrap().0.arguments;
        assert_eq!(tokens_consumed, tokens.len()-1);
        assert_eq!(arguments.len(), 1);
        assert_eq!(macr.body.directives.len(), 3);
    }

    #[test]
    fn block_parse_only_accepts_idents_as_args() {
        let tokens = lex_string("[32] => {}").unwrap();

        Block::solve(&tokens).unwrap_err();
    }
}

//! Define a whole file of basm.

use crate::{lexer::token::Token, parser::{Pattern, PatternResult, directive::Directive, pattern::Many}};

/// A file of basm code.
pub struct ParsedFile {
    pub directives: Vec<Directive>,
}

impl Pattern for ParsedFile {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = Many::<Directive>::solve(&tokens)?;

        let file = ParsedFile {
            directives: res.1,
        };

        Ok((res.0, file))
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex_string, parser::parse_tokens};

    #[test]
    fn parses_valid_program_without_error() {
        let tokens = lex_string(include_str!("../../test-resources/fib.basm"))
            .unwrap();

        parse_tokens(&tokens).unwrap();
    }
}

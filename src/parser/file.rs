//! Define a whole file of basm.

use crate::{lexer::token::Token, parser::{Pattern, PatternResult, directive::Directive, pattern::TerminatedMany, terminals::Eof}};

/// A file of basm code.
pub struct ParsedFile {
    pub directives: Vec<Directive>,
    pub eof: Eof
}

impl Pattern for ParsedFile {
    fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
        let res = TerminatedMany::<Directive, Eof>::solve(&tokens)?;

        let file = ParsedFile {
            directives: res.1.0,
            eof: res.1.1,
        };

        Ok((res.0, file))
    }

    fn name() -> String { "file".to_string() }
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

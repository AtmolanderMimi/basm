//! Defines the parsing process for fields like [main] and [data].
//! Meta fields are more complex, and have their own module.

use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::CharOps;

use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::Pattern;

/// Pattern for constructing an [`MainField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainFieldPattern(
    // header
    Then<LeftSquarePattern, Then<MainIdentPattern, Then<RightSquarePattern,
    // contents
    ScopePattern>>>
);

/// The `[main]` field.
/// A `[main]` field, even if empty must always be followed by a scope.
/// 
/// For example:
/// ```basm
/// [main] [
/// // nothing here
/// ]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MainField {
    #[allow(missing_docs)]
    pub left_bracket: LeftSquare,
    #[allow(missing_docs)]
    pub main: MainIdent,
    #[allow(missing_docs)]
    pub right_bracket: RightSquare,
    #[allow(missing_docs)]
    pub contents: Scope,
}

impl Pattern for MainFieldPattern {
    type ParseResult = MainField;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = MainField {
                    left_bracket: res.0,
                    main: res.1.0,
                    right_bracket: res.1.1.0,
                    contents: res.1.1.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

impl LanguageItem for MainField {
    fn slice(&self) -> SfSlice {
        let start = self.left_bracket.0.slice.start_char();
        let end = self.contents.slice().end_char();

        self.left_bracket.0.slice.source().slice_char(start..end)
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::token::TokenType, parser::solve_pattern, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn main_field_token_pattern() {
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("INCR".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(1),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::NumLit(1),
            TokenType::NumLit(1),
            TokenType::InstructionDelimitor,
            TokenType::LSquare,
                TokenType::Ident("ADDP".to_string()),
                TokenType::NumLit(0),
                TokenType::NumLit(1),
                TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MainFieldPattern>(&tokens).unwrap();
        assert_eq!(res.contents.contents.len(), 3);

        // if the name of the field is not "main"
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("not_main".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("INCR".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(1),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::NumLit(1),
            TokenType::NumLit(1),
            TokenType::InstructionDelimitor,
            TokenType::LSquare,
                TokenType::Ident("ADDP".to_string()),
                TokenType::NumLit(0),
                TokenType::NumLit(1),
                TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MainFieldPattern>(&tokens);
        assert!(res.is_err());
    }
}

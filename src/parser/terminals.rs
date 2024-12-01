//! Implementation of terminal patterns, these patterns only represent one token

use crate::lexer::token::{Token, TokenType};
use crate::parser::{AdvancementState, Advancement, Pattern};
use crate::parser::PatternMatchingError;

macro_rules! single_token_pattern {
    ($r:ident, $b:ident, $p:pat, $e:expr) => {
        #[derive(Debug, Clone, PartialEq)]
        pub(super) struct $r<'a> {
            token: Token<'a>
        }

        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Default)]
        pub(super) struct $b;

        impl<'a> Pattern<'a> for $b {
            type ParseResult = $r<'a>;
            fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
                if let $p = token.t_type {
                    let result = $r {
                        token: token.clone(),
                    };
                    return Advancement::new_no_overeach(AdvancementState::Done(result));
                } else {
                    Advancement::new(AdvancementState::Error(PatternMatchingError::UnexpectedToken {
                            expected: $e,
                            got: token.clone().into_owned(),
                        }),
                        1,
                    )
                }
            }
        }
    };
}

single_token_pattern!(
    Ident,
    IdentPattern,
    TokenType::Ident(_),
    TokenType::Ident("any".to_string())
);

single_token_pattern!(
    NumLit,
    NumLitPattern,
    TokenType::NumLit(_),
    TokenType::NumLit(732)
);
single_token_pattern!(
    CharLit,
    CharLitPattern,
    TokenType::CharLit(_),
    TokenType::CharLit('e')
);

single_token_pattern!(
    Plus,
    PlusPattern,
    TokenType::Plus,
    TokenType::Plus
);

single_token_pattern!(
    Minus,
    MinusPattern,
    TokenType::Minus,
    TokenType::Minus
);

single_token_pattern!(
    Semicolon,
    SemicolonPattern,
    TokenType::InstructionDelimitor,
    TokenType::InstructionDelimitor
);

#[cfg(test)]
mod test {
    use crate::source::SfSlice;

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token<'static> {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn single_token_pattern() {
        let token = bogus_token(TokenType::Ident("bublbles".to_string()));
        let adv = IdentPattern::default().advance(
            &token
        );
        if let AdvancementState::Done(_) = adv.state {
            // yay
        } else {
            panic!("expected done")
        }

        let token = bogus_token(TokenType::Plus);
        let adv = IdentPattern::default().advance(
            &token
        );
        if let AdvancementState::Error { .. } = adv.state {
            // yay
        } else {
            panic!("expected the unexpected")
        }

        let token = bogus_token(TokenType::Plus);
        let adv = PlusPattern::default().advance(
            &token
        );
        if let AdvancementState::Done(_) = adv.state {
            // yay
        } else {
            panic!("expected done")
        }

        let token = bogus_token(TokenType::CharLit('2'));
        let adv = CharLitPattern::default().advance(
            &token
        );
        if let AdvancementState::Done(_) = adv.state {
            // yay
        } else {
            panic!("expected done")
        }
    }
}
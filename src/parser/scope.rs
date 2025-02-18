use either::Either;

use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::CharOps;

use super::instruction::Instruction;
use super::instruction::InstructionPattern;
use super::terminals::{LeftSquare, LeftSquarePattern, RightSquare, RightSquarePattern};
use super::componants::{Many, Or, Then};
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::Pattern;

/// Pattern for constructing an [`Scope`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScopePattern(
    // boxed in because of the recursion
    Box<Then<LeftSquarePattern, Then<Many<Or<InstructionPattern, ScopePattern>>, RightSquarePattern>>>
);

/// A scope.
/// A scope is simply a block of square brackets, like so: `[]`.
/// Scopes can contain zero or more instructions or other scopes simultaniously.
/// (although as shown above an empty scope is still valid)
/// Because of that definition, `[main]` is not a scope since
/// it contains tokens that are not valid instructions or scopes.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct Scope {
    pub left_bracket: LeftSquare,
    pub contents: Vec<Either<Instruction, Scope>>,
    pub right_bracket: RightSquare,
}

impl Pattern for ScopePattern {
    type ParseResult = Scope;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = Scope {
                    left_bracket: res.0,
                    contents: res.1.0,
                    right_bracket: res.1.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

impl LanguageItem for Scope {
    fn slice(&self) -> SfSlice {
        let start = self.left_bracket.0.slice.start_char();
        let end = self.right_bracket.0.slice.end_char();
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
    fn scope_token_pattern() {
        let tokens = vec![
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

        let res = solve_pattern::<ScopePattern>(&tokens).unwrap();
        let num_instructions = res.contents.iter().filter(|i| i.is_left()).count();
        let num_scopes = res.contents.iter().filter(|i| i.is_right()).count();
        assert_eq!(num_instructions, 2);
        assert_eq!(num_scopes, 1);

        let tokens = vec![
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
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<ScopePattern>(&tokens);
        assert!(res.is_err())
    }
}

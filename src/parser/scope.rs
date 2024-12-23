use either::Either;

use crate::lexer::token::Token;
use crate::source::SfSlice;

use super::instruction::Instruction;
use super::instruction::InstructionPattern;
use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::Pattern;

/// Pattern for constructing an [`Scope`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScopePattern<'a>(
    // boxed in because of the recursion
    Box<Then<'a, LeftSquarePattern, Then<'a, Many<'a, Or<'a, InstructionPattern<'a>, ScopePattern<'a>>>, RightSquarePattern>>>
);

/// An instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope<'a> {
    pub left_bracket: LeftSquare<'a>,
    pub contents: Vec<Either<Instruction<'a>, Scope<'a>>>,
    pub right_bracket: RightSquare<'a>,
}

impl<'a> Pattern<'a> for ScopePattern<'a> {
    type ParseResult = Scope<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
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

impl<'a> LanguageItem<'a> for Scope<'a> {
    type Owned = Scope<'static>;

    fn into_owned(self) -> Self::Owned {
        let contents = self.contents.into_iter().map(|c| c.map_either(|l| l.into_owned(), |r| r.into_owned()))
            .collect();

        Scope {
            left_bracket: self.left_bracket.into_owned(),
            right_bracket: self.right_bracket.into_owned(),
            contents,
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        let start = self.left_bracket.0.slice.start();
        let end = self.right_bracket.0.slice.end();
        self.left_bracket.0.slice.reslice_char(start..end)
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::token::TokenType, parser::solve_pattern, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token<'static> {
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

use either::Either;

use crate::lexer::token::Token;

use super::expression::Expression;
use super::expression::ExpressionPattern;
use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::Pattern;

/// Pattern for constructing an [`Ìnstruction`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstructionPattern<'a>(
    Then<'a, IdentPattern, Then<'a, Many<'a, Or<'a, ExpressionPattern<'a>, ScopePattern<'a>>>, SemicolonPattern>>
);

/// An instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction<'a> {
    name: Ident<'a>,
    arguments: Vec<Either<Expression<'a>, Scope<'a>>>,
    semicolon: Semicolon<'a>,
}

impl<'a> Pattern<'a> for InstructionPattern<'a> {
    type ParseResult = Instruction<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = Instruction {
                    name: res.0,
                    arguments: res.1.0,
                    semicolon: res.1.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
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
    fn instruction_token_pattern() {
        let tokens = vec![
            TokenType::Ident("ADDP".to_string()),
            TokenType::Ident("i".to_string()),
            TokenType::Minus,
            TokenType::NumLit(0),
            TokenType::Ident("i".to_string()),
            TokenType::Plus,
            TokenType::NumLit(1),
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<InstructionPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);

        let tokens = vec![
            TokenType::Ident("WHNE".to_string()),
            TokenType::Ident("i".to_string()),
            TokenType::NumLit(10),
            TokenType::LSquare,
                TokenType::Ident("INCR".to_string()),
                TokenType::Ident("i".to_string()),
                TokenType::NumLit(1),
                TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<InstructionPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 3);

        let tokens = vec![
            TokenType::Ident("ADDP".to_string()),
            TokenType::Ident("i".to_string()),
            TokenType::Minus,
            TokenType::NumLit(0),
            TokenType::Ident("i".to_string()),
            TokenType::Plus,
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<InstructionPattern>(&tokens);
        assert!(res.is_err());
    }
}

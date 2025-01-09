use either::Either;

use crate::lexer::token::Token;
use crate::source::SfSlice;

use super::expression::Expression;
use super::expression::ExpressionPattern;
use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::Pattern;

/// Pattern for constructing an [`Ìnstruction`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InstructionPattern(
    Then<IdentPattern, Then<Many<Or<ExpressionPattern, ScopePattern>>, SemicolonPattern>>
);

/// An instruction.
/// Even if situated at the end of a scope and even if the last instruction is a scope,
/// instruction must always be concluded by a `;`.
/// Arguments are not seperated by anything other than whitespaces.
/// Because of course, to be identified as different tokens they need no be fused.
/// Although common sense would steer you towards writing instructions in a one line per fashion,
/// there is not restriction on whitespaces and how they are used.
/// 
/// Here is an example of a valid [`Instruction`]:
/// Normal
/// ```basm
/// WHNE 4 0 [
/// ZERO 4;
/// ];
/// ```
/// 
/// Also valid, but unorthodox
/// ```basm
/// WHNE          4
/// 0 [
/// 
///  ZERO  
///    4   ];
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    #[allow(missing_docs)]
    pub name: Ident,
    #[allow(missing_docs)]
    pub arguments: Vec<Either<Expression, Scope>>,
    #[allow(missing_docs)]
    pub semicolon: Semicolon,
}

impl Pattern for InstructionPattern {
    type ParseResult = Instruction;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
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

impl LanguageItem for Instruction {
    fn slice(&self) -> SfSlice {
        let start = self.name.0.slice.start();
        let end = self.semicolon.0.slice.end();
        self.name.0.slice.reslice_char(start..end)
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

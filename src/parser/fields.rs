//! Defines fields like [main] and [@META(arg)].

use either::Either;

use crate::lexer::token::Token;

use super::instruction::Instruction;
use super::instruction::InstructionPattern;
use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::*;
use super::componants::*;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::Pattern;

/// Pattern for constructing an [`MainField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainFieldPattern<'a>(
    // header
    Then<'a, LeftSquarePattern, Then<'a, MainIdentPattern, Then<'a, RightSquarePattern,
    // contents
    Many<'a, Or<'a, InstructionPattern<'a>, ScopePattern<'a>>>>>>
);

/// The `[main]` field.
#[derive(Debug, Clone, PartialEq)]
pub struct MainField<'a> {
    left_bracket: LeftSquare<'a>,
    main: MainIdent<'a>,
    right_bracket: RightSquare<'a>,
    contents: Vec<Either<Instruction<'a>, Scope<'a>>>,
}

impl<'a> Pattern<'a> for MainFieldPattern<'a> {
    type ParseResult = MainField<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
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

/// Pattern for constructing an [`MetaField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetaFieldPattern<'a>(
    // python users getting a stroke from reading the blinding genious that is my use of the type system
    // header
    Then<'a, LeftSquarePattern, Then<'a, AtPattern, Then<'a, IdentPattern, Then<'a, Many<'a, IdentPattern>, Then<'a, RightSquarePattern,
    // contents
    Many<'a, Or<'a, InstructionPattern<'a>, ScopePattern<'a>>
    >>>>>>
);

/// A `[@META arg]` field.
#[derive(Debug, Clone, PartialEq)]
pub struct MetaField<'a> {
    left_bracket: LeftSquare<'a>,
    at: At<'a>,
    name: Ident<'a>,
    arguments: Vec<Ident<'a>>,
    right_bracket: RightSquare<'a>,
    contents: Vec<Either<Instruction<'a>, Scope<'a>>>,
}

impl<'a> Pattern<'a> for MetaFieldPattern<'a> {
    type ParseResult = MetaField<'a>;

    fn advance(&mut self, token: &'a Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = MetaField {
                    left_bracket: res.0,
                    at: res.1.0,
                    name: res.1.1.0,
                    arguments: res.1.1.1.0,
                    right_bracket: res.1.1.1.1.0,
                    contents: res.1.1.1.1.1,
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
    fn main_field_token_pattern() {
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
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

        let res = solve_pattern::<MainFieldPattern>(&tokens).unwrap();
        assert_eq!(res.contents.len(), 3);

        // if the name of the field is not "main"
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("not_main".to_string()),
            TokenType::RSquare,
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

        let res = solve_pattern::<MainFieldPattern>(&tokens);
        assert!(res.is_err());
    }

    #[test]
    fn meta_field_token_pattern() {
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(res.contents.len(), 2);

        // without @
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MetaFieldPattern>(&tokens);
        assert!(res.is_err());

        // with following [main]
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(732),
            TokenType::InstructionDelimitor,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(dbg!(res.contents).len(), 2);
    }
}

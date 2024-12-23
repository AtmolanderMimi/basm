//! Defines fields like [main] and [@META(arg)].

use either::Either;

use crate::lexer::token::Token;
use crate::source::SfSlice;

use super::instruction::Instruction;
use super::instruction::InstructionPattern;
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
pub struct MainFieldPattern<'a>(
    // header
    Then<'a, LeftSquarePattern, Then<'a, MainIdentPattern, Then<'a, RightSquarePattern,
    // contents
    Many<'a, Or<'a, InstructionPattern<'a>, ScopePattern<'a>>>>>>
);

/// The `[main]` field.
#[derive(Debug, Clone, PartialEq)]
pub struct MainField<'a> {
    pub left_bracket: LeftSquare<'a>,
    pub main: MainIdent<'a>,
    pub right_bracket: RightSquare<'a>,
    pub contents: Vec<Either<Instruction<'a>, Scope<'a>>>,
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
    pub left_bracket: LeftSquare<'a>,
    pub at: At<'a>,
    pub name: Ident<'a>,
    pub arguments: Vec<Ident<'a>>,
    pub right_bracket: RightSquare<'a>,
    pub contents: Vec<Either<Instruction<'a>, Scope<'a>>>,
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

impl<'a> LanguageItem<'a> for MainField<'a> {
    type Owned = MainField<'static>;

    fn into_owned(self) -> Self::Owned {
        let contents = self.contents.into_iter().map(|c| c.map_either(|l| l.into_owned(), |r| r.into_owned()))
            .collect();

        MainField {
            left_bracket: self.left_bracket.into_owned(),
            main: self.main.into_owned(),
            right_bracket: self.right_bracket.into_owned(),
            contents,
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        let start = self.left_bracket.0.slice.start();
        let end = self.contents.last()
            .map(|l| l.as_ref().either(|l| l.slice().end(), |r| r.slice().end()))
            .unwrap_or(self.right_bracket.slice().end());

        self.left_bracket.0.slice.reslice_char(start..end)
    }
}

impl<'a> LanguageItem<'a> for MetaField<'a> {
    type Owned = MetaField<'static>;

    fn into_owned(self) -> Self::Owned {
        let contents = self.contents.into_iter().map(|c| c.map_either(|l| l.into_owned(), |r| r.into_owned()))
            .collect();

        MetaField {
            left_bracket: self.left_bracket.into_owned(),
            at: self.at.into_owned(),
            name: self.name.into_owned(),
            right_bracket: self.right_bracket.into_owned(),
            contents,
            arguments: self.arguments.into_iter().map(|a| a.into_owned()).collect()
        }
    }

    fn slice(&self) -> SfSlice<'a> {
        let start = self.left_bracket.0.slice.start();
        let end = self.contents.last()
            .map(|l| l.as_ref().either(|l| l.slice().end(), |r| r.slice().end()))
            .unwrap_or(self.right_bracket.slice().end());

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
        assert_eq!(res.contents.len(), 2);
    }
}

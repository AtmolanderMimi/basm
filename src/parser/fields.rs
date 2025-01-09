//! Defines the parsing process for fields like [main] and [@META(arg)].

use crate::lexer::token::Token;
use crate::source::SfSlice;

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

/// Pattern for constructing an [`MetaField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetaFieldPattern(
    // python users getting a stroke from reading the blinding genious that is my use of the type system
    // header
    Then<LeftSquarePattern, Then<AtPattern, Then<IdentPattern, Then<Many<IdentPattern>, Then<RightSquarePattern,
    // contents
    ScopePattern
    >>>>>
);

/// A `[@META arg]` field.
/// A `[@META]` field, even if empty must always be followed by a scope.
/// 
/// For example:
/// ```basm
/// [@NOTHING] [
/// // nothing here
/// ]
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MetaField {
    #[allow(missing_docs)]
    pub left_bracket: LeftSquare,
    #[allow(missing_docs)]
    pub at: At,
    #[allow(missing_docs)]
    pub name: Ident,
    #[allow(missing_docs)]
    pub arguments: Vec<Ident>,
    #[allow(missing_docs)]
    pub right_bracket: RightSquare,
    #[allow(missing_docs)]
    pub contents: Scope,
}

impl Pattern for MetaFieldPattern {
    type ParseResult = MetaField;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
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

impl LanguageItem for MainField {
    fn slice(&self) -> SfSlice {
        let start = self.left_bracket.0.slice.start();
        let end = self.contents.slice().end();

        self.left_bracket.0.slice.reslice_char(start..end)
    }
}

impl LanguageItem for MetaField {
    fn slice(&self) -> SfSlice {
        let start = self.left_bracket.0.slice.start();
        let end = self.contents.slice().end();

        self.left_bracket.0.slice.reslice_char(start..end)
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

    #[test]
    fn meta_field_token_pattern() {
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(res.contents.contents.len(), 2);

        // without @
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
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
            TokenType::LSquare,
            TokenType::Ident("ZERO".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::Ident("INCR".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("main".to_string()),
            TokenType::RSquare,
            TokenType::LSquare,
            TokenType::Ident("SET".to_string()),
            TokenType::NumLit(0),
            TokenType::NumLit(732),
            TokenType::InstructionDelimitor,
            TokenType::RSquare,
            TokenType::Eof,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(res.contents.contents.len(), 2);
    }
}

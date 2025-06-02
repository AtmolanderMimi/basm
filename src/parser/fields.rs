//! Defines the parsing process for fields like [main] and [data].
//! Meta fields are more complex, and have their own module.

use either::Either;

use crate::impl_language_item;
use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::Sliceable;

use super::componants::Or;
use super::meta_field::MetaFieldPattern;
use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::SetupIdent;
use super::terminals::SetupIdentPattern;
use super::terminals::{LeftSquare, LeftSquarePattern, MainIdent, MainIdentPattern, RightSquare, RightSquarePattern};
use super::componants::Then;
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::MetaField;
use super::Pattern;

/// A representation of a field. Any type of field.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    Main(MainField),
    Setup(SetupField),
    Meta(MetaField),
}

/// Pattern for constructing an [`Field`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FieldPattern(
    Or<MainFieldPattern, Or<SetupFieldPattern, MetaFieldPattern>>
);

impl Pattern for FieldPattern {
    type ParseResult = Field;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = match res {
                    Either::Left(m) => Field::Main(m),
                    Either::Right(Either::Left(s)) => Field::Setup(s),
                    Either::Right(Either::Right(m)) => Field::Meta(m),
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

impl Field {
    /// Returns `true` if the field is a main field.
    pub fn is_main(&self) -> bool {
        if let Field::Main(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the field is a setup field.
    pub fn is_setup(&self) -> bool {
        if let Field::Setup(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the field is a meta field.
    pub fn is_meta(&self) -> bool {
        if let Field::Meta(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `Some` if the field is a main field.
    pub fn unwrap_main(self) -> Option<MainField> {
        if let Field::Main(m) = self {
            Some(m)
        } else {
            None
        }
    }

    /// Returns `Some` if the field is a setup field.
    pub fn unwrap_setup(self) -> Option<SetupField> {
        if let Field::Setup(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns `Some` if the field is a meta field.
    pub fn unwrap_meta(self) -> Option<MetaField> {
        if let Field::Meta(m) = self {
            Some(m)
        } else {
            None
        }
    }
}

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
#[allow(missing_docs)]
pub struct MainField {
    pub left_bracket: LeftSquare,
    pub main: MainIdent,
    pub right_bracket: RightSquare,
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
        let start = self.left_bracket.0.slice.start();
        let end = self.contents.slice().end();

        self.left_bracket.0.slice.source().slice(start..end)
            .unwrap()
    }
}

/// Pattern for constructing an [`SetupField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SetupFieldPattern(
    // header
    Then<LeftSquarePattern, Then<SetupIdentPattern, Then<RightSquarePattern,
    // contents
    ScopePattern>>>
);

/// The `[setup]` field.
/// A `[setup]` field, even if empty must always be followed by a scope.
/// 
/// For example:
/// ```basm
/// [setup] [
/// // nothing here
/// ]
/// ```
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct SetupField {
    pub left_bracket: LeftSquare,
    pub setup: SetupIdent,
    pub right_bracket: RightSquare,
    pub contents: Scope,
}

impl Pattern for SetupFieldPattern {
    type ParseResult = SetupField;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = SetupField {
                    left_bracket: res.0,
                    setup: res.1.0,
                    right_bracket: res.1.1.0,
                    contents: res.1.1.1,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

impl_language_item!(SetupField, left_bracket, contents);

#[cfg(test)]
mod tests {
    use crate::{lex_file, lexer::token::TokenType, parser::{parse_tokens, solve_pattern, PatternMatchingError}, source::{SfSlice, SourceFile}};

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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<MainFieldPattern>(&tokens);
        assert!(res.is_err());
    }

    #[test]
    fn setup_field_token_pattern() {
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("setup".to_string()),
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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<SetupFieldPattern>(&tokens).unwrap();
        assert_eq!(res.contents.contents.len(), 3);

        // if the name of the field is not "main"
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("not_setup".to_string()),
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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<SetupFieldPattern>(&tokens);
        assert!(res.is_err());
    }

    #[test]
    fn more_than_one_main() {
        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this cool clock i got god motherfucking damn] []
        [main] []
        [setup] []
        [@meow] []
        [main] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        if let PatternMatchingError::MoreThanOneMain(_) = parse_tokens(&tokens).unwrap_err() {
            // okay :))
        } else {
            panic!("should have main errored")
        };

        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [setup] []
        [@meow] []
        [main] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        parse_tokens(&tokens).unwrap();
    }

    #[test]
    fn more_than_one_setup() {
        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this cool clock i got god motherfucking damn] []
        [main] []
        [setup] []
        [@meow] []
        [setup] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        if let PatternMatchingError::MoreThanOneSetup(_) = parse_tokens(&tokens).unwrap_err() {
            // okay :))
        } else {
            panic!("should have setup errored")
        };

        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [setup] []
        [@meow] []
        [main] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        parse_tokens(&tokens).unwrap();
    }

    #[test]
    fn one_or_zero_main() {
        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [setup] []
        [@meow] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        assert!(parse_tokens(&tokens).unwrap().main_field.is_none());

        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [main] []
        [setup] []
        [@meow] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        assert!(parse_tokens(&tokens).unwrap().main_field.is_some());
    }

    #[test]
    fn one_or_zero_setup() {
        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [main] []
        [@meow] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        assert!(parse_tokens(&tokens).unwrap().setup_field.is_none());

        let sf = SourceFile::from_raw_parts("./k".into(), 
        "[@look at this clock i got god motherfucking damn] []
        [main] []
        [setup] []
        [@meow] []".to_string()).leak();
        let tokens = lex_file(sf).unwrap();
        assert!(parse_tokens(&tokens).unwrap().setup_field.is_some());
    }
}

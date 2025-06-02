use either::Either;

use crate::impl_language_item;
use crate::lexer::token::Token;
use crate::source::SfSlice;
use crate::utils::Sliceable;

use super::scope::Scope;
use super::scope::ScopePattern;
use super::terminals::{At, AtPattern, Ident, IdentPattern, LeftSquare, LeftSquarePattern, RightSquare, RightSquarePattern};
use super::componants::{Many, Or, Then};
use super::Advancement;
use super::AdvancementState as AdvState;
use super::LanguageItem;
use super::Pattern;
use super::instruction::{ScopeIdent, ScopeIdentPattern};

/// Pattern for constructing an [`MetaField`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MetaFieldPattern(
    // python users getting a stroke from reading the blinding genious that is my use of the type system
    // header
    Then<LeftSquarePattern, Then<AtPattern, Then<IdentPattern, Then<Many<SignatureArgumentPattern>, Then<RightSquarePattern,
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
    pub arguments: Vec<SignatureArgument>,
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

impl_language_item!(MetaField, left_bracket, contents);

/// Pattern to create [`SignatureArgument`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignatureArgumentPattern(
    Or<IdentPattern, ScopeIdentPattern>,
);

impl Pattern for SignatureArgumentPattern {
    type ParseResult = SignatureArgument;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let val = match res {
                    Either::Left(arg) => SignatureArgument::Operand(arg),
                    Either::Right(arg) => SignatureArgument::Scope(arg),
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

/// A meta-instruction argument in the instruction signature.
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureArgument {
    /// An argument expecting an operand.
    Operand(Ident),
    /// An agument expecting a scope.
    Scope(ScopeIdent),
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{lexer::token::TokenType, parser::solve_pattern, source::SfSlice};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn meta_field_token_pattern() {
        // normal
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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(res.contents.contents.len(), 2);

        // normal with scope [arg]
        let tokens = vec![
            TokenType::LSquare,
            TokenType::At,
            TokenType::Ident("IFEQ".to_string()),
            TokenType::Ident("addr".to_string()),
            TokenType::Ident("value".to_string()),
            TokenType::LSquare,
            TokenType::Ident("scope".to_string()),
            TokenType::RSquare,
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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_matches!(res.arguments[2], SignatureArgument::Scope(_));
        assert_eq!(res.arguments.len(), 3);
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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

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
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<MetaFieldPattern>(&tokens).unwrap();
        assert_eq!(res.arguments.len(), 2);
        assert_eq!(res.contents.contents.len(), 2);
    }

    #[test]
    fn signature_argument() {
        // operand
        let tokens = vec![
            TokenType::Ident("value_arg".to_string()),
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<SignatureArgumentPattern>(&tokens).unwrap();
        if let SignatureArgument::Operand(_) = res {
            // ok
        } else {
            panic!("{res:?} was not operand argument")
        }

        // scope
        let tokens = vec![
            TokenType::LSquare,
            TokenType::Ident("scope_arg".to_string()),
            TokenType::RSquare,
        ].into_iter()
        .map(|tt| bogus_token(tt)).collect::<Vec<_>>();

        let res = solve_pattern::<SignatureArgumentPattern>(&tokens).unwrap();
        if let SignatureArgument::Scope(_) = res {
            // ok
        } else {
            panic!("{res:?} was not scope argument")
        }
    }
}
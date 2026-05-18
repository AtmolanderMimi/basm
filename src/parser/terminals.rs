//! Implementation of terminal patterns, these patterns only represent one token

use crate::lexer::token::{Token, TokenType};
use crate::parser::{Pattern, LanguageItem, UnexpectedTokenError, PatternResult};
use crate::source::SfSlice;

macro_rules! single_token_pattern {
    ($name:ident, $match_pattern:pat, $default:expr) => {
        /// Newtype wrapper over a token of the type specified by it's name.
        /// The inner token it **guarentied** to be the same as the name implies.
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(
            pub Token,
        );

        impl LanguageItem for $name {
            fn slice(&self) -> SfSlice {
                self.0.slice.clone()
            }
        }

        impl Pattern for $name {
            fn solve(tokens: &[Token]) -> PatternResult<Self> {
                let Some(token) = tokens.get(0) else {
                    return Err(UnexpectedTokenError::new_got_nothing(vec![$default]))
                };

                if let $match_pattern = token.t_type {
                    let result = $name(token.clone());
                    Ok((1, result))
                } else {
                    Err(UnexpectedTokenError::new(vec![$default], token.clone()))
                }
            }
        }
    };
}

single_token_pattern!(
    Ident,
    TokenType::Ident(_),
    TokenType::Ident("any".to_string())
);

single_token_pattern!(
    NumLit,
    TokenType::NumLit(_),
    TokenType::NumLit(732)
);

single_token_pattern!(
    CharLit,
    TokenType::CharLit(_),
    TokenType::CharLit('e')
);

single_token_pattern!(
    StrLit,
    TokenType::StrLit(_),
    TokenType::StrLit("any".to_string())
);

single_token_pattern!(
    Plus,
    TokenType::Plus,
    TokenType::Plus
);

single_token_pattern!(
    Minus,
    TokenType::Minus,
    TokenType::Minus
);

single_token_pattern!(
    Multiply,
    TokenType::Star,
    TokenType::Star
);

single_token_pattern!(
    Divide,
    TokenType::Slash,
    TokenType::Slash
);

single_token_pattern!(
    Modulo,
    TokenType::Modulo,
    TokenType::Modulo
);

single_token_pattern!(
    LogicalNot,
    TokenType::ExclamationMark,
    TokenType::ExclamationMark
);

single_token_pattern!(
    LogicalAnd,
    TokenType::LogicalAnd,
    TokenType::LogicalAnd
);

single_token_pattern!(
    LogicalOr,
    TokenType::LogicalOr,
    TokenType::LogicalOr
);

single_token_pattern!(
    LogicalEqual,
    TokenType::LogicalEqual,
    TokenType::LogicalEqual
);

single_token_pattern!(
    LogicalInequal,
    TokenType::LogicalInequal,
    TokenType::LogicalInequal
);

single_token_pattern!(
    GreaterThan,
    TokenType::GreaterThan,
    TokenType::GreaterThan
);

single_token_pattern!(
    GreaterThanEqual,
    TokenType::GreaterThanEqual,
    TokenType::GreaterThanEqual
);

single_token_pattern!(
    LessThan,
    TokenType::LessThan,
    TokenType::LessThan
);

single_token_pattern!(
    LessThanEqual,
    TokenType::LessThanEqual,
    TokenType::LessThanEqual
);

single_token_pattern!(
    Semicolon,
    TokenType::Semicolon,
    TokenType::Semicolon
);

single_token_pattern!(
    Colon,
    TokenType::Colon,
    TokenType::Colon
);

single_token_pattern!(
    ThickArrow,
    TokenType::ThickArrow,
    TokenType::ThickArrow
);

single_token_pattern!(
    LeftSquare,
    TokenType::LSquare,
    TokenType::LSquare
);

single_token_pattern!(
    RightSquare,
    TokenType::RSquare,
    TokenType::RSquare
);

single_token_pattern!(
    LeftCurly,
    TokenType::LCurly,
    TokenType::LCurly
);

single_token_pattern!(
    RightCurly,
    TokenType::RCurly,
    TokenType::RCurly
);

single_token_pattern!(
    LeftParen,
    TokenType::LParen,
    TokenType::LParen
);

single_token_pattern!(
    RightParen,
    TokenType::RParen,
    TokenType::RParen
);

single_token_pattern!(
    Comma,
    TokenType::Comma,
    TokenType::Comma
);

single_token_pattern!(
    At,
    TokenType::At,
    TokenType::At
);

single_token_pattern!(
    Pound,
    TokenType::Pound,
    TokenType::Pound
);

single_token_pattern!(
    Eof,
    TokenType::Eof,
    TokenType::Eof
);

// TODO: tests

//! Implementation of terminal patterns, these patterns only represent one token

use crate::lexer::token::{Token, TokenType};
use crate::parser::{Pattern, LanguageItem, UnexpectedTokenError, PatternResult};
use crate::source::SfSlice;

macro_rules! single_token_pattern {
    ($type_name:ident, $name:expr, $match_pattern:pat) => {
        /// Newtype wrapper over a token of the type specified by it's name.
        /// The inner token it **guarentied** to be the same as the name implies.
        #[derive(Debug, Clone, PartialEq)]
        pub struct $type_name(
            pub Token,
        );

        impl LanguageItem for $type_name {
            fn slice(&self) -> SfSlice {
                self.0.slice.clone()
            }
        }

        impl Pattern for $type_name {
            fn solve(tokens: &[Token]) -> PatternResult<Self> {
                let Some(token) = tokens.get(0) else {
                    return Err(UnexpectedTokenError::new_got_nothing($name))
                };
                
                if let $match_pattern = token.t_type {
                    let result = $type_name(token.clone());
                    Ok((1, result))
                } else {
                    Err(UnexpectedTokenError::new($name, token.clone()))
                }
            }
            
            fn name() -> String { $name.to_string() }
        }
    };
}

single_token_pattern!(
    Ident,
    "ident",
    TokenType::Ident(_)
);

single_token_pattern!(
    NumLit,
    "numlit",
    TokenType::NumLit(_)
);

single_token_pattern!(
    CharLit,
    "charlit",
    TokenType::CharLit(_)
);

single_token_pattern!(
    StrLit,
    "strlit",
    TokenType::StrLit(_)
);

single_token_pattern!(
    Plus,
    "+",
    TokenType::Plus
);

single_token_pattern!(
    Minus,
    "-",
    TokenType::Minus
);

single_token_pattern!(
    Multiply,
    "*",
    TokenType::Star
);

single_token_pattern!(
    Divide,
    "/",
    TokenType::Slash
);

single_token_pattern!(
    Modulo,
    "%",
    TokenType::Modulo
);

single_token_pattern!(
    LogicalNot,
    "!",
    TokenType::ExclamationMark
);

single_token_pattern!(
    LogicalAnd,
    "&&",
    TokenType::LogicalAnd
);

single_token_pattern!(
    LogicalOr,
    "||",
    TokenType::LogicalOr
);

single_token_pattern!(
    LogicalEqual,
    "==",
    TokenType::LogicalEqual
);

single_token_pattern!(
    LogicalInequal,
    "!=",
    TokenType::LogicalInequal
);

single_token_pattern!(
    GreaterThan,
    ">",
    TokenType::GreaterThan
);

single_token_pattern!(
    GreaterThanEqual,
    ">=",
    TokenType::GreaterThanEqual
);

single_token_pattern!(
    LessThan,
    "<",
    TokenType::LessThan
);

single_token_pattern!(
    LessThanEqual,
    "<=",
    TokenType::LessThanEqual
);

single_token_pattern!(
    Semicolon,
    ";",
    TokenType::Semicolon
);

single_token_pattern!(
    Colon,
    ":",
    TokenType::Colon
);

single_token_pattern!(
    ThickArrow,
    "=>",
    TokenType::ThickArrow
);

single_token_pattern!(
    LeftSquare,
    "[",
    TokenType::LSquare
);

single_token_pattern!(
    RightSquare,
    "]",
    TokenType::RSquare
);

single_token_pattern!(
    LeftCurly,
    "{",
    TokenType::LCurly
);

single_token_pattern!(
    RightCurly,
    "}",
    TokenType::RCurly
);

single_token_pattern!(
    LeftParen,
    "(",
    TokenType::LParen
);

single_token_pattern!(
    RightParen,
    ")",
    TokenType::RParen
);

single_token_pattern!(
    Comma,
    ",",
    TokenType::Comma
);

single_token_pattern!(
    At,
    "@",
    TokenType::At
);

single_token_pattern!(
    Pound,
    "#",
    TokenType::Pound
);

single_token_pattern!(
    Eof,
    "eof",
    TokenType::Eof
);

// TODO: tests

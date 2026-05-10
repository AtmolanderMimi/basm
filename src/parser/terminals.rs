//! Implementation of terminal patterns, these patterns only represent one token

use crate::lexer::token::{Token, TokenType};
use crate::parser::{Pattern, LanguageItem, UnexpectedTokenError, PatternResult};
use crate::source::SfSlice;

macro_rules! single_token_pattern {
    ($result_name:ident, $pattern_name:ident, $match_pattern:pat, $default:expr) => {
        /// Newtype wrapper over a token of the type specified by it's name.
        /// The inner token it **guarentied** to be the same as the name implies.
        #[derive(Debug, Clone, PartialEq)]
        pub struct $result_name(
            pub Token,
        );

        impl LanguageItem for $result_name {
            fn slice(&self) -> SfSlice {
                self.0.slice.clone()
            }
        }

        /// A pattern for the creation of a structure which is only one token long.
        /// That struture is what the name implies. (Before the "Pattern")
        #[allow(unused)]
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct $pattern_name;

        impl Pattern for $pattern_name {
            type ParseResult = $result_name;
            fn solve(tokens: &[Token]) -> PatternResult<Self::ParseResult> {
                let Some(token) = tokens.get(0) else {
                    return Err(UnexpectedTokenError::new_got_nothing(vec![$default]))
                };

                if let $match_pattern = token.t_type {
                    let result = $result_name(token.clone());
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
    IdentPattern,
    TokenType::Ident(_),
    TokenType::Ident("any".to_string())
);

impl Ident {
    /// Returns the value of the inner ident.
    pub fn value(&self) -> &str {
        if let TokenType::Ident(s) = &self.0.t_type {
            s
        } else {
            panic!("ident struct is ident token type invariant")
        }
    }
}

single_token_pattern!(
    NumLit,
    NumLitPattern,
    TokenType::NumLit(_),
    TokenType::NumLit(732)
);

impl NumLit {
    /// Returns the value of the inner number literal.
    pub fn value(&self) -> u32 {
        if let TokenType::NumLit(n) = &self.0.t_type {
            *n
        } else {
            panic!("numlit struct is num lit token type invariant")
        }
    }
}

single_token_pattern!(
    CharLit,
    CharLitPattern,
    TokenType::CharLit(_),
    TokenType::CharLit('e')
);

impl CharLit {
    /// Returns the value of the inner character literal.
    pub fn value(&self) -> char {
        if let TokenType::CharLit(c) = &self.0.t_type {
            *c
        } else {
            panic!("charlit struct is char lit token type invariant")
        }
    }
}

single_token_pattern!(
    StrLit,
    StrLitPattern,
    TokenType::StrLit(_),
    TokenType::StrLit("any".to_string())
);

impl StrLit {
    /// Returns the value of the inner character literal.
    pub fn value(&self) -> &str {
        if let TokenType::StrLit(s) = &self.0.t_type {
            s
        } else {
            panic!("charlit struct is char lit token type invariant")
        }
    }
}

single_token_pattern!(
    Plus,
    PlusPattern,
    TokenType::Plus,
    TokenType::Plus
);

single_token_pattern!(
    Minus,
    MinusPattern,
    TokenType::Minus,
    TokenType::Minus
);

single_token_pattern!(
    Star,
    StarPattern,
    TokenType::Star,
    TokenType::Star
);

single_token_pattern!(
    Slash,
    SlashPattern,
    TokenType::Slash,
    TokenType::Slash
);

single_token_pattern!(
    Semicolon,
    SemicolonPattern,
    TokenType::Semicolon,
    TokenType::Semicolon
);

single_token_pattern!(
    LeftSquare,
    LeftSquarePattern,
    TokenType::LSquare,
    TokenType::LSquare
);

single_token_pattern!(
    RightSquare,
    RightSquarePattern,
    TokenType::RSquare,
    TokenType::RSquare
);

single_token_pattern!(
    At,
    AtPattern,
    TokenType::At,
    TokenType::At
);

single_token_pattern!(
    Eof,
    EofPattern,
    TokenType::Eof,
    TokenType::Eof
);

// TODO: tests

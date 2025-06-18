//! Tools used to parse a string of tokens into sensible a sensible structure (the [`ParsedProgram`] struct).

// because i want to share patterns, but users shouldn't have to use them directly
// so it is fine they can't because of private bounds.
#![allow(private_interfaces)]

mod terminals;
mod componants;
mod expression;
mod instruction;
mod scope;
mod fields;
mod meta_field;

use std::usize;

use componants::{Many, Then};
use fields::{Field, FieldPattern, SetupField};
use terminals::EofPattern;
use thiserror::Error;

use crate::{lexer::token::{Token, TokenType}, source::SfSlice, CompilerError, Lint};

#[allow(unused_imports)]
pub use terminals::{Ident, NumLit, CharLit, Plus, Minus, Semicolon, LeftSquare, RightSquare, At, MainIdent};
#[allow(unused_imports)]
pub use expression::{Expression, ValueRepresentation, Mod};
#[allow(unused_imports)]
pub use fields::MainField;
#[allow(unused_imports)]
pub use meta_field::{MetaField, SignatureArgument};
#[allow(unused_imports)]
pub use instruction::{Instruction, Argument};
#[allow(unused_imports)]
pub use scope::Scope;

/// Defines a language pattern.
#[allow(missing_docs, private_interfaces)]
pub trait Pattern: Default {
    type ParseResult: Clone;

    /// Advances a pattern.
    /// The patterns becomes invalid after returning `Done` or `NotExpected`.
    /// **Any calls of this method after this, are considered undefined behaviour.**
    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult>;
}

#[derive(Debug, Clone, PartialEq)]
enum AdvancementState<T> {
    Advancing,
    Done(T),
    Error(PatternMatchingError),
}

struct Advancement<T> {
    // The number of tokens that were not used to make a decision, but not included in the pattern.
    pub overeach: usize,
    pub state: AdvancementState<T>,
}

impl<T> Advancement<T> {
    pub fn new_no_overeach(state: AdvancementState<T>) -> Advancement<T> {
        Advancement {
            overeach: 0,
            state,
        }
    }

    pub fn new(state: AdvancementState<T>, overeach: usize) -> Advancement<T> {
        Advancement {
            overeach,
            state,
        }
    }
}

/// Error happening during the parsing process.
/// Language item parsers will throw this error
/// when encountering a pattern in the tokens that doesn't match their expectation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PatternMatchingError {
    /// A token was not expected, it is invalid for the pattern.
    #[error("expected {expected:?} token, got {got:?}")] // got got :3
    UnexpectedToken {
        /// The token that would have been valid.
        expected: TokenType,
        /// The token that was gotten.
        got: Token,
    },

    /// More than one main field was parsed, there can only be one main field per file.
    #[error("more than one main field was parsed; there can only be one main field per file")]
    MoreThanOneMain(MainField),

    /// More than one setup field was parsed, there can only be one setup field per file.
    #[error("more than one setup field was parsed, there can only be one setup field per file")]
    MoreThanOneSetup(SetupField),
}

impl CompilerError for PatternMatchingError {
    fn lint(&self) -> Option<crate::Lint> {
        let l = match self {
            Self::UnexpectedToken { got, .. } => {
                Lint::from_slice_error(got.slice.clone())
            },
            Self::MoreThanOneMain(m) => Lint::from_slice_error(m.slice()),
            Self::MoreThanOneSetup(s) => Lint::from_slice_error(s.slice()),
        };

        Some(l)
    }
}

/// Feeds a pattern with tokens, implements backtracking when overeaching in the tokens.
#[derive(Debug, Clone, PartialEq)]
struct PatternFeeder<'a, T: Pattern> {
    pattern: T,
    tokens: &'a [Token],
    current_token: usize,
}

impl<'a, T: Pattern> PatternFeeder<'a, T> {
    fn new(tokens: &'a [Token]) -> Self {
        PatternFeeder {
            pattern: T::default(),
            tokens,
            current_token: 0,
        }
    }

    // Advances by one token. Should be considered UB after getting Error or Done.
    fn advance(&mut self) -> Advancement<T::ParseResult> {
        let token = self.tokens.get(self.current_token)
            .expect("patterns should end before running out of tokens");

        let adv = self.pattern.advance(token);
        self.current_token += 1;
        self.current_token -= adv.overeach;

        adv
    }
}

/// A function that solves a string of [`Token`] via a pattern.
pub fn solve_pattern<T: Pattern>(tokens: &[Token]) -> Result<T::ParseResult, PatternMatchingError> {
    let mut feeder: PatternFeeder<'_, T> = PatternFeeder::new(tokens);

    loop {
        match feeder.advance().state {
            AdvancementState::Advancing => (),
            AdvancementState::Done(res) => return Ok(res),
            AdvancementState::Error(e) => return Err(e),
        }
    }
}

/// Pattern for constructing a [`ParsedProgram`].
#[derive(Debug, Clone, PartialEq, Default)]
struct FilePattern(
    Then<Many<FieldPattern>, EofPattern>
);

/// A whole, parsed, basm file.
/// A basm file contain 0 or 1 [`MainField`] and [`SetupField`],
/// but it can be augmented by any one or more [`MetaField`].
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFile {
    #[allow(missing_docs)]
    pub meta_instructions: Vec<MetaField>,
    #[allow(missing_docs)]
    // main may be none, if the file is a library file
    pub main_field: Option<MainField>,
    #[allow(missing_docs)]
    pub setup_field: Option<SetupField>,
}

impl Pattern for FilePattern {
    type ParseResult = ParsedFile;

    fn advance(&mut self, token: &Token) -> Advancement<Self::ParseResult> {
        use AdvancementState as AdvState;

        let adv = self.0.advance(token);
        let overeach = adv.overeach;

        match adv.state {
            AdvState::Advancing => Advancement::new(AdvState::Advancing, overeach),
            AdvState::Done(res) => {
                let mut fields = res.0;
                let main_index = fields.iter().enumerate()
                    .find(|(_, f)| f.is_main())
                    .map(|(i, _)| i);
                let main = if let Some(main_index) = main_index {
                    Some(fields.remove(main_index).unwrap_main().unwrap())
                } else {
                    None
                };

                let setup_index = fields.iter().enumerate()
                    .find(|(_, f)| f.is_setup())
                    .map(|(i, _)| i);
                let setup = if let Some(setup_index) = setup_index {
                    Some(fields.remove(setup_index).unwrap_setup().unwrap())
                } else {
                    None
                };

                // check for extra unwanted fields, which are not metas,
                // at this point only metas should be left in the iterator
                let maybe_unwanted = fields.iter().find(|f| !f.is_meta());
                if let Some(unwanted) = maybe_unwanted {
                    let err = match unwanted {
                        Field::Main(m) => PatternMatchingError::MoreThanOneMain(m.clone()),
                        Field::Setup(s) => PatternMatchingError::MoreThanOneSetup(s.clone()),
                        _ => panic!()
                    };

                    // TODO: fix this to use the overeach of the Many<> 
                    return Advancement::new(AdvState::Error(err), 0);
                }

                let metas = fields.into_iter().map(|f| f.unwrap_meta().unwrap());

                let val = ParsedFile {
                    meta_instructions: Vec::from_iter(metas),
                    main_field: main,
                    setup_field: setup,
                };

                Advancement::new(AdvState::Done(val), overeach)
            },
            AdvState::Error(e) => Advancement::new(AdvState::Error(e), overeach),
        }
    }
}

/// A generic trait to be implemented onto each language item.
pub trait LanguageItem {
    /// A slice defining the position of the language item.
    fn slice(&self) -> SfSlice;
}

impl LanguageItem for Token {
    fn slice(&self) -> SfSlice {
        self.slice.clone()
    }
}

#[macro_export]
/// Used to implement the [`LanguageItem`] trait's `slice()` method in a generic method.
/// Creates an implementation which creates a slice from the start of `start` to the end of `end`. 
macro_rules! impl_language_item {
    ($type:ty, $start:ident, $end:ident) => {
        impl LanguageItem for $type {
            fn slice(&self) -> SfSlice {
                let start_slice = self.$start.slice();
                let start = start_slice.start();
                let end = self.$end.slice().end();
                
                start_slice.source().slice(start..end)
                .unwrap()
            }
        }
    };
}

/// Parses the tokens into a structured form ([`ParsedProgram`]).
pub fn parse_tokens(tokens: &[Token]) -> Result<ParsedFile, PatternMatchingError> {
    solve_pattern::<FilePattern>(tokens)
}

/// The collection of patterns used to parse for structures.
/// (At least all the patterns for the structures which are public)
pub mod patterns {
    pub use super::{solve_pattern, Pattern};

    use crate::parser::terminals;
    use crate::parser::componants;
    use crate::parser::expression;
    use crate::parser::instruction;
    use crate::parser::scope;
    use crate::parser::fields;
    use crate::parser::meta_field;

    pub use terminals::{AtPattern, EofPattern, StarPattern, IdentPattern, MinusPattern, NumLitPattern, StrLitPattern, CharLitPattern, MainIdentPattern, SemicolonPattern, LeftSquarePattern, RightSquarePattern};
    pub use componants::{Or, Then, Many};
    pub use expression::ExpressionPattern;
    pub use instruction::{ArgumentPattern, ScopeIdentPattern, InstructionPattern};
    pub use scope::ScopePattern;
    pub use fields::{FieldPattern, MainFieldPattern, SetupFieldPattern};
    pub use meta_field::{MetaFieldPattern, SignatureArgumentPattern};
}

#[cfg(test)]
mod tests {
    use std::path::absolute;

    use crate::{lex_file, source::{SfSlice, SourceFile}};

    use super::*;

    fn bogus_token(t_type: TokenType) -> Token {
        Token::new(t_type, SfSlice::new_bogus("fishg"))
    }

    #[test]
    fn parsed_file_token_pattern() {
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

        let res = solve_pattern::<FilePattern>(&tokens).unwrap();
        assert_eq!(res.meta_instructions.len(), 1);

        // no [main]
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

        let res = solve_pattern::<FilePattern>(&tokens);
        assert!(res.unwrap().main_field.is_none());

        // no meta
        let tokens = vec![
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

        let res = solve_pattern::<FilePattern>(&tokens).unwrap();
        assert_eq!(res.meta_instructions.len(), 0);
    }

    #[test]
    fn parsing_fib_example() {
        let abs_path = absolute("./test-resources/fib.basm").unwrap();
        let file = SourceFile::from_file(&abs_path).unwrap().leak();
        let res = lex_file(&file);
        let tokens = res.unwrap();
        let program = solve_pattern::<FilePattern>(&tokens).unwrap();

        assert_eq!(program.meta_instructions.len(), 1);
        assert_eq!(program.main_field.unwrap().contents.contents.len(), 7);
    }
}

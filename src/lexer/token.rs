//! Defines what a syntactic [`Token`] is and how to parse substrings for it.

use std::{num::IntErrorKind, ops::Range};

use crate::{source::SfSlice, utils::{Sliceable, IsAlphanumeric}};

use super::LiteralError;

#[derive(Debug, Clone, PartialEq)]
/// A syntactic token
pub struct Token {
    /// The type of that token. Aka "What's that?".
    pub t_type: TokenType,
    /// The slice of the token. Should include the whole token and only the token.
    pub slice: SfSlice,
}

impl Token {
    /// Creates a new [`Token`].
    pub fn new(t_type: TokenType, slice: SfSlice) -> Token {
        Token {
            t_type,
            slice,
        }
    }

    /// Returns the range in bytes that contains this token.
    pub fn range(&self) -> Range<usize> {
        self.slice.range()
    }
}

/// Syntactic tokens types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// "72", an Num literal.
    NumLit(u32),
    /// '"Hello, World"', a Str literal.
    StrLit(String),
    /// "'c'", a Char literal.
    CharLit(char),
    /// "+", used to offset values.
    Plus,
    /// "-", used to offset values.
    Minus,
    /// "[", an opening square bracket, many uses.
    LSquare,
    /// "]", an closing square bracket, many uses.
    RSquare,
    /// "@", used to declare the signature of a meta-instruction.
    At,
    /// ";", delimits instructions.
    InstructionDelimitor,
    /// "//", starts a comment on the rest of the line.
    /// Is only used by the lexer to avoid comments.
    /// This will not be found in the AST.
    LineComment,
    /// Any alphanumeric squence that starts with a letter and
    /// is not any other token.
    Ident(String),
    /// End of file identificator. Signifies the end of the token string.
    Eof,
}

impl TokenType {
    const MAPPING: &'static [(&'static str, TokenType)] = &[
        ("+", Self::Plus),
        ("-", Self::Minus),
        ("[", Self::LSquare),
        ("]", Self::RSquare),
        ("@", Self::At),
        ("//", Self::LineComment),
        (";", Self::InstructionDelimitor),
        // lits go here also idents in spirit, cus they can't be mapped like this
    ];

    #[cfg(debug_assertions)]
    /// function just to remember to update other functions if a token is added
    /// please add above to MAPPING ↑
    fn _exhaustive(&self) {
        #[allow(clippy::pedantic)]
        match self {
            Self::At => (),
            Self::CharLit(_) => (),
            Self::LineComment => (),
            Self::Ident(_) => (),
            Self::LSquare => (),
            Self::Minus => (),
            Self::NumLit(_) => (),
            Self::Plus => (),
            Self::RSquare => (),
            Self::StrLit(_) => (),
            Self::InstructionDelimitor => (),
            Self::Eof => (),
        }
    }
}

impl<'a> Token {
    // beware, trash ahead. also:
    //   /---\  /-----\     /---|_ _|/--\|-|
    //   \    \/     O \    |   /| | |  /| |
    //    ->            |   |  > | | \ < | |
    //   / -  /\---    /    | |  | |  > ||  -
    //   \---/  \-----/     |_| |_ _|< _/| | |
    /// Returns a [`Token`] and it's position in the string, **EXCEPT FOR IDENTS/LITS**
    /// The token's range postion is absolute.
    pub fn parse_token_non_lit(sf_slice: &SfSlice) -> Option<Token> {
        let slice = sf_slice.inner_slice();
        let trim_slice = slice.trim();

        // don't check for tokens if we are in a string
        let in_string = slice.chars().filter(|c| *c == '"').count() % 2 == 1 || slice.ends_with('"');
        let in_char = slice.chars().filter(|c| *c == '\'').count() % 2 == 1 || trim_slice.starts_with('\'');
        if in_string || in_char {
            return None;
        }

        let mut matches = vec![];

        for pair in TokenType::MAPPING {
            if let Some(i) = slice.find(pair.0) {
                if !pair.0.is_alphanumeric() {
                    matches.push((i..(i+pair.0.len()), pair.1.clone()));
                }

                // transform to chars
                let i = slice.char_indices().enumerate().find_map(|(idx, (jdx, _))| {
                    if i == jdx {
                        Some(idx)
                    } else {
                        None
                    }
                }).expect("\"i\" should be on valid character boundary");

                let before_char_index = i.checked_sub(1);
                let before_char = if let Some(i_before) = before_char_index {
                    slice.chars().nth(i_before)
                        .expect("There should be chars before the char after them")
                } else {
                    ' '
                };
                let before_char_is_alphanumeric = before_char.is_alphanumeric() ||
                    before_char == '_';

                let after_char_index = i + pair.0.len();
                let after_char = slice.chars().nth(after_char_index);
                let after_char_is_alphanumeric = after_char.is_none_or(|c| {
                    c.is_alphanumeric() || c == '_'
                });

                let neighbors_are_alphanumeric = before_char_is_alphanumeric ||
                    after_char_is_alphanumeric;
                if !neighbors_are_alphanumeric {
                    matches.push((i..(i+pair.0.len()), pair.1.clone()));
                }
            }
        }

        // sorts by acending order, breaks ties
        matches.sort_by(|mat1, mat2| {
            mat2.0.start.cmp(&mat1.0.start)
        });
        
        let matched_token = matches.into_iter().filter(|m| {
            let higher_level_matches = TokenType::MAPPING.iter()
                    // checks mappings that contain this mapping
                    .filter(|pair2| {
                        let m_str = slice.get(m.0.clone()).expect("Should always be valid");
                        pair2.0.contains(m_str)
                    })
                    // gets the lenght
                    .map(|pair2| pair2.0.len())
                    // checks if
                    .filter(|higher_m| {
                        let match_diff = higher_m - m.0.len();
                        let distance_from_end = slice.len() as i32 - m.0.end as i32;

                        distance_from_end < match_diff as i32
                    })
                    .collect::<Vec<_>>();
            higher_level_matches.is_empty()
        })
        .nth(0).map(move |inner| {
            let inner = inner.clone();
            let char_slice = sf_slice.slice(inner.0)
                .unwrap();

            Token::new(inner.1, char_slice)
        });

        matched_token
    }

    /// Parses a string for a literal/ident. Note this should only be used on a
    /// string that contains a **FULL LITERAL/IDENT AND NOTHING ELSE**, because it might
    /// detect alphanumeric tokens (ex: "let") as idents.
    /// The tokens' range positions are absolute.
    pub fn parse_token_lit(sf_slice: &SfSlice) -> Result<Option<Token>, LiteralError> {
        let string = sf_slice.inner_slice();
        let trim_str = string.trim();
        let trim_str_start = string.find(trim_str)
            .expect("trim_str is just a substring");

        // Str
        let trim_str_range = trim_str_start..(trim_str_start+trim_str.len());
        if trim_str.starts_with('\"') && trim_str.ends_with('\"') {
            let string_contents = trim_str.replace('\"', "");
            let string_contents = string_contents.replace("\\n", "\n");

            let slice = sf_slice.slice(trim_str_range).unwrap();
            return Ok(Some(Token::new(TokenType::StrLit(string_contents.to_string()), slice)));
        }

        // Char
        if trim_str.starts_with('\'') && trim_str.ends_with('\'') && trim_str.len() > 1 {
            let char_content = trim_str.replace('\'', "");
            let char_content = char_content.replace("\\n", "\n");

            if char_content.is_empty() {
                let error_slice = sf_slice.slice(trim_str_range)
                    .expect("byte slice should not be oob");
                return Err(LiteralError::EmptyChar(error_slice));
            }
            if char_content.len() >= 2 {
                let err_slice = sf_slice.slice(trim_str_range)
                        .unwrap();
                return Err(LiteralError::TooFullChar(err_slice))
            }

            let ch = char_content.chars().next()
                .expect("the checks should have caught that we have at least one char");
            let slice = sf_slice.slice(trim_str_range)
                .unwrap();
            return Ok(Some(Token::new(TokenType::CharLit(ch), slice)));
        }

        // Num
        if trim_str.is_numeric() {
            let res = trim_str.parse::<u32>();

            let num = match res {
                Ok(n) => n,
                Err(parse_error) => {
                    match parse_error.kind() {
                        IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                            let err_slice = sf_slice.slice(trim_str_range)
                                .unwrap();
                            return Err(LiteralError::InvalidNumber(err_slice))
                        },
                        _ => panic!("number {trim_str} should have been valid")
                    }
                }
            };

            return Ok(Some(Token::new(
                TokenType::NumLit(num),
                sf_slice.slice(trim_str_range).unwrap(),
            )));
        }

        // If none of the above, try Ident
        let trim_str_without_under = trim_str.replace('_', "");
        if trim_str_without_under.is_alphanumeric() {
            return Ok(Some(Token::new(
                TokenType::Ident(trim_str.to_string()),
                sf_slice.slice(trim_str_range).unwrap(),
            )));
        }
        
        if !trim_str.is_empty() {
            let err_slice = sf_slice.slice(trim_str_range)
                .unwrap();
            return Err(LiteralError::Unparseable(err_slice));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    fn non_lit_match(token: Option<Token>, expected_t_type: Option<TokenType>) {
        match expected_t_type {
            // linting is broken

            Some(expected_t_type) => {
                if let Some(Token { t_type: t, ..}) = token {
                    assert_eq!(t, expected_t_type);
                }
            }
            None => assert_matches!(
                token,
                None,
            ),
        }
    }

    fn non_lit_match_range(token: Option<Token>, expected_t_type: TokenType, expected_range: Range<usize>) {
        non_lit_match(token.clone(), Some(expected_t_type));

        assert_eq!(token.unwrap().range(), expected_range);
    }

    fn lit_match(token: &Result<Option<Token>, LiteralError>, expected_t_type: Option<TokenType>) {
        match expected_t_type {
            Some(expected_t_type) => {
                if let Ok(Some(Token { t_type: t, ..})) = token {
                    assert_eq!(*t, expected_t_type);
                }
            },
            None => assert_matches!(
                token,
                Ok(None),
            ),
        }
    }

    fn lit_match_range(token: Result<Option<Token>, LiteralError>, expected_t_type: TokenType, expected_range: Range<usize>) {
        lit_match(&token, Some(expected_t_type));
        assert_eq!(token.unwrap().unwrap().range(), expected_range);
    }

    /// new SfSlice, but shorter name
    fn sfs(contents: &str) -> SfSlice {
        SfSlice::new_bogus(contents)
    }

    #[test]
    fn parse_token_non_lit_nothing() {
        assert!(Token::parse_token_non_lit(&sfs("")).is_none());
    }

    #[test]
    fn parse_token_non_lit_alphanumeric_tokens_not_found_in_ident() {
        assert!(Token::parse_token_non_lit(&sfs("while_condition")).is_none()); // while
        assert!(Token::parse_token_non_lit(&sfs("pinchifly")).is_none()); // if
        assert!(Token::parse_token_non_lit(&sfs("outlet")).is_none()); // let
    }

    #[test]
    fn parse_token_non_lit_real_world() {
        // cause we can't know if it is ident
        non_lit_match(
            Token::parse_token_non_lit(&sfs("\n    let")),
            None
        );

        non_lit_match_range(
            Token::parse_token_non_lit(&sfs(" n -")),
            TokenType::Minus,
            3..4,
        );
        non_lit_match(
            Token::parse_token_non_lit(&sfs(" &")),
            None
        );

        // NOTE: parsing is not guarentied to be in order anymore (currently is in last first)
        //// parses are in order
        //non_lit_match_range(
        //    Token::parse_token_non_lit(&sfs("@ + - //")),
        //     TokenType::At, 0..1
        //);
    }

    #[test]
    fn parse_token_lit_nothing() {
        assert_eq!(
            Token::parse_token_lit(&sfs("")),
            Ok(None),
        );
        assert_eq!(
            Token::parse_token_lit(&sfs("\n\n    \n ")),
            Ok(None),
        );
    }

    #[test]
    fn parse_token_lit_str() {
        lit_match_range(
            Token::parse_token_lit(&sfs("\n\"Hello, World!\"")),
            TokenType::StrLit("Hello, World!".to_string()),
            1..16
        );
        lit_match_range(
            Token::parse_token_lit(&sfs(" \"\"")),
            TokenType::StrLit("".to_string()),
            1..3
        );
        lit_match_range(
            Token::parse_token_lit(&sfs("\"Sfdsfa339472evm weoi 03d \"")),
            TokenType::StrLit("Sfdsfa339472evm weoi 03d ".to_string()),
            0..27
        );
    }

    #[test]
    fn parse_token_lit_char() {
        lit_match_range(
            Token::parse_token_lit(&sfs("\n'c' ")),
            TokenType::CharLit('c'),
            1..4
        );

        // NOTE: This does not work, because à, is actually treated as two different characters
        // in a row in rust. Like this: `a. Because of this, FsSlice does not match with
        // most IDE's character position.
        //lit_match_range(
        //    Token::parse_token_lit(sfs(" 'à'")),
        //    TokenType::CharLit('à'),
        //    1..4
        //);

        let res = Token::parse_token_lit(&sfs("''"));
        if let Err(LiteralError::EmptyChar(_)) = res {
            // aight ok
        } else {
            panic!("{:?} is wrong error or not error", res)
        }


        let res = Token::parse_token_lit(&sfs("'kiddie kindson corp!!!'  "));
        if let Err(LiteralError::TooFullChar( .. )) = res {
            // aight ok
        } else {
            panic!("{:?} is wrong error or not error", res)
        }
    }

    #[test]
    fn parse_token_lit_num() {
        lit_match_range(
            Token::parse_token_lit(&sfs("\n72 ")),
            TokenType::NumLit(72),
            1..3
        );
        lit_match_range(
            Token::parse_token_lit(&sfs(" 142 \n")),
            TokenType::NumLit(142),
            1..4
        );
    }
}
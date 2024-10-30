use std::{num::{IntErrorKind, ParseIntError}, ops::Range};

use crate::{error::Lint, source::SfSlice, utils::{CharOps, IsAlphanumeric}, Num};

use super::LiteralError;

#[derive(Debug, Clone, PartialEq)]
/// A syntactic token
pub struct Token {
    pub t_type: TokenType,
    pub char_range: Range<usize>,
}

impl Token {
    /// Creates a new [Token].
    pub fn new(t_type: TokenType, char_range: Range<usize>) -> Token {
        Token {
            t_type,
            char_range,
        }
    }
}

/// Syntactic tokens types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    /// "fn", function declaration.
    FnDecl,
    /// "let", variable declaration.
    VarDecl,
    /// "[Ident]: [type Ident]", type declaration.
    /// Can also be used to signify the return type of a function.
    TypeDecl,
    /// "+", adds left and right.
    Plus,
    /// "-", substracts right from left.
    Minus,
    /// "*", multiply left and right.
    Multiply,
    /// "%", remainder of the division of left by right. ex: 5 % 2 = 1
    Modulo,
    /// "==", true if left and right are equal.
    Equals,
    /// "!=", true if left and right are not equal.
    NotEquals,
    /// ">", true if left is bigger than right.
    BiggerThan,
    /// "<", true if left is smaller than right.
    SmallerThan,
    /// ">=", true if left is bigger or equal to right.
    BiggerThanEqual,
    /// "<=", true if left is smaller or equal to right.
    SmallerThanEqual,
    /// "||", true if left or right is true.
    Or,
    /// "&&", true if left and right are true.
    And,
    /// "=", the assignment operator.
    AssignOp,
    /// "&", the clone operator.
    CloneOp,
    /// ";", the delimiter between statements.
    StatementDelimiter,
    /// ",", the seperator between elements in an array.
    ElementSeperator,
    /// "if", an if branch.
    If,
    /// "while", a while loop.
    While,
    /// "72", an Num literal.
    NumLit(Num),
    /// '"Hello, World"', a Str literal.
    StrLit(String),
    /// "'c'", a Char literal.
    CharLit(char),
    /// "true"/"false", a Bool literal.
    BoolLit(bool),
    /// "[", an opening square bracket, many uses.
    LSquare,
    /// "]", an closing square bracket, many uses.
    RSquare,
    /// "{", an opening curly bracket, many uses.
    LCurly,
    /// "}", an closing curly bracket, many uses.
    RCurly,
    /// "(", an opening parentheses, many uses.
    LParen,
    /// ")", an closing parentheses, many uses.
    RParen,
    /// Any alphanumeric squence that starts with a letter and
    /// is not any other token.
    Ident(String),
}

impl TokenType {
    pub const MAPPING: &'static [(&'static str, TokenType)] = &[
        ("fn", Self::FnDecl),
        ("let", Self::VarDecl),
        (":", Self::TypeDecl),
        ("+", Self::Plus),
        ("-", Self::Minus),
        ("*", Self::Multiply),
        ("%", Self::Modulo),
        ("==", Self::Equals),
        ("!=", Self::NotEquals),
        (">", Self::BiggerThan),
        ("<", Self::SmallerThan),
        (">=", Self::BiggerThanEqual),
        ("<=", Self::SmallerThanEqual),
        ("||", Self::Or),
        ("&&", Self::And),
        ("=", Self::AssignOp),
        ("&", Self::CloneOp),
        (";", Self::StatementDelimiter),
        (",", Self::ElementSeperator),
        ("if", Self::If),
        ("while", Self::While),
        // lits go here also idents in spirit, cus they can't be mapped like this
        ("[", Self::LSquare),
        ("]", Self::RSquare),
        ("{", Self::LCurly),
        ("}", Self::RCurly),
        ("(", Self::LParen),
        (")", Self::RParen),
    ];

    #[cfg(debug_assertions)]
    /// function just to remember to update other function if a token is added
    /// please add above to MAPPING ↑
    fn exhaustive(&self) {
        match self {
            Self::FnDecl => (),
            Self::VarDecl => (),
            Self::AssignOp => (),
            Self::BiggerThan => (),
            Self::BiggerThanEqual => (),
            Self::BoolLit(_) => (),
            Self::CharLit(_) => (),
            Self::CloneOp => (),
            Self::ElementSeperator => (),
            Self::Equals => (),
            Self::Ident(_) => (),
            Self::If => (),
            Self::LCurly => (),
            Self::LParen => (),
            Self::LSquare => (),
            Self::Minus => (),
            Self::Modulo => (),
            Self::Multiply => (),
            Self::NotEquals => (),
            Self::NumLit(_) => (),
            Self::Plus => (),
            Self::RCurly => (),
            Self::RParen => (),
            Self::RSquare => (),
            Self::SmallerThan => (),
            Self::SmallerThanEqual => (),
            Self::StatementDelimiter => (),
            Self::StrLit(_) => (),
            Self::TypeDecl => (),
            Self::While => (),
            Self::And => (),
            Self::Or => (),
        }
    }
}

impl Token {
    // beware, trash ahead. also:
    //   /---\  /-----\     /---|_ _|/--\|-|
    //   \    \/     O \    |   /| | |  /| |
    //    ->            |   |  > | | \ < | |
    //   / -  /\---    /    | |  | |  > ||  -
    //   \---/  \-----/     |_| |_ _|< _/| | |
    /// Returns a [Token] and it's position in the string, **EXCEPT FOR IDENTS/LITS**
    /// The token's range postion is absolute.
    pub fn parse_token_non_lit(sf_slice: SfSlice) -> Option<Token> {
        let slice = sf_slice.inner_slice();
        let mut matches = vec![];

        for pair in TokenType::MAPPING {
            if let Some(i) = slice.find(pair.0) {
                if !pair.0.is_alphanumeric() {
                    matches.push((i..(i+pair.0.len()), pair.1.clone()));
                }

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
                let after_char_is_alphanumeric = after_char.map_or(true, |c| {
                    c.is_alphanumeric() || c == '_'
                });

                let neighbors_are_alphanumeric = before_char_is_alphanumeric ||
                    after_char_is_alphanumeric;
                if !neighbors_are_alphanumeric {
                    matches.push((i..(i+pair.0.len()), pair.1.clone()));
                } else {
                    continue;
                }
            } else {
                continue;
            }

        }

        // sorts by descending order
        matches.sort_by(|mat1, mat2| {
            mat2.0.end.cmp(&mat1.0.end)
        });
        // sorts by acending order, breaks ties
        matches.sort_by(|mat1, mat2| {
            mat1.0.start.cmp(&mat2.0.start)
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
            higher_level_matches.len() < 1
        })
        .nth(0).map_or(None, |inner| {
            let inner = inner.clone();
            Some(Token::new(inner.1, inner.0))
        });

        matched_token.map(|mut t| {
            // the tokens are actually in byte range, so we have to return them to char range
            t.char_range = slice.byte_to_char_range(t.char_range)
                .expect("slice should be valid");
            // and then we offset them so that they are relative to the source file rather than
            // the slice
            t.char_range = (t.char_range.start+sf_slice.offset())..(t.char_range.end+sf_slice.offset());
            t
        })
    }

    /// Parses a string for a literal/ident. Note this should only be used on a
    /// string that contains a **FULL LITERAL/IDENT AND NOTHING ELSE**, because it might
    /// detect alphanumeric tokens (ex: "let") as idents.
    /// The tokens' range positions are absolute.
    pub fn parse_token_lit<'a>(sf_slice: SfSlice<'a>) -> Result<Option<Token>, LiteralError<'a>> {
        let string = sf_slice.inner_slice();
        let trim_str = string.trim();
        let trim_str_start = string.find(trim_str)
            .expect("trim_str is just a substring");

        // Str
        let trim_str_range = trim_str_start..(trim_str_start+trim_str.len());
        if trim_str.starts_with("\"") && trim_str.ends_with("\"") {
            let string_contents = trim_str.replace("\"", "");

            return Ok(Some(Token::new(TokenType::StrLit(string_contents.to_string()), trim_str_range)));
        }

        // Char
        if trim_str.starts_with("'") && trim_str.ends_with("'") {
            let char_content = trim_str.replace("'", "");
            if char_content.len() == 0 {
                let error_slice = sf_slice.byte_slice(trim_str_range)
                    .expect("byte slice should not be oob");
                return Err(LiteralError::EmptyChar(error_slice.to_owned()));
            }
            if char_content.len() >= 2 {
                return Err(LiteralError::TooFullChar(
                    // FIXME: would be better if these lints were absolute, probably need metadata
                    // about the substring and it's provenance
                    todo!(), //Lint::new_error(trim_str_range),
                    char_content,
            ))
            }

            let ch = char_content.chars().next()
                .expect("the checks should have caught that we have at least one char");
            return Ok(Some(Token::new(TokenType::CharLit(ch), trim_str_range)));
            //todo!("char lits");
        }

        // Bool
        if trim_str == "true" {
            return Ok(Some(Token::new(TokenType::BoolLit(true), trim_str_range)));
        }
        if trim_str == "false" {
            return Ok(Some(Token::new(TokenType::BoolLit(false), trim_str_range)));
        }

        // Num
        if trim_str.is_numeric() {
            let res = trim_str.parse::<Num>();

            let num = match res {
                Ok(n) => n,
                Err(parse_error) => {
                    match parse_error.kind() {
                        IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                            return Err(LiteralError::InvalidNumber(
                                
                                todo!("clear this up"),//Lint::new_error(trim_str_range),
                                todo!("clear this up")//trim_str.to_string(),
                            ))
                        },
                        _ => panic!("number {trim_str} should have been valid")
                    }
                }
            };

            return Ok(Some(Token::new(
                TokenType::NumLit(num),
                trim_str_range,
            )));
        }

        // If none of the above, try Ident
        if trim_str.is_alphanumeric() {
            return Ok(Some(Token::new(
                TokenType::Ident(trim_str.to_string()),
                trim_str_range,
            )));
        }
        
        if trim_str.len() != 0 {
            todo!("add failed to parse token error")
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// new SfSlice, but shorter name
    fn sfs(contents: &str) -> SfSlice<'static> {
        SfSlice::new_bogus(contents)
    }

    #[test]
    fn parse_token_non_lit_nothing() {
        assert!(Token::parse_token_non_lit(sfs("")).is_none());
    }

    #[test]
    fn parse_token_non_lit_unknowable_doesnt_get_recognised() {
        assert_eq!(Token::parse_token_non_lit(sfs("whilebogus=")), None);
        assert_eq!(Token::parse_token_non_lit(sfs("one==")).unwrap().t_type, TokenType::Equals);
        assert_eq!(Token::parse_token_non_lit(sfs("two= ==")).unwrap().t_type, TokenType::AssignOp);
    }

    #[test]
    fn parse_token_non_lit_alphanumeric_tokens_not_found_in_ident() {
        assert!(Token::parse_token_non_lit(sfs("while_condition")).is_none()); // while
        assert!(Token::parse_token_non_lit(sfs("pinchifly")).is_none()); // if
        assert!(Token::parse_token_non_lit(sfs("outlet")).is_none()); // let
    }

    #[test]
    fn parse_token_non_lit_real_world() {
        // cause we can't know if it is ident
        assert_eq!(Token::parse_token_non_lit(sfs("\n    let")),
            None);
        // now we can guarenty that this is the VarDecl token
        assert_eq!(Token::parse_token_non_lit(sfs("\n    let ")),
            Some(Token::new(TokenType::VarDecl, 5..8)));

        // no need to look farther ":" can only be TypeDecl, we could then
        // parse out the rest of the string for ident
        assert_eq!(Token::parse_token_non_lit(sfs(" a:Num")),
            Some(Token::new(TokenType::TypeDecl, 2..3)));

        assert_eq!(Token::parse_token_non_lit(sfs("while ")),
            Some(Token::new(TokenType::While, 0..5)));
        assert_eq!(Token::parse_token_non_lit(sfs(" n -")),
            Some(Token::new(TokenType::Minus, 3..4)));
        assert_eq!(Token::parse_token_non_lit(sfs(" 1;")),
            Some(Token::new(TokenType::StatementDelimiter, 2..3)));

        assert_eq!(Token::parse_token_non_lit(sfs(" &")),
            None);
        assert_eq!(Token::parse_token_non_lit(sfs(" &b")),
            Some(Token::new(TokenType::CloneOp, 1..2)));
        assert_eq!(Token::parse_token_non_lit(sfs(" &&")),
            Some(Token::new(TokenType::And, 1..3)));

        // parses are in order
        assert_eq!(Token::parse_token_non_lit(sfs("
    while n != 0 {
        // &b: \"the value of\" b
        // by default all operations are destructive
        // including moves, because bf
        let c: Num = &b;
        b = a + b; // here these destroy both a and b making them invalid
        a = c;

        // maybe i can switch this to \"n--\" to simplify to the translating
        n = n - 1;
    }")), Some(Token::new(TokenType::While, 5..10)))
    }

    #[test]
    fn parse_token_lit_nothing() {
        assert_eq!(
            Token::parse_token_lit(sfs("")),
            Ok(None),
        );
        assert_eq!(
            Token::parse_token_lit(sfs("\n\n    \n ")),
            Ok(None),
        );
    }

    #[test]
    fn parse_token_lit_bool() {
        assert_eq!(
            Token::parse_token_lit(sfs("\n  \n\ntrue   ")),
            Ok(Some(Token::new(TokenType::BoolLit(true), 5..9))),
        );
        assert_eq!(
            Token::parse_token_lit(sfs("\n \n false")),
            Ok(Some(Token::new(TokenType::BoolLit(false), 4..9))),
        );

        assert_eq!(
            Token::parse_token_lit(sfs("\n \n afalsethingamabob")),
            Ok(Some(Token::new(TokenType::Ident("afalsethingamabob".to_string()), 4..21))),
        );
    }

    #[test]
    fn parse_token_lit_str() {
        assert_eq!(
            Token::parse_token_lit(sfs("\n\"Hello, World!\" ")),
            Ok(Some(Token::new(TokenType::StrLit("Hello, World!".to_string()), 1..16))),
        );
        assert_eq!(
            Token::parse_token_lit(sfs(" \"\"")),
            Ok(Some(Token::new(TokenType::StrLit("".to_string()), 1..3))),
        );

        assert_eq!(
            Token::parse_token_lit(sfs("\"Sfdsfa339472evm weoi 03d \"")),
            Ok(Some(Token::new(TokenType::StrLit("Sfdsfa339472evm weoi 03d ".to_string()), 0..27))),
        );
    }

    #[test]
    fn parse_token_lit_char() {
        assert_eq!(
            Token::parse_token_lit(sfs("\n'c' ")),
            Ok(Some(Token::new(TokenType::CharLit('c'), 1..4))),
        );
        assert_eq!(
            Token::parse_token_lit(sfs(" 'à'")),
            Ok(Some(Token::new(TokenType::CharLit('à'), 1..4))),
        );

        let res = Token::parse_token_lit(sfs("''"));
        if let Err(LiteralError::EmptyChar(_)) = res {
            // aight ok
        } else {
            panic!("{:?} is wrong error or not error", res)
        }


        let res = Token::parse_token_lit(sfs("'kiddie kindson corp!!!'  "));
        if let Err(LiteralError::TooFullChar( .. )) = res {
            // aight ok
        } else {
            panic!("{:?} is wrong error or not error", res)
        }
    }

    #[test]
    fn parse_token_lit_num() {
        assert_eq!(
            Token::parse_token_lit(sfs("\n72 ")),
            Ok(Some(Token::new(TokenType::NumLit(72), 1..3))),
        );
        assert_eq!(
            Token::parse_token_lit(sfs(" 142 \n\0")),
            Ok(Some(Token::new(TokenType::NumLit(142), 1..4))),
        );

        assert_eq!(
            Token::parse_token_lit(sfs("\n7a2 ")),
            Ok(Some(Token::new(TokenType::NumLit(72), 1..3))),
        );

        let res = Token::parse_token_lit(sfs("\n7142 "));
        if let Err(LiteralError::InvalidNumber(..)) = res {
            // aight ok
        } else {
            panic!("{:?} is wrong error or not error", res)
        }
    }
}
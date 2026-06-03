//! Turns strings with escape sequences (e.g: \n, \", \', \\) into their actual string value

use either::Either;

use crate::{compiler::{expression::ExpressionEvaluationError}, parser::{CharLit, LanguageItem, StrLit}};
const ESCAPE_SEQUENCES: &[(char, char)] = &[
    ('\\', '\\'),
    ('n', '\n'),
    ('t', '\t'),
    ('\"', '\"'),
    ('\'', '\''),
];

/// Turns escape sequences into their value.
/// Returns a string of the escape sequence which failed on error.
pub fn normalize_string_literal(string: Either<&StrLit, &CharLit>) -> Result<String, ExpressionEvaluationError> {
    let slice = string.as_ref().either(|s| s.slice_str(), |c| c.slice_str());
    // this assumes that the string literal is surrounded by quotes, which it should always be
    let slice_without_quotes = &slice[1..slice.len()-1];

    normalize_string_formatting(&slice_without_quotes)
        .map_err(|sequence| ExpressionEvaluationError::EscapeSequencesIsInvalid {
        lit: string.map_either(|s| s.clone(), |c| c.clone()),
        sequence
    })
}

/// Turns escape sequences into their value.
/// Returns a string of the escape sequence which failed on error.
fn normalize_string_formatting(string: &str) -> Result<String, String> {
    let mut nomalized_string = String::new();
    let mut characters = string.chars();

    while let Some(ch) = characters.next() {
        if ch == '\\' {
            if let Some(escaped) = characters.next() {
                let Some(true_character) = ESCAPE_SEQUENCES.iter()
                    .find_map(|(from, to)| if *from == escaped { Some(*to) } else { None })
                else {
                    return Err(format!("\\{}", escaped));    
                };

                nomalized_string.push(true_character);
            } else {
                nomalized_string.push('\\');
            }
        } else {
            nomalized_string.push(ch);
        }
    }

    Ok(nomalized_string)
}
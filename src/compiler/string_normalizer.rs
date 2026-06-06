//! Turns strings with escape sequences (e.g: \n, \", \', \\) into their actual string value

use thiserror::Error;

const ESCAPE_SEQUENCES: &[(char, char)] = &[
    ('\\', '\\'),
    ('n', '\n'),
    ('t', '\t'),
    ('\"', '\"'),
    ('\'', '\''),
];

#[derive(Debug, Clone, PartialEq, Error)]
#[error("escape sequence {0} is invalid")]
pub struct InvalidEscapeSequence(String);

/// Turns escape sequences into their value.
/// Returns a string of the escape sequence which failed on error.
pub fn normalize_string_formatting(string: &str) -> Result<String, InvalidEscapeSequence> {
    let mut nomalized_string = String::new();
    let mut characters = string.chars();

    while let Some(ch) = characters.next() {
        if ch == '\\' {
            if let Some(escaped) = characters.next() {
                let Some(true_character) = ESCAPE_SEQUENCES.iter()
                    .find_map(|(from, to)| if *from == escaped { Some(*to) } else { None })
                else {
                    return Err(InvalidEscapeSequence(format!("\\{}", escaped)));    
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
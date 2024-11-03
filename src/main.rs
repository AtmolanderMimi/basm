use std::path::absolute;

use bf_unfucked::{error, lexer::lex_file, source::SourceFile};

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or_else(|| panic!("please provide a file path to the executable!"));
    let abs_path = absolute(path).unwrap();
    let sf = SourceFile::from_file(abs_path)
        .unwrap();
    let (tokens, errors) = lex_file(&sf);
    println!("------------- [ TOKENS ] ---------------");
    println!("{:#?}", tokens);
    if errors.len() != 0 {
        println!("\n-------------------- [ ERRORS ] ------------------");
        println!("{:#?}", errors);
    }
}

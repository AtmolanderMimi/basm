use std::path::absolute;

use bf_unfucked::{error, lexer::lex_file, source::SourceFile, CompilerError};

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or("./test-resources/fib.bfu".to_string());//.unwrap_or_else(|| panic!("please provide a file path to the executable!"));
    let abs_path = absolute(path).unwrap();
    let sf = SourceFile::from_file(abs_path)
        .unwrap();
    let (tokens, errors) = lex_file(&sf);
    println!("------------------ [ TOKENS ] ------------------");
    println!("{:#?}", tokens);
    if errors.len() != 0 {
        println!("\n------------------ [ ERRORS ] ------------------");
        for e in errors {
            // TODO: Add comments
            println!("{}", e.description());
        }
    }
}

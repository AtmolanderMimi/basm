use std::path::absolute;

use basm::{parser::parse_tokens, source::SourceFile, CompilerError};

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or("./test-resources/fib.basm".to_string());//.unwrap_or_else(|| panic!("please provide a file path to the executable!"));
    let abs_path = absolute(path).unwrap();
    let sf = SourceFile::from_file(abs_path)
        .unwrap();
    let (tokens, errors) = basm::lex_file(&sf);
    println!("------------------ [ TOKENS ] ------------------");
    println!("{:#?}", tokens);
    if !errors.is_empty() {
        println!("\n------------------ [ ERRORS ] ------------------");
        for e in errors {
            println!("{}", e.description());
        }
    }

    println!("\n------------------ [ PARSED ] ------------------");
    let program = parse_tokens(&tokens).unwrap();
    println!("{program:#?}");
}

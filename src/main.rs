use std::path::absolute;

use basm::{compiler::compile, parser::parse_tokens, source::SourceFile, CompilerError};

fn main() {
    let path = std::env::args().nth(1)
        .unwrap_or("./test-resources/fib.basm".to_string());//.unwrap_or_else(|| panic!("please provide a file path to the executable!"));
    let abs_path = absolute(path).unwrap();
    let sf = SourceFile::from_file(abs_path)
        .unwrap();
    let (tokens, errors) = basm::lex_file(&sf);
    println!("------------------ [ TOKENS ] ------------------");
    if !errors.is_empty() {
        println!("\n------------------ [ ERRORS ] ------------------");
        for e in errors {
            println!("{}", e.description());
        }
    }

    println!("\n------------------ [ PARSED ] ------------------");
    let program = match parse_tokens(&tokens) {
        Ok(p) => p,
        Err(e) => { println!("{}", e.description()); panic!() },
    };

    println!("\n------------------ [ COMPILED ] ------------------");
    let program = match compile(&program) {
        Ok(p) => p,
        Err(e) => { println!("{}", e.description()); panic!() },
    };
    println!("{program}");
}

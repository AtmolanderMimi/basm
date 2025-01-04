//! # Brain Aneurysm
//! 
//! started as of 2024-10-12

#![feature(assert_matches)]

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::must_use_candidate,
    // messes up with colored, not all `to_string` are directly going to be displayed yakno
    clippy::to_string_in_format_args,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
)]

pub mod error;
use std::path;

pub use error::{CompilerError, Lint};
pub mod lexer;
pub use lexer::lex_file;
pub mod source;
pub mod utils;
pub mod parser;
pub mod compiler;
pub mod interpreter;
pub mod clap_cli;
pub use clap_cli::CliCommand;
use source::SourceFile;

/// Transpiles bfu source code into bf.
pub fn transpile<'a>(sf: &'a SourceFile) -> Result<String, Vec<Box<dyn CompilerError + 'a>>> {
    let (tokens, errors) = lexer::lex_file(&sf);
    if !errors.is_empty() {
        let boxed_errs = errors.into_iter()
            .map(|e| Box::new(e) as Box<dyn CompilerError>)
            .collect();
        return Err(boxed_errs)
    }

    let program = match parser::parse_tokens(&tokens) {
        Ok(p) => p,
        Err(e) => return Err(vec![Box::new(e)]),
    };

    let program = match compiler::compile(&program) {
        Ok(p) => p,
        Err(e) => return Err(vec![Box::new(e)])
    };
    
    Ok(program)
}
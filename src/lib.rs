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
    clippy::struct_excessive_bools,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
)]

pub mod error;

pub use error::{CompilerError, Lint};
pub mod lexer;
pub use lexer::lex_file;
pub mod source;
use source::SourceFile;
pub mod utils;
pub mod parser;
pub mod compiler;
pub mod interpreter;
pub mod clap_cli;
pub use clap_cli::CliCommand;
mod optimiser;
pub use optimiser::optimise;

/// Transpiles bfu source code into bf.
pub fn transpile<'a>(sf: &'static SourceFile) -> Result<String, Vec<Box<dyn CompilerError + 'a>>> {
    let tokens = match lexer::lex_file(sf) {
        Ok(tokens) => tokens,
        Err((_, errors)) => {
            let boxed_errs = errors.into_iter()
            .map(|e| Box::new(e) as Box<dyn CompilerError>)
            .collect();
            return Err(boxed_errs)
        }
    };

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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
        use std::{hint::black_box, path::PathBuf, time::Instant};
        
        use crate::interpreter::InterpreterBuilder;

        use super::*;

    #[test]
    #[ignore = "compute intensive (needs release)"]
    fn compile_performance_reading_alot_of_data() {
        #[cfg(debug_assertions)]
        panic!("please run as release");

        #[cfg(not(debug_assertions))]
        {
            const ACCEPTABLE_TIME: f32 = 0.05;
            
            let timer = Instant::now();
            let sf = SourceFile::from_raw_parts(
                PathBuf::from("./../test-resources/big-file.basm"),
                include_str!("./../test-resources/big-file.basm").to_string()
            ).leak();

            // the file is supposed to error, so we don't unwrap
            let _ = black_box(transpile(sf));

            let duration = timer.elapsed();
            if duration.as_secs_f32() > ACCEPTABLE_TIME {
                panic!("transpiling took {}", duration.as_secs_f32());
            }
        }
    }

    #[test]
    #[ignore = "compute intensive"]
    fn basm_bf_interpreter() {
        let sf = SourceFile::from_raw_parts(
            PathBuf::default(),
            include_str!("./../test-resources/better-bf-interpreter.basm").to_string()
        ).leak();

        let bf_prog = transpile(sf).unwrap();

        // hello world program
        let hello_bf = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.!";

        let mut inter = InterpreterBuilder::new(&bf_prog)
            .with_bulk_input()
            .finish();
        assert!(inter.add_to_input_buffer(hello_bf));
        inter.complete().unwrap();
        assert_eq!(inter.captured_output(), "Hello World!\n");

        // fibonacci program
        let mut fib_bf = include_str!("./../test-resources/fib.bf").to_string();
        fib_bf.push('!');

        let mut inter = InterpreterBuilder::new(&bf_prog)
            .with_output_as_number()
            .with_bulk_input()
            .finish();
        assert!(inter.add_to_input_buffer(&fib_bf));
        inter.complete().unwrap();
        assert_eq!(inter.captured_output(), "1 2 3 5 8 13 21 34 55 89 144 ");
    }
}
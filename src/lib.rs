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
    let (tokens, errors) = lexer::lex_file(sf);
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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
        use std::{hint::black_box, path::PathBuf, time::Instant};
        
        use super::*;

    #[test]
    #[ignore = "compute intensive (needs release)"]
    fn compile_performance_reading_alot_of_data() {
        #[cfg(debug_assertions)]
        panic!("please run as release");

        #[cfg(not(debug_assertions))]
        {
            // TODO: big-file is not big enough to justify this
            const ACCEPTABLE_TIME: f32 = 0.3;
            
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
}
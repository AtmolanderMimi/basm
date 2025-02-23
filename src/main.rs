use std::{fs, io::Write, path};

use basm::{clap_cli::CompileArgs, source::SourceFile, CliCommand, CompilerError};
use clap::Parser;
use colored::Colorize as _;

const MALFORMED_INPUT: &str = "the input path is malformed";
//const MALFORMED_OUTPUT: &str = "the output path is malformed";
const INACCESSIBLE_INPUT: &str = "failed to access input";
const INACCESSIBLE_OUTPUT: &str = "failed to access output";
const UNWRITEABLE_OUTPUT: &str = "failed to write to output file";

fn main() {
    let cli = CliCommand::parse();
    
    let file_path = match &cli {
        CliCommand::Compile(args) => &args.file_path,
        CliCommand::Run(args) => &args.file_path,
    };

    let abs_path = path::absolute(file_path)
        .unwrap_or_else(|_| error_out(MALFORMED_INPUT));

    let is_basm_file = match &cli {
        CliCommand::Compile(_) => true,
        CliCommand::Run(args) => !args.raw,
    };

    // transpiling (or not)
    let mut program = if is_basm_file {
        let sf = SourceFile::from_file(&abs_path)
            .unwrap_or_else(|_| error_out(INACCESSIBLE_INPUT))
            .leak();

        let program = match basm::transpile(sf) {
            Err(errors) => {
                eprintln!("\n------------------ [ ERRORS ] ------------------");
                for e in errors {
                    eprintln!("{}", CompilerError::description(&*e));
                };
                std::process::exit(1)
            },
            Ok(p) => p,
        };

        program
    } else {
        fs::read_to_string(&abs_path)
            .unwrap_or_else(|_| error_out(INACCESSIBLE_INPUT))
    };

    let optimise = match &cli {
        CliCommand::Compile(args) => !args.unoptimised,
        CliCommand::Run(args) => !args.unoptimised,
    };

    if optimise {
        program = basm::optimise(&program);
    }

    // show (if necessary)
    let show = match &cli {
        CliCommand::Compile(args) => args.show,
        CliCommand::Run(args) => args.show,
    };

    if show {
        println!("{program}");
    }

    // writing to output file (if necessary)
    if let CliCommand::Compile(CompileArgs { out, .. }) = &cli {
        let out_path = out.clone().unwrap_or_else(|| {
            let mut file_path = abs_path.clone();
            file_path.set_extension("bf");
            
            file_path.to_string_lossy().to_string()
        });

        let mut output_file = fs::File::create(&out_path)
        .unwrap_or_else(|_| error_out(INACCESSIBLE_OUTPUT));

        output_file.write_all(program.as_bytes())
        .unwrap_or_else(|_| error_out(UNWRITEABLE_OUTPUT));
    }

    // interpreting (if necessary)
    let run_args = if let CliCommand::Run(args) = cli {
        // okay
        args
    } else {
        return
    };

    let mut interpreter = match run_args.build_interpreter(&program) {
        Ok(i) => i,
        Err(e) => error_out(&e.to_string())
    };
    match interpreter.complete() {
        Ok(()) => (),
        Err(e) => {
            let msg = format!("{}: {}", "Intepreter Error".red().bold(), e.to_string());
            eprintln!("{}", msg)
        },
    }

    if run_args.dump {
        interpreter.print_dump();
    }
}

fn error_out(reason: &str) -> ! {
    eprintln!("{reason}");
    std::process::exit(1)
}

use std::{fs, io::Write, path::absolute};

use basm::{clap_cli::CompileArgs, interpreter::InterpreterBuilder, source::SourceFile, transpile, CliCommand, CompilerError};
use clap::Parser;

const MALFORMED_INPUT: &str = "the input path is malformed";
const MALFORMED_OUTPUT: &str = "the output path is malformed";
const INACCESSIBLE_INPUT: &str = "failed to access input";
const INACCESSIBLE_OUTPUT: &str = "failed to access output";
const UNWRITEABLE_OUTPUT: &str = "failed to write to output file";

const INVALID_CELL_SIZE: &str = "invalid cell size, must be 8, 16, or 32";

fn main() {
    let cli = CliCommand::parse();
    
    let file_path = match &cli {
        CliCommand::Compile(args) => &args.file_path,
        CliCommand::Run(args) => &args.file_path,
    };

    let abs_path = absolute(file_path)
        .unwrap_or_else(|_| error_out(MALFORMED_INPUT));

    let is_basm_file = match &cli {
        CliCommand::Compile(_) => true,
        CliCommand::Run(args) => !args.raw,
    };

    // transpiling (or not)
    let program = if is_basm_file {
        let sf = SourceFile::from_file(&abs_path)
            .unwrap_or_else(|_| error_out(INACCESSIBLE_INPUT));

        let program = match transpile(&sf) {
            Err(errors) => {
                println!("\n------------------ [ ERRORS ] ------------------");
                for e in errors {
                    println!("{}", CompilerError::description(&*e));
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

        output_file.write(program.as_bytes())
        .unwrap_or_else(|_| error_out(UNWRITEABLE_OUTPUT));
    }

    // interpreting (if necessary)
    let run_args = if let CliCommand::Run(args) = cli {
        // okay
        args
    } else {
        return
    };

    let builder = InterpreterBuilder::new(&program);

    // cell type
    let builder = match (run_args.signed, run_args.cell_size) {
        (false, 8) => builder.with_u8(),
        (false, 16) => builder.with_u16(),
        (false, 32) => builder.with_u32(),
        (true, 8) => builder.with_i8(),
        (true, 16) => builder.with_i16(),
        (true, 32) => builder.with_i32(),
        _ => error_out(INVALID_CELL_SIZE),
    };

    // overflow behaviour
    let builder = if run_args.abort_overflow {
        builder.with_aborting_behaviour()
    } else {
        builder.with_wrapping_behaviour()
    };

    // tape limit
    let builder = if let Some(limit) = run_args.tape_limit {
        builder.with_tape_leght(limit)
    } else {
        builder.without_tape_lenght()
    };

    let mut interpreter = builder.finish();
    match interpreter.complete() {
        Ok(()) => (),
        Err(e) => error_out(&e.to_string()),
    }
}

fn error_out(reason: &str) -> ! {
    println!("{reason}");
    std::process::exit(1)
}

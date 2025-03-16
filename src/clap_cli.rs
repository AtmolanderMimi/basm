//! The cli parser defined via the clap crate and its tooling.

use clap::{command, Args, Parser};
use thiserror::Error;

use crate::interpreter::{InterpreterBuilder, InterpreterTrait};

/// The basm cli tool for transpiling basm into brainfuck and interpreting basm code transpiled into brainfuck.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug, PartialEq, Clone)]
pub enum CliCommand {
    /// Compiles the program and writes it to file
    Compile(CompileArgs),
    /// Compiles and interprets the program
    Run(RunArgs),
}

/// Arguments for the `run` command.
#[derive(Args)]
#[derive(Debug, PartialEq, Clone)]
pub struct RunArgs {
    /// Path to the basm file
    pub file_path: String,

    /// Sets the size of cells in bits (only 8, 16 and 32)
    #[arg(long, short, default_value_t = 8)]
    pub cell_size: usize,

    /// Sets the cells as signed containing signed numbers
    #[arg(long, short = 'i', default_value_t = false)]
    pub signed: bool,

    /// Aborts the execution of the program when a cell over/under-flows
    #[arg(long, short, default_value_t = false)]
    pub abort_overflow: bool,

    /// Limits the lenght of the tape in cells, aborts execution if it is reached
    #[arg(long, short)]
    pub tape_limit: Option<usize>,

    /// Treats the input as integer numbers rather than characters
    #[arg(long, short = 'n', default_value_t = false)]
    pub number_input: bool,

    /// Treats the output as integer numbers rather than characters
    #[arg(long, short = 'm', default_value_t = false)]
    pub number_output: bool,

    /// Removes the bufferring of the unused parts of inputs to provide to them to later bf inputs.
    #[arg(long, short = 's', default_value_t = false)]
    pub single_input: bool,

    /// Interprets the file as brainfuck, skips the compiling process
    #[arg(long, short, default_value_t = false)]
    pub raw: bool,

    /// Print the transpiled brainfuck
    #[arg(long, short = 'p', default_value_t = false)]
    pub show: bool,

    /// Skips the use of the inbuilt brainfuck optimiser
    #[arg(long, short = 'u', default_value_t = false)]
    pub unoptimised: bool,

    /// Dump the tape and tape pointer position to terminal once the program ends
    /// (includes by erroring out)
    #[arg(long, short = 'd', default_value_t = false)]
    pub dump: bool,
}

impl RunArgs {
    /// Builds an interpreter configured using the cli flags.
    /// May return `Err` containing a `String` if the arguments are invalid
    pub fn build_interpreter(&self, program: &str) -> Result<Box<dyn InterpreterTrait>, InterpreterBuildingError> {
        let builder = InterpreterBuilder::new(program);

        // cell type
        let builder = match (self.signed, self.cell_size) {
            (false, 8) => builder.with_u8(),
            (false, 16) => builder.with_u16(),
            (false, 32) => builder.with_u32(),
            (true, 8) => builder.with_i8(),
            (true, 16) => builder.with_i16(),
            (true, 32) => builder.with_i32(),
            (_, s) => return Err(InterpreterBuildingError::InvalidCellSize { got: s }),
        };

        // overflow behaviour
        let builder = if self.abort_overflow {
            builder.with_aborting_behaviour()
        } else {
            builder.with_wrapping_behaviour()
        };

        // tape limit
        let builder = if let Some(limit) = self.tape_limit {
            builder.with_tape_leght(limit)
        } else {
            builder.without_tape_lenght()
        };

        // input type
        let builder = if self.number_input {
            builder.with_input_as_number()
        } else {
            builder.with_input_as_character()
        };

        // output type
        let builder = if self.number_output {
            builder.with_output_as_number()
        } else {
            builder.with_output_as_character()
        };

        // bulk input
        let builder = if !self.single_input {
            builder.with_bulk_input()
        } else {
            builder.without_bulk_input()
        };

        Ok(builder.finish())
    }
}

/// An error caught during interpreter initialisation.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InterpreterBuildingError {
    /// Cell size was invalid, and thus the interpreter could not be initialized.
    #[error("specified cell size is invalid, expected 8, 16 or 32, got {got},")]
    InvalidCellSize {
        /// The specified cell size, which was invalid
        got: usize,
    }
}

/// Arguments for the `compile` command.
#[derive(Args)]
#[derive(Debug, PartialEq, Clone)]
pub struct CompileArgs {
    /// Path to the basm file
    pub file_path: String,

    /// The path to put the compiled file
    #[arg(long, short)]
    pub out: Option<String>,

    /// Print the transpiled bf script
    #[arg(long, short = 'p', default_value_t = false)]
    pub show: bool,

    /// Skips the use of the inbuilt brainfuck optimiser
    #[arg(long, short = 'u', default_value_t = false)]
    pub unoptimised: bool,
}
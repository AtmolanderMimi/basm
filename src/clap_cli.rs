//! The cli parser defined via the clap crate and its tooling.

use clap::{command, Args, Parser};

/// The clap cli interface commands.
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

    /// Interprets the file as brainfuck, skips the compiling process
    #[arg(long, short, default_value_t = false)]
    pub raw: bool,

    /// Print the transpiled brainfuck
    #[arg(long, short = 'p', default_value_t = false)]
    pub show: bool,

    /// Dump the tape and tape pointer position to terminal once the program ends
    /// (includes by erroring out)
    #[arg(long, short = 'd', default_value_t = false)]
    pub dump: bool,
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
}
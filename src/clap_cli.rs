use clap::{command, Args, Parser, Subcommand};

/// The clap cli interface.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[derive(Debug, PartialEq, Clone)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
#[derive(Debug, PartialEq, Clone)]
pub enum Commands {
    /// Compiles the program and writes it to file
    Compile(CompileArgs),
    /// Compiles and interprets the program
    Run(RunArgs),
}

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

    /// Print the transpiled brainfuck
    #[arg(long, short = 'p', default_value_t = false)]
    pub show: bool,

    /// Interprets the file as brainfuck, not basm
    #[arg(long, short, default_value_t = false)]
    pub raw: bool,
}

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
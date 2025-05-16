# Cli Capabilities
Alongside the language itself, I'd like to include some information about the transpiler I wrote for it,
because, of course, there's no point in writing a book about a compiler*less* language.

The only (probably forever) basm compiler is
[available on github](https://github.com/AtmolanderMimi/basm)
under the retrospectively confusing name `basm`.
This tool doesn't just include a compiler,
it also comes with a bf interpreter to run transpiled basm directly.

Currently, the cli has two subcommands: `compile` and `run`.
Both of them are similar in that they both take in a file path and have some flags in common.

## `compile`
The `compile` subcommand is the simplest way to use the basm cli.
Similar to tool like `gcc`, it will create a file in the current working directory named like the one passed in with the extention replaced with `.bf`.
File name and output path can be specified with the `-o` flag.
If compilation fails, error information will be printed to the terminal.

### Flags
{{#custom compile-flags}}

## `run`
This command compiles then run the specified basm source file.
You can use the bf interpreter directly if you use this command the `-r` flag,
which specifies that the file is a bf source file, not a basm source file.
Unlike `compile`, this does not create a file containing the compiled bf.
If the compilation fails, error information will be printed to the terminal.

### Flags
{{#custom run-flags}}

## Bf Optimisations
`basm` applies some basic optimisations to the bf resulting from the transpilation process by default.
It can merge operators to reduce redundant use (e.g: `+++-` would turn into `++`)
and it can reorder operations so that less tape pointer moves are used.
Overall, the purpose of the built in optimiser is to reduce the number of operators in compiled scripts.
By default, basm doesn't remove redundent operations and is generally quite stupid.
For example, an instruction like `INCR 12 0;`, which does nothing,
would still move the tape pointer to the 12th cell. This gets optimised out.

All optimisation are made to have no impact on the resulting program, at a few exceptions:
* Willingfully moving the tape pointer before 0 will be optimised out if there is a cancelling move afterwards (e.g: `<<<<<<<<<<<<<<<>>>>>>>>>>>>>>>`)
* Any pointer moves that don't lead to other operations (at the end of the program) are removed

If you suspect that optimisations are messing with your program,
you can disable them with the `-u` flag.
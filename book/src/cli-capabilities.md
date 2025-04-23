# Cli Capabilities
Alongside the language itself, I'd like to include some information about the transpiler I wrote for it,
because of course without a compiler there's no point in writing a book about a language.

The only (probably forever) basm compiler is
[available on github](https://github.com/AtmolanderMimi/basm)
under the retrospectively confusing name `basm`.
The basm cli doesn't just include a compiler though,
it also comes with a bf interpreter to run the transpiled basm directly.

Currently, the cli comes has two subcommands: `compile` and `run`.
Both of them are similar, in that they both take in a file path and have some flags in common.

## `compile`
The `compile` subcommand is the simplest way to use the basm cli.
Similar to tool like `gcc`, it will create a file named the same as the passed in one `.bf` extention in the current working directory, unless specified with the `-o` flag.
# (IF THIS REMINDER IS LEFT HERE I DIDN'T CHANGE IT, AND SO IT IS ACTUALLY IN THE DIRECTORY OF THE SOURCE FILE)
If the compilation fails, error information will be printed to the terminal.

### Flags
{{#custom compile-flags}}

## `run`
This command both compiles and run the specified basm source file via the inbuilt bf interpreter.
You can also use the bf interpreter directly if you use the `-r` flag,
which specifies that the file is a bf source file.
Unlike `compile`, this does not create a file containing the compiled bf.
If the compilation fails, error information will be printed to the terminal.

### Flags
{{#custom run-flags}}

## Bf Optimisations
`basm` comes with basic bf optimisation applied out of the box after the compilation.
It can merge operators to reduce redundant use (`+++-` would turn into `++`)
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
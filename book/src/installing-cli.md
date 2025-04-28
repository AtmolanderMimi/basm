# Installing the Cli

First, before getting into programming in basm, you'll need to install the basm transpiler.
You can find compiled binaries of the `basm` cli tool in the releases section at [https://github.com/AtmolanderMimi/basm]() or you can choose to compile the Rust code straight from source with `cargo`.
(Don't be scared, despite being built 100% Rust, learning basm does not require any Rust knowlege)

Once installed you can use the `basm` tool through the terminal.
It comes with a basm to bf transpiler, simple bf code optimiser and a bf interpreter.
To transpile and run any basm file use the `run` subcommand like below:

```bash
basm run my-file.basm
```

For a more compresensive list of  capabilities, you can use the `help` subcommand or read [the chapter on the `basm` cli tool](./cli-capabilities.md).

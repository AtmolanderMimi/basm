# Installing the Cli

First and formost, before getting into programming in basm, you'll need to install the basm transpiler.
You can find compiled binaries of the `basm` cli tool in the releases section at [https://github.com/AtmolanderMimi/basm]() or you can choose to compile the Rust code straight from source with `cargo`.
(Don't be scared, despite being built in 100% Rust, learning basm does not require any Rust knowlege)

Once installed you can use the `basm` tool through the terminal.
It comes with a basm-bf transpiler, simple bf code optimiser and a bf interpreter.
To transpile and run any basm file you can simply enter the command below:
```bash
basm run my-file.basm
```

To see more options you can use the `help` subcommand or read [the chapter on the `basm` cli tool](./cli-capabilities.md).
# Our First Program

Here is how you write a "Hello, world!" program in basm:

```basm
[main] [
PSTR 0 "Hello, world!";
]
```

Which would transpile to:
```bf
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
+++++++++++++++++++++++++++++.+++++++..
+++.[-]++++++++++++++++++++++++++++++++++++++++++++.------------.
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
--------.+++.------.--------.
[-]+++++++++++++++++++++++++++++++++.[-]
```

This program usues the cell no.0 as a buffer for each character in the string `"Hello, world!"`.
Quite simple isn't it? (compared to the bf at least) Let's start dissecting what it all means.
First, we begin from the top with the `[main]` decorator which indicates that the following
scope will be a main field. There can only be one main field per program and, by the name, you
have probably already guessed what it is. It's the entrypoint of our program! Any instruction put inside the scope of the `[main]` field will be transpiled and executed at runtime.

## Instructions
"Ok, but what's an instruction in basm?", I hear you asking.
An instruction is the basic building block of basm programs.
They represent an action and are the smallest unit that can be translated into bf.
You can think of them like functions in other programming languages:
They take a set number of typed arguments and operate using these arguments.
In this program, the operation was "print a string" using the cell at index `0`.
Here is the syntax to insert any instruction:
```
INSTRUCTION_NAME arg1 arg2 .. ;
```
`INSTRUCTION_NAME` is the name of the instruction.
An instruction's name is not required to be fully capitalised,
although it is the standard that I am going to use.
`arg1` and `arg2` are arguments to the instruction.
Arguments are seperated by whitespaces (don't put commas!).
To end an instruction we need to put a semicolon, and we are done here.
In the case of the `PSTR` instruction above:
* `PSTR` was the name (meaning "print string"),
* `0` was the first argument and
* `"Hello, world!"` was the second argument.

## Scopes
As said prior a `[main]` field is made up of a `[main]` decorator and a scope. So what is a scope?
A scope is zero or more instructions put between square brackets.
They can be used to make fields and can serve as arguments to instructions,
such as conditionals and loops.
Scopes can also store their own aliases and serve as lifetimes for them.
(we will see this later in the Aliases chapter)

This would be a valid scope:
```basm
[]  // totally empty
    // (comments can be made by writing "//" which will make the rest of a line a comment)
```

.. and so would this third argument in this `WHNE` instruction:
```basm
WHNE 0 42 [
    INCR 0 1;
];
```

Like other languages, it is totally valid to write a scope within another scope.
With what you know now, it is not useful, but you will learn that
it can convenient for making local temporary aliases and sublogic in parts of code.
Note that scope aliases, which you will also see later in the chapter on aliases,
can't be inlined like this.

This is perfectly valid code:
```basm
[main] [
// doing stuff
INCR 0 42;

// oof, the main scope is getting quite crowded let's get out
[
    OUT 0;
]
]
```

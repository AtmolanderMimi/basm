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

Quite simple isn't it? (compared to the bf at least) Let's start dissecting what it all means.
First, we begin from the top with the `[main]` decorator which indicates that the following
scope will be a main field. There can only be one main field per program and, by the name, you
have probably already guessed what it is. It's the entrypoint of our program! Any instruction put inside the scope of the `[main]` field will be transpiled and executed at runtime.

## Instructions
"Ok, but what's an instruction in basm?", I hear you asking.
An instruction is the basic building block of basm programs.
They represent an action and are the smallest unit that can be translated into bf.
Here is the syntax to insert an instruction:
```
INSTRUCTION_NAME arg1 arg2 .. ;
```
`INSTRUCTION_NAME` is the name of the instruction.
An instruction's name is not required to be fully capitalised,
although it is the standard that I am going to use.
`arg1` and `arg2` are arguments to the instruction.
Arguments are seperated by whitespaces (don't put commas!).
Instructions have a set number of arguments with strongly typed types.
To end an instruction we need to put a semicolon, and we are done.
In the case of the `PSTR` instruction above:
* `PSTR` was the name (meaning "print string"),
* `0` was the first argument and
* `"Hello, world!"` was the second argument.

## Scopes
As said prior a `[main]` field is made up of a `[main]` decorator and a scope. So what is a scope?
A scope is zero or more instructions put between square brackets.
They can be used to make fields and can serve arguments to instructions,
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
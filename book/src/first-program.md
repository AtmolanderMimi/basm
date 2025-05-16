# Our First Program

Like any good programming language, you must start from the basics. The basics in this case being a "Hello, world!" program.
Here is how you write one in basm:

```basm
[main] [
PSTR 0 "Hello, world!";
]
```

This source code file, once transpiled through the `basm` cli would compile down to this is bf:

```bf
++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
+++++++++++++++++++++++++++++.+++++++..
+++.[-]++++++++++++++++++++++++++++++++++++++++++++.------------.
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.
--------.+++.------.--------.
[-]+++++++++++++++++++++++++++++++++.[-]
```

This program uses cell 0 as a buffer to increment and output each character in the string `"Hello, world!"` consecutively.
Quite simple isn't it? *(compared to the bf at least)*

Let's start dissecting what it all means.
First, we begin at the top with the `[main]` decorator which indicates that the following
scope will be a main field. There can only be one main field per program, and by the name you
have probably already guessed what it is. It's the entry point of our program! Any instruction put inside the scope of the `[main]` field will be transpiled and then executed at runtime. In this example, the main field only held one instruction, which is `PSTR 0 "Hello, world!"`.

## Instructions Statement

"Ok, but what's an instruction in basm?", I hear you asking.
An instruction is the basic building block of basm programs.
They represent an action and are the smallest unit of logic in the programming language.
You can think of them like functions in other programming languages:
They take a set number of typed arguments and operate using these arguments.
In this program, the operation was "print a string" using the cell at index `0`.
Here is the syntax for any instruction statement:

```
INSTRUCTION_NAME arg1 arg2 .. ;
```

`INSTRUCTION_NAME` is the name of the instruction.
An instruction's name is not required to be fully capitalized,
although it is the standard that I am going to use.
`arg1` and `arg2` are the arguments to the instruction.
Arguments are separated by whitespaces (don't put commas!).
To end an instruction we need to put a semicolon, and we are done here.
In the case of the `PSTR` instruction above:

* `PSTR` was the name (meaning "print string"),
* `0` was the first argument and
* `"Hello, world!"` was the second argument.

## Scopes

As said prior, a `[main]` field is made up of a `[main]` decorator and a scope. So what is a scope?
A scope is a collection of zero or more instructions or scopes.
Scopes are denoted by matching square brackets which encapsulate their contents.
They can be used to make fields and can serve as arguments to instructions,
such as conditionals and loops.
Scopes also serve as lifetimes for aliases.
Meaning any alias defined within themselves are invalid outside of themselves,
basically serving to limit the lifetime of aliases.
(we will see this later in the Aliases chapter)

The scope syntax is very simple, just put square brackets around stuff! This would be a valid scope:

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

Like other languages, it is totally valid to write a scope within another scope as a statement.
Doing this, inlines the contents of the scope, practically like you would have not written the scope contents in a scope.
With what you know now, scope statements are not useful, but you will learn that
they can  be convenient for making local, temporary aliases and sublogic in parts of code.
*Note that scope aliases, which you will also see later in the chapter on aliases, can't be inlined like this.*

Here is an example of using a scope as an expression: (only instruction need semicolons to terminate their statement, scope statements don't need it)

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

Scope statements (for inlining) and scope expressions (for instruction arguments) are written exactly the same.
Their expression-*ness* is dependent on context.
When in a scope, a scope is parsed as a statement. Whereas, when in an instruction argument (which is most of the time), a scope is an expression.

### Note

From now on, basm code in code blocks will not always be surrounded by a `[main]` field
for ease of reading *(and writing ;))*.
All code will be implicitly contained in `[main]` unless otherwise noted.

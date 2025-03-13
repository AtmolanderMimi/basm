# Brain Aneurysm

This file and other content within this direcotory are disorganised notes and documentation used for the creation and *pondering* of basm.
Stuff in here is not guarentied to be up to date or to even be implemented.
Hopefully one day a basm-book detailing the language should be produced.

## Description
BrainAneurysm (or for short basm) is a very simple, assembly-like, language transpiling to Brain*Fuck*. *(ono, big boy words!)*
The purpose of this language is to abstract of the parts of BrainFuck that makes it *fucky* like:
* The relative nature of memory,
* Difficulty working with text,
* The lack of organisation features,
  Mitigating these problems will hopefully allow writing complex programs.
  This project is only for my own personal pleasure as a creator and is not aimed at creating efficient bf programs.
  If you useable and well informed implementations of bf transpilers look [here](https://esolangs.org/wiki/Brainfuck_code_generation).

My main motivation for this project is to prove that if it is turing complete, it can do anything any other language could.
Whilst bf is very well known for being one of the smallest languages that is turing complete,
I believe that not many people have actually used it to create works that take advantage of the theoratically
all-purpose nature of bf. Whilst I say this, i know fully that this language is not and will never be the right
tool for the job of creating programs in this niche language that is bf. It is moreso a learning experience for myself.

Although this project was supposed to be a fully fledged language similar in syntax to Rust or C,
with after further reconsideration (looking into making a ast and parser), I have desided to 180 the
syntax of this language to something more akin to an assembly language.
Though i wouldn't call an assembly language, since it transpiles to a lower language, being bf.

In basm, files are split into three main fields:
* `[data]` which is used to preload cells of the bf tape with data before running any code,
* `[main]` which is the start point of your program,
* `[@name arg arg]` which defines a meta-instruction, there can be more than one of these fields,
Fields are decorators (like the ones above) followed by a scope.

## The `[data]` Field
This field allows you to define an values to be preloaded a certain adresses on the tape.
You can either load a single value into a single cell by using `SET (addr) (value)`
or you can load an entire string using `STR (starting addr) (string literal)`.
Loading a string via `STR` will set it into memory by equating each character with its ASCII value.
Strings in basm are assumed to start and end with a cell of value -1. Thus making the smallest string use up two cells.
A "-1 cell", when working with unsigned intergers is simply `0 - 1`. Which means underflowing, so for u8 it would be 255.
When providing the adress to it use to one referencing the starting -1 cell (which is going to be the same as the one you used to declare it).
At compile-time, the data field will get translated into a serie of instructions set before `[main]`.

### Instructions
| Name | Arguments | Function |
|-|-|-|
| CELL | addr, value | initiates the `addr` cell with the `value` |
| STR | addr, string | initiates the cells including and after `addr` with the ASCII values of the `string` |

### Example
[data]
CELL 0 42; // sets cell 0 to 42
STR 1 "The answer to life the universe an>
This field defines the entrypoint of your program.
Instructions written within it will be executed at runtime sequencially.
Arguments cells of instructions should be understood as "consumed", meaning that their value
is non-deterministic, unless the cell is a result argument, or stated otherwise.
To use a "consumed" cell you should use instructions.

### Instructions
| Name | Arguments | Function |
|-|-|-|
| ALIS | ident, value | aliases a value or scope to an identifier, this instruction is purely abstraction |
| INLN | [scope] | inlines a scope |
| - | - | - |
| RAW  | "str" | inlines the string in the transpiled code, this can be used to include brainfuck operators |
| BBOX | addr | moves the tape pointer to the adress of `addr` |
| ASUM | addr | tells to compiler to assume that the tape pointer is at `addr`, if that assumption is wrong all cells accesses will be offset |
| - | - | - |
| ZERO | addr | sets the value of `addr` to 0 |
| COPY | addr1, addr2, addr3 | copies the value of `addr1` into `addr2` and `addr3` |
| - | - | - |
| IN   | addr | takes input form the user and sets it in `addr`, behaviour will vary between bf implementations |
| OUT  | addr | send `addr` to the output, `addr` is not consumed |
| - | - | - |
| INCR | addr, value | increments the value of the `addr` cell by `value` |
| DECR | addr, value | decrements the value of the `addr` cell by `value` |
| ADDP | addr1, addr2 | adds `addr2` to `addr1` in place |
| SUBP | addr1, addr2 | substract `addr2` from `addr1` in place |
| - | - | - |
| WHNE | addr, value, [scope] | while the value of `addr` cell is not equal to `value` runs the `[scope]`. `addr` is not consumed |

### Example
Fibonacci:
```basm
[main] [
INCR 1 1 // a
INCR 2 0 // b
WHNE 0 3 [
    INCR 0 1;

    COPY 1 3 4;
    ADDP 1 4;
    ADDP 1 2;
    ZERO 2;
    ADDP 2 3;

    OUT  1;
];
]
```

```bf
>+              | INCR 1
>               | INCR 2
<<---[+++       | WHNE
 +              | INCR 0

 >[->>+>+<<<]   | COPY 1
 >>>[-<<<+>>>]  | ADDP 4
 <<[-<+>]       | ADDP 2
 [-]            | ZERO 2
 >[-<+>]        | ADDP 3

 <<.            | OUT  1
 <---           | WHNE 0
]+++            | WHNE 0
```

## The `[@name arg arg]` Fields
These fields allows you to define your own meta-instructions, to use elsewhere in the program.
You can make as many of these fields as you require, as long as there is no two meta-instructions
with the same name. It is also disallowed to make meta-instructions with the same name as built-in instructions.
Don't be scared though as there is no requirement for the name, other than it being an alphanumerical sequence with `_` allowed.
Meta-instructions, once defined, can be used from anywhere below their definition like built-in instructions.
As the name implies, meta-instructions are just instructions that are defined by a sequence of other instructions at compile time.
This means, that meta-instctions are **inlined** and cannot be recusive.

Arguments to meta-instructions are aliased by the name of the arguments in the meta-instruction signature.
There cannot be two arguments of the same name.
It is important to note that meta-instructions are in their own scope, thus any aliases from main should not affect them.
By default arguments are values, but we can specify that an argument is a scope by surrounding it by brackets in the
meta-instruction signature. Like so: `[ident]`. Scope aliases defined like this can then be accessed directly like any other alias
without surrounding them by brackets in the meta-instruction body.

### Example
Let's make an instruction that lets us set a cell to a specific value:
```basm
[@SET addr value] [
ZERO addr;
INCR addr value;
]
```

Here is a bit more complex example where we implement a multiplication instruction:
```basm
[@MULT addr1 addr2 addr3 sp)] [
// reserving two cells on the stack
ALIS factor_copy1 sp+1;
ALIS factor_copy2 sp+2;
// since this new "sp" alias is just shadowing the old one, when the meta-instruction body
// will end this alias will get invalidated and the prior sp will be restored, like we reclamed space on the stack
ALIS sp sp+2; 

WHNE addr2 0 [
    DECR addr2 1;
    COPY addr1 factor_copy1 factor_copy2;
    ADDP factor_copy1 addr3;
    ADDP factor_copy2 addr1; // this is just moving arg1 to arg2 if arg2 = 0, which it is
];
]
```
Since multiplication requires us to take a bit more memory on the tape than just the addresses
specified to us by the arguments, we take a sp (stack pointer) arguents that tells us where to
take that these extra temporary cells required for the operation.

## Aliasing
Aliasing via the `ALIS` instructions allows you to specify identifiers equating to values or scopes.
Note that the name can be any alphanumerical sequence of characters and `_`.
These aliases can be used to name adresses or constants for example.
Aliases automatically shadow other prior aliases with the same name, and get invalidated
once they run out of the scope. In basm the only two elements creating scopes are `[]` blocks
and meta-instruction. Higher scope aliases can go into lower scope, but lower scope aliases cannot reach higher scopes.
The two alias types (being value and scope) both have a different name pool, meaning an alias both `val` and `[val]`
can coexist.

### Scope aliases
Scope aliases are defined like value aliases at the exception that we alias to a scope. Like this:
```basm
ALIS my_scope [ INCR 0 1; ];
```
Scope aliases can then be used by surrounding the set alias identifier by a set of brackets.
They can be used to pass into instructions that require scopes. Example:
```basm
[main] [
ALIS i 0;
ALIS my_scope [ INCR i 1; ];

INCR i 80;
// in this context this reads: while i is not 72, execute `my_scope` (which simply decreases i)
WHNE i 72 [my_scope];

// this should return 72 or ':' if not using number output
OUT i;
]
```

## Working with Raw Brainfuck
Basm isn't great at some things (like adressing memory dynamically), which is why there exists
three instructions to help you include brainfuck within basm code. They were already presented prior,
but let's reintroduce them in more detail:
* **RAW**: Takes in a string an includes it into the final transpiled code, we can use it to smuggle in brainfuck operators.
* **BBOX**: Forces the tape pointer to move to a specified cell (aka "black box")
* **ASUM**: Sets the assumed tape pointer position to the specified value, we can say we assume it. If the assumed position is invalid, this causes an offset in cell adressing.

`BBOX` and `ASUM` both manipulate the tape pointer. Before I can start to explain how to use them and
how this relates to including brainfuck, you need to know why we need them.
During the compilation process, the compiler keeps track of the position of the tape pointer at all points in the program.
It needs to do this so it can know how to move any arbitrary cell. We use static cell indexing (aka the cell's index are relative to the start),
which is very dissimilar to how brainfuck handles it. In brainfuck "simply going to cell 2" requires us to use a variable amount
of pointer shifting operators, which in term require us to know our current cell location in order to
get the difference between cell A and B and then shift the pointer by that difference.

So, made short, we need the compiler to know where the tape pointer is at all points.
For example, if the compiler's assumption about the tape pointer is wrong, let's say it thinks we are at 5, but we are at 0,
and then we try to access 6 it would increase the pointer by 1. This would put us on cell 1 rather than 6.
Effectively, this offsets us by -5, which is not a good thing (in most cases).

Normally, the assumed pointer position is always right, because the language does not let you invalidate it
at the exception of `ASUM` and possibly `RAW`.

But why would we need to mess with the pointer? To be able to unlock the power of relative adressing.
Most likely you will want to write brainfuck so you can forgo static adressing. The compiler doesn't quite know
what you want to do when writing brainfuck, which is where enters `BBOX` and `ASUM`.
`BBOX` can be invoked before your brainfuck code to guarenty the pointer's position.
`ASUM` can be invoked after your brainfuck code to correct for pointer shifts you have made that the compiler is not aware of.

Here's an example of all three instructions in action:
```basm
// prints out a string, strings are stored preceded and followed by a "-1" cell
[@PSTR string_start string_end]
BBOX string_start+1; // we put the pointer on the first non-"-1" cell
RAW "+[-.>+]-";      // we output every cell until we reach the -1 cell denoting the end
ASUM string_end;     // we can assume the pointer is currently at the end of the string
```

**BELOW IS CURRENTLY INVALID AS IT CHECKS FOR 0 FOR ARRAY BOUNDS.**
You may have noticed in the example that we could have simply used our `OUT` and `WHNE` instruction instead of using `.` and `[]`.
Despite what I said prior, invalid assumed tape pointers aren't inherently bad if you know how to use them.
We could write the example above using hybrid basm/bf, like so:
```basm
[@PSTR string_start string_end]
BBOX string_start+1;
ASUM 0;      // 0 is going to be equal to our current pointer position (offsetting by string_start+1)

WHNE 0 0 [
    OUT 0;
    BBOX 0;  // although the instructions in this example do not move the pointer around, it is important to make sure the pointer is reset
    RAW ">"; // increases the offset by 1
]
ASUM string_end;
```

## Language Items
In basm, every instruction is formed by a sequence of language items.
For example, `ADDP addr1 addr2;` would be `[ident, expression, expression, declaration_delimitor]`.

### Value Expressions
Expressions in basm are very simple, they are formed by an alias or literal,
possibly offset by another alias or literal and represent a value.
Here are examples of expressions:
* `732` (number literal)
* `10+9` (number literal offset by another number literal)
* `'a'` (character literal, gets interpreted as it's ASCII value)
* `my_alias` (alias)
* `my_alias-1` (alias offset by number literal)

Written 2024-11-23
# Brain Aneurysm

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
Strings in basm are assumed to start and end with a cell of value zero. Thus making the smallest string use up two cells.
When providing the adress to it use to one referencing the starting zero cell (which is going to be the same as the one you used to declare it).
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
| INLN | ident (scope) | inlines an aliased scope |
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
| WHNE | addr, value, [instruction] | while the value of `addr` cell is not equal to `value` runs the `[instruction]`. `addr` is not consumed |

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
The two alias types (being value and scope) both share the same alias names, meaning an alias aliasing a value can be
overshadowed by a redefinition to a scope.

## Language Items
In basm, every instruction is formed by a sequence of language items.
For example, `ADDP addr1 addr2;` would be `[ident, expression, expression, declaration_delimitor]`.

### Expressions
Expressions in basm are very simple, they are formed by an alias or literal,
possibly offset by another alias or literal. (There can only be one add/sub per expression)
Here are examples of expressions:
* `732` (number literal)
* `10+9` (number literal offset by another number literal)
* `'a'` (character literal, gets interpreted as it's ASCII value)
* `my_alias` (alias)
* `my_alias-1` (alias offset by number literal)

Written 2024-11-23
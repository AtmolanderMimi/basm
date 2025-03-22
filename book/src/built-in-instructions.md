# In-built Instructions

Basm uses a very reduced set of in-built instructions.
This means that any instruction that could be easily reproduced by combining
two or more other instructions were not included with very few exceptions.
For example, there is no multiplication instruction built-in.
I said the instruction set was small and, in fact,
the set of built-in instructions is so reduced that I can show them all here:

### Arithmetic
| Name | Arguments | Function |
|-|-|-|
| **ZERO** | addr | sets the value of `addr` to 0 |
| **INCR** | addr, value | increments the value of the `addr` cell by `value` |
| **DECR** | addr, value | decrements the value of the `addr` cell by `value` |
| **ADDP** | addr1, addr2 | adds `addr2` to `addr1` in place |
| **SUBP** | addr1, addr2 | substract `addr2` from `addr1` in place |
| **COPY** | addr1, addr2, addr3 | copies the value of `addr1` into `addr2` and `addr3` |

### Control Flow / Loop
| Name | Arguments | Function |
|-|-|-|
| **WHNE** | addr, value, [scope] | while the value of `addr` cell is not equal to `value` runs the `[scope]`. `addr` is not consumed |

### I/O
| Name | Arguments | Function |
|-|-|-|
| **IN**   | addr | takes input form the user and sets it in `addr`, behaviour will vary between bf implementations |
| **OUT**  | addr | send `addr` to the output, `addr` is not consumed |
| **LSTR** | start_addr, "str" | loads the string character by character into cells from the `start_addr` |
| **PSTR** | addr, "str" | prints the string character by character using the cell at `addr` to buffer the bytes |

### Language / Compilation
| Name | Arguments | Function |
|-|-|-|
| **ALIS** | ident, value | aliases a value or scope to an identifier, this instruction is purely abstraction |
| **INLN** | [scope] | inlines a scope |
| **RAW**  | "str" | inlines the string in the transpiled code, this can be used to include brainfuck operators |
| **BBOX** | addr | moves the tape pointer to the address of `addr` |
| **ASUM** | addr | tells to compiler to assume that the tape pointer is at `addr`, if that assumption is wrong all cells accesses will be offset |

You can find this table of instructions in the reference section of this book.

## Types
Seeing this chart you might be a little confused about how some of the arguments are formatted.
If confusion lasts for more than eight hours contact a doctor (or continue reading).
In basm source code each value is typed and totally static,
meaning that values within the source code don't get mutated at runtime,
only cells in the tape can be mutated.
(basm is like on big set of macros, it would be for values to be mutable at runtime)
Basm has 3 types for these value being:
* `number` denoted by nothing
* `scope` denoted by being surrounded by `".."`
* `string` denoted by being surrounded by `[..]`

Instructions will only take arguments of the corresponding type to the one of their defined arguments.
Usually their arguments are ordered by their type and meaning.
The way I like to order them follows this little diagram:
```
address numbers -> value numbers -> string -> scopes -> stack pointer number
```

### Number Type
Numbers in basm are the most common type for literals and are probably going to be the type you use most
as they are very versatile.

As you may have seen in the table above numerics usually have one of two purposes,
either they serve as an address to a cell or they serve as a value.
For example, `INCR` takes in the arguments `addr` and `value` which are both numeric,
but their value hold two very different meaning.
The value of the passed in `addr` denotes the address of the cell to be operated on.
Wheras the value of the passed in `value` denotes how much the cell at `addr` should be increased.
Be sure to not mix numerics representing address and value as that is a surefire way to make bugs!

There is three ways to write a number literal:
* Simply via a positive number, ex: `0`, `42`, `732`
* Via a character literal, ex: `'b'`, `'F'`
* Or by combining two number literals into an expression, ex: `3+'a'`

Character literals' values are mapped to their value in Unicode.
Expression are interesting as they allow modularity to arguments.
If I have an array starting at index `3` and I want to zero the first 3 tiles in that array,
I could elegantly write it like so:
```basm
ZERO 3+0;
ZERO 3+1;
ZERO 3+2;
```

Another interesting property about expressions is that they themselves are number literals,
meaning that we can cue them together like to.
```basm
// reminder that values in sources files are computed at compile-time
//     75  - 32  - 1 = 42
INCR 0 'K' - ' ' - 1;
OUT 0;
```

The way expressions are built is that they take a "base" value and then search for a "modifier".
A modifier has a certain value which they apply to the base,
that value being the numeric value directly at the right of it.
It then merges the two values (the base and modifier value) into one expression and repeats.
Currently the only two modifiers types which are implemented are addition and substraction
which are indentified via `+` and `-` respectively.

The example above with explicit priority would look like this:
*(this is not valid basm syntax)*
```
((75) - 32) - 1
```

### Scopes
Scopes has we have spoken in the chapter before are collections of instructions and other scopes.
You can pass them into functions by simply writing them in their literal form.
Here's a reminder of what a scope literal looks like:
```basm
[
    // ...content here...
]
```

### String
Strings as types are very limited in usage,
they cannot be aliased and you cannot make a meta-instruction with a string arugment.
This means basically that the string type is only used by in-built instructions
(i.e: `PSTR`, `LSTR` and `RAW`).
Whilst writing code, it is very rare to actually end up working with string anymore then
loading them into memory or printing them out.
Because of this string support is currently fairly lacking,
you cannot operate on them in any meaningful way in basm 1.0.
You only have to power to summon them through their literal form,
which like many programming languages is simply text between two quotes (`"`).
For example:
```basm
RAW "this will be included in the transpiled file!
";
```

Noticed how I needed to actually include a newline character
in the source file for the string to contain it?
This is because basm does not contain escapeable characters like `\n` as of now.
*It also means that you can't write a string containing `"` (or a character literal with `'`).
That's because the lexer, in the state it currently is,
would freak out if I ever tried to implement escaped characters.
(I **really** need to rewrite the lexer)*

## Assumptions Made by Instructions
In basm, instructions "consume" the cells at source addresses unless otherwise noted.
"consuming", in this context, means that the cell is set to zero.
For example, when we add two numbers toghter using `ADDP`, the second, non output cell,
will be inadventenly be set to 0 due to the nature of the operation we made on it.

Instructions also assume that address arguments don't alias (aka have the same value).
An instruction statement with two same addresses mostly just ends up creating
programs which will never end by creating an endless loop,
although they can also cause other undesireable behaviour.

Also, notably, Instructions assume that the cells at output addresses are 0.
If that condition is not met, an instruction which *"sets a cell to a value"* would rather *"increment a cell by that value"*.

## Notable Instructions
Now that you know a bit more about what constitues an instruction, let's look at some notable
instructions which may not be immediatly intuitive just by reading the description.

### WHNE (While Not Equal)
**Arguments**
| Name | Type | Description |
| - | - | - |
| addr | number | address of the cell being compared against the `value` |
| value | number | value that the cell at `addr` must reach to stop the loop |
| scope | scope | code to be executed in loop while the condition is not resolved |

This instruction is vital, not only for writing good software in basm,
but for writing software that can be considered turing-complete in general.
`WHNE` is the only source of conditional execution and looping
included whitin the whole pool of built-in instructions (excluding `RAW` because it's weird).
Thus, all sources of conditionality needs to be derive from this
It is a sad day to be alive, but bf has no other primitive form of conditional programming.
This instruction translates directly to a matching pair of `[` and `]` bf operators.
As the name implies the behaviour is to:
1. Check if cell at `addr` is equal to `value` if it is then exit
2. Execute the code in `scope`
3. Return to step 1

Unlike most other operations WHNE does not consume the value it cell it checks, meaning you don't have
to reinitialise the cell after each check.

**Example:** This makes it easy to create something like a counter
```basm
// outputs values from 0-99
WHNE 0 100 [
    OUT 0;
    INCR 0 1;
];
```

### COPY
**Arguments**
| Name | Type | Description |
| - | - | - |
| addr1 | number | address the source cell to be copied to `addr2` and `addr3`, the cell at `addr1` is consumed |
| addr2 | number | address of a cell receiving a copy of the cell `addr1` |
| addr3 | number | address of a cell receiving a copy of the cell `addr1` |

`COPY` is weird one who is needed because of the weird nature of the weird language which is bf.
there is no way to add two numbers togheter without one disappearing into the void,
never to be seen again. This is why we require `COPY`,
if we want operate on a value and still have it afterwards we will need to have it twice.
Once to use for the operation and the other to keep it alive.
`COPY` was made for this, it takes in one cell and sets the value of two other cell
to the value of the source `addr1` cell, consuming it in the process.
This behaviour is essential for building almost any program.

**Example:** a program outputting the double of the last number forever, or when the cell limit is reached:
```basm
INCR 0 1;
WHNE 0 0 [
    OUT 0;
    COPY 0 2 3;
    
    ADDP 0 2;
    ADDP 0 3;
];
```

### ADDP (Add in Place)
**Arguments**
| Name | Type | Description |
| - | - | - |
| addr1 | number | output address |
| addr2 | number | input address to be added to `addr1` |

You might be surprised to see `ADDP` in this list, after all it's an addition operation,
how complicated can it be?
Welp, not that much, but it does have a property which is not reflected by its name.
That being the ability to act as a move instruction.
As long as the value at `addr1` is 0, 
calling `ADDP` will simply move the contents of the cell at `addr2` to `addr1`.
As thus we can give `ADDP` a little pet name like `MOVE` as it effectively serves as so.

**Example:** I want to copy the value of cell 0 to cell 1 without consuming it
```basm
INCR 0 42;

// reminder that we can't copy to 0 directly as that would make two pointers alias,
// which in turn would cause an infinite loop
COPY 0 1 2;

// we "move" 2 -> 0
ADDP 0 2;
```

### RAW
**Arguments**
| Name | Type | Description |
| - | - | - |
| str | string | a string to be included in the transpiled code at the appropriate location |

`RAW` is a bit of a special case in that it can do the job of all the other instructions.
If I truely wanted to reduce the instruction set to it's max we would have one instruction
and it would be this.
Now, including a string in the compiled output does not seem like it would be able to do much at first.
*Like, ok, we can put comments in the output?*
But that's not the main attraction of `RAW`, the real reason why it is of interest is because you can
smuggle in **ANY** string with you.
This means that you could add characters like `+`, `-`, `>`, `<`, `[`, `]`, `,` and `.`
allowing us to insert raw bf code directly in our basm source file.

With this power, we can havest the full potential (it being rather small still) of bf.
We can write dynamic code to our heart content without being limited by basm's static memory adressing.
It all comes with a downside though, any code included within `RAW` is not checked and,
as thus, simply writing `RAW ">";` is enough to completely mess up your program by effectively offsetting all of the cells by 1.
The art of writing basm a program capable of harnessing the power of dynamic memory adressing
is one hard to come by these days,
but in time (reading the chapter on relative code) you shall learn how to master it.

**Example:** Why would I want to use basm, when bf worked just fine for a "hello, world!"?
```basm
// be careful to not include bf operators in your text!
RAW "my hello world program:
";
RAW "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
```

Which will give us exacly this output, as expected:
```bf
my hello world program:
++++++++[>++++[->++>+++>+++>+<<<<]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>->.+<.<.+++.------.--------.>>.>++.
```

### ALIS (Alias)
Oh, wait, we have tho whole next chapter just for that one!

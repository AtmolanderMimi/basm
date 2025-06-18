# In-built Instructions

Any basm source file start with some predefined instructions called called in-built.
These instructions are valid wherever and whenever.
Basm has very reduced set of in-built instructions for simplicity.
This means that any instruction that could be easily reproduced by combining
two or more instructions was not included (with some few exceptions).
For example, there is no multiplication instruction built in.
I said the instruction set was small and, in fact,
the set of built-in instructions is so reduced that I can show the 16 of them all here:

### Arithmetic


| Name     | Arguments           | Function                                                                  |
| ---------- | --------------------- | --------------------------------------------------------------------------- |
| **ZERO** | addr                | sets the value of`addr` to 0                                              |
| **INCR** | addr, value         | increments the value of the`addr` cell by `value`                         |
| **DECR** | addr, value         | decrements the value of the`addr` cell by `value`                         |
| **ADDP** | addr1, addr2        | adds`addr2` to `addr1`, the result is stored in `addr1` (in place)        |
| **SUBP** | addr1, addr2        | substract`addr2` from `addr1`, the result is stored in `addr1` (in place) |
| **COPY** | addr1, addr2, addr3 | copies the value of`addr1` into `addr2` and `addr3`                       |

### Control Flow / Loop


| Name     | Arguments            | Function                                                                                         |
| ---------- | ---------------------- | -------------------------------------------------------------------------------------------------- |
| **WHNE** | addr, value, [scope] | while the value of`addr` cell is not equal to `value` runs the `[scope]`. `addr` is not consumed |

### I/O


| Name     | Arguments         | Function                                                                                       |
| ---------- | ------------------- | ------------------------------------------------------------------------------------------------ |
| **IN**   | addr              | takes input from the user and sets it in`addr`, behaviour will vary between bf implementations |
| **OUT**  | addr              | outputs the value of`addr`, `addr` is not consumed                                             |
| **LSTR** | start_addr, "str" | loads the string character by character into cells from the`start_addr` advancing forward      |
| **PSTR** | addr, "str"       | prints the string character by character using the cell`addr` as a buffer                      |

### Language / Compilation


| Name     | Arguments               | Function                                                                                                                     |
| ---------- | ------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| **ALIS** | ident, value or [scope] | creates an alias to a value or scope named`ident`. This instruction is purely abstraction                                    |
| **INLN** | [scope]                 | inlines a scope                                                                                                              |
| **RAW**  | "str"                   | includes the string after transpilation, this can be used to include brainfuck operators                                     |
| **BBOX** | addr                    | moves the tape pointer to`addr`                                                                                              |
| **ASUM** | addr                    | tells to compiler to assume that the tape pointer is at`addr`. If that assumption is wrong all cells accesses will be offset |

You can find this table of instructions in the reference section of this book.

## Types

Seeing this chart you might be a little confused due to how arguments in the `Arguments` column are formatted.
*If confusion lasts for more than eight hours contact a doctor (or continue reading)*.

Of course, there is a reason why arguments are denoted differently, it's because they don't have the same expected type!
Each value in basm source code is typed and totally static,
meaning that values within the source code can't get mutated at runtime.
Only the value of cells in the tape can be mutated during execution.
(basm is like one big set of macros, it would be for impossible for values to be mutable at runtime)

Basm has 3 types for its source code values being:

* `number` denoted by nothing
* `scope` denoted by being surrounded with `[..]`
* `string` denoted by being surrounded with `".."`

Instructions statements will only take arguments with types corresponding to the instruction's arguments' types.
You cannot pass in a string where a number is expected, types will not coerce.
Instructions in the built-in set have arguments which are ordered by their type and meaning.
The way I like to order them follows this little diagram:

```
address numbers -> pure numbers -> strings -> scopes -> stack pointer number
```

### Number Type

Numbers in basm are the most common type for literals and are probably going to be the type you use most
as they are very versatile.

As you may have seen in the table above, numerics usually have one of two purposes.
Either they serve as an address to a cell or they serve as a *pure number*.
For example, `INCR` takes in the arguments `addr` and `value` which are both numeric,
but their values hold two very different meanings.
The value of `addr` (address) denotes the address of the cell to be operated on.
Whereas the value of the passed in `value` (pure number) denotes by how much the cell at `addr` should be increased.
Be sure to not mix numerics representing address and and numerics representing (pure) numbers as that is a surefire way to create bugs!

There is three ways to write a number literal:

* Via a positive number, ex: `0`, `42`, `732`
* Via a character literal, ex: `'b'`, `'F'`
* Or by combining two number literals into an expression, ex: `3+'a'`

Character literals' values are mapped to their value in Unicode.
Expression are interesting as they allow modularity to arguments.
If I have an array starting at index `3` and I want to zero the first 3 cells in that array,
I could elegantly write it like so:

```basm
ZERO 3+0;
ZERO 3+1;
ZERO 3+2;
```

Another interesting property about expressions is that they themselves are number literals,
meaning that we can chain them together like to.

```basm
// reminder that values in sources files are computed at compile-time
// expressions don't resolve at runtime!
//     75  - 32  - 1 = 42
INCR 0 'K' - ' ' - 1;
OUT 0;
```

The way expressions are built is that they take a "base" value and then a "modifier".
It merges the two values (the base and modifier value) into one expression and repeats.
Currently, modifiers have four types which are implemented.
* Addition: `+`,
* Substraction: `-`,
* Multiplicaton: `*`,
* Integer division (towards 0): `/`

Basm uses left to right priority, totally forgoing PEMDAS and any parentheses to denote priority.
Here would be the implicit priority of that last example:
```
((75) - 32) - 1 = 42
```

A little note on integer division modifiers, basm doesn't do decimal numbers.
This means that division are kinda weird as they can't result a decimal number.
So, all divisions made are truncated (practically, the decimal digits are removed) towards 0.
As a result, `5/3`, which is in normal maths equal to 1.6666.., is equal to 1 in basm.
This also introduces a loss of precision. `10/3*3` is not in fact 10, but 9.
Since we execute from left to right we first compute `10/3` which is 3.3333..
That gets rounded to 3 and then we execute the multiplication modifier on our rounded 3, so we get 9!

#### The Difference Between Address Numbers and Pure Numbers

You may have noticed that I separate numbers into two "sub-types" based on their meaning.
There are "address numbers" which are numeric values which represent addresses to cells and
there are "pure numbers" which are numeric values which represent numeric values used for operations.
Nothing stops you from using address numbers where pure numbers belong. This is also true the other way around.

This distinction is going to be important once we start naming these numeric values with aliases.
You will see, I prefix address number aliases with `A` for "address"
and pure value aliases with `V` for pure number "value".

In the built-in's' tables above, arguments named `addr` or `addr_nth` are **address numbers**,
whereas arguments named `value` are **pure numbers**.

### Scopes

Scopes, as I described in the last chapter, are collections of instructions and other scopes.
You can pass them into instructions as arguments by simply writing them in their literal form (which means the same thing as expression here).
Here's a reminder of what a scope literal looks like:

```basm
[
    // ...content here...
]
```

### String

Strings as types are very limited in usage,
they cannot be aliased and you cannot make a meta-instruction with a string arugment.
This means, basically, that the string type is only used by in-built instructions
(i.e: `PSTR`, `LSTR` and `RAW`).
Whilst writing code, it is very rare to actually end up working with string anymore than
loading them into memory or printing them out.
Because of this, string support is currently fairly lacking.
You cannot operate on source code strings in any meaningful way in basm 1.0.
The only power you hold is to summon them through their literal form,
which like many programming languages is simply text between two quotes (`"`).
For example:

```basm
RAW "this will be included in the transpiled file!
";
```

Noticed how I needed to actually include a newline character
in the source file for the string to contain it?
This is because basm does not contain escapable characters like `\n` as of now.
*It also means that you can't write a string containing `"` (or a character literal with `'`).
That's because the lexer, in the state it currently is,
would freak out if I ever tried to implement escaped characters.
(I **really** need to rewrite the lexer)*

## Assumptions Made by Instructions

All instructions both the ones you make and the ones which are built-in will hold some sort of assumptions.

In basm, instructions "consume" the values of the source cells unless otherwise noted.
"consuming", in this context, means that the cell is set to 0.
For example, when we add two numbers together using `ADDP`, the second cell, which is not an output,
will inadvertently be zeroed due to the operation we made on it.
Cells which hold the result of operations are of course not zeroed.

Apart from that, instructions also assume that address arguments don't have the same value (aka "alias", but that word has another meaning in basm).
An instruction statement passed in with two of the same addresses have unspecified behaviour.
Most often, this ends up locking the program in an endless loop,
although it can also cause other undesirable behaviour.

Lastly, notably, instructions assume that the values of the cells at output addresses are 0.
If that condition is not met, an instruction which *"sets a cell to a value"* would rather *"increment a cell by that value"*.

## Notable Instructions

Now that you know a bit more about what constitutes an instruction, let's look at some notable
instructions which may not be immediately intuitive just by reading their description.

### WHNE (While Not Equal)

**Arguments**


| Name  | Type   | Description                                                     |
| ------- | -------- | ----------------------------------------------------------------- |
| addr  | number | address of the cell being compared against the`value`           |
| value | number | value that the cell at`addr` must reach to stop the loop        |
| scope | scope  | code to be executed in loop while the condition is not resolved |

This instruction is vital, not only for writing good software in basm,
but for writing any software that can be considered Turing-complete in general.
`WHNE` is the only source of conditional execution and looping
included within the whole pool of built-in instructions (excluding `RAW`, because it's weird).
Thus, all sources of conditionality need to be derive from this.
It is a sad day to be alive, but bf has no other primitive form of conditional than a while not equal to zero with the `[` and `]` operators.
This instruction translates directly to a matching pair of `[`and`]` alongside some `+` and `-` so that we aren't restricted to equality with zero.
As the name implies the behaviour is to:

1. Check if cell at `addr` is equal to `value`. If it is, exit
2. Execute the code in `scope`
3. Return to step 1

Unlike most other operations WHNE does not consume the value of the cell it checks, meaning you don't have
to reinitialize the cell after each check.

**Example:** The fact that the cell is not consumed makes it easy to create something like a counter

```basm
// outputs values from 0-99
WHNE 0 100 [
    OUT 0;
    INCR 0 1;
];
```

### COPY

**Arguments**


| Name  | Type   | Description                                                                                 |
| ------- | -------- | --------------------------------------------------------------------------------------------- |
| addr1 | number | address the source cell to be copied to`addr2` and `addr3`, the cell at `addr1` is consumed |
| addr2 | number | address of a cell receiving a copy of the cell`addr1`                                       |
| addr3 | number | address of a cell receiving a copy of the cell`addr1`                                       |

`COPY` is a *weird* one which is needed because of the *weird* nature of the *weird* language which is bf *(weirdly)*.
There is no way to add two numbers together without one disappearing into the void, never to be seen again.
This is why we require `COPY`.
If we want operate on a value and still have a copy of it afterwards we will need to have the value twice.
Once to use for the operation and the other to keep it alive.
`COPY` was made for this, it takes in one cell and sets the value of two other cells
to the value of the source `addr1` cell, consuming it in the process.
This behaviour is essential for building almost any program.

**Example:** a program outputting the double of the last number forever, or until the cell value limit is reached:

```basm
INCR 0 1;
WHNE 0 0 [
    OUT 0;
    COPY 0 2 3;
  
    ADDP 0 2;
    ADDP 0 3;
];
```

*Now, most basm professionals don't want you to know this... but it's acceptable for the two output addresses of `COPY` to be the same.*
Doing so will copy the value of the cell twice in the same place, practically doubling it.
Although I know it is safe in this circumstance to have two address arguments with the same value due to the fact that *I kinda made the language*,
this is not a safe bet to take on *ANY OTHER* instruction! (pretty cool though)

### ADDP (Add in Place)

**Arguments**


| Name  | Type   | Description                                       |
| ------- | -------- | --------------------------------------------------- |
| addr1 | number | cell to be added to`addr2`, output is stored here |
| addr2 | number | cell to be added to`addr1`                        |

You might be surprised to see `ADDP` in this list, after all it's an addition operation,
how complicated can it be?
Welp, not all that much, but it still does have a property which is not reflected by its name.
That being the ability to act as a move instruction.

As long as the value at `addr1` is 0,
calling `ADDP` will simply move the contents of the cell at `addr2` to `addr1`.
This is because `x + 0 = x`, or in layman's terms: adding something to nothing will just return the initial something.
And, since we happen to return that "initial something" to somewhere different from where we took it from, we effectively moved that "something".
So, we can give`ADDP`a little pet name like`MOVE` as it really serves as so sometimes.

**Example:** I want to copy the value of cell 0 to cell 1 without consuming it

```basm
INCR 0 42;

// reminder that we can't copy to 0 directly as that would make two pointers alias,
// which in turn would cause an infinite loop
// (no, the doubling trick with COPY was not about that)
COPY 0 1 2;

// we "move" 2 -> 0
ADDP 0 2;
```

### RAW

**Arguments**


| Name | Type   | Description                                                                |
| ------ | -------- | ---------------------------------------------------------------------------- |
| str  | string | a string to be included in the transpiled code at the appropriate location |

`RAW` is a bit of a special case in that it can do the job of all the other instructions.
If I truly wanted to reduce the instruction set to the maximum there would be one instruction
and it would be this.

Now, including a string in the compiled output does not seem like it would be able to do much at first.
*Like, ok, we can put comments in the output?*
But that's not the main attraction of `RAW`. The real reason it is of interest is because you can
smuggle **ANY** string into the compiled file.
This means that it is possible to include operators like `+`, `-`, `>`, `<`, `[`, `]`, `,` and `.`
allowing us to insert raw bf code directly in our basm source file.
With this, we can harvest the full power (it being rather small still) of bf.
This comes with one downside, tape pointer movement, which is usually completely handled by the compiler,
and unmatched brackets are not checked within strings passed to `RAW`.
An unmatched bracket will probably just match with a totally unrelated one, messing up jumps made by `WHNE`
and messing with tape pointer position **unintentionally** is **BAD**.
I won't go into much details why it is, but simply put, the compiler needs to know were the pointer is at all times
(unless you plan for it, we'll see that in the relative-code chapter).
Just writing `RAW ">";` is enough to completely mess up your program by effectively offsetting all the cells by 1.

With `RAW "">";`, we can write dynamic code to our heart content without being limited by basm's static memory addresssing.
The art of writing basm a program capable of harnessing the power of dynamic memory addresssing
is one hard to come by these days,
but in time (reading the chapter on relative code) you shall learn how to master it.

**Example:** Why would I want to use basm, when bf worked just fine for a "hello, world!"?

```basm
// be careful to not include bf operators in your text!
RAW "my hello world program:
";
RAW "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
```

Which will give us exactly this output, as expected:

```bf
my hello world program:
++++++++[>++++[->++>+++>+++>+<<<<]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>->.+<.<.+++.------.--------.>>.>++.
```

### ALIS (Alias)

Oh, wait... we have the whole next chapter just for that one!

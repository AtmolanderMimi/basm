number# Writing Relative Code

Onto the last remaining chip of knowlege that basm requires: Harvesting the Power of Relative Code!
Remember at the very start of this book, where I said that every memory access is static unless otherwise stated?
Well, this chapter is about stating you don't want static accessing and would much prefer write relative logic.
Sometimes static addressing is either less efficient, harder or simply impossible to use for certain problems.
In these cases you will need to reach for relative logic.

But, what is the difference between relative and static in basm exactly?
By default, basm refers to cells in a static manner, meaning that an address will always refer to the same cell (e.g address `1` will always refer to the same cell).
This kind of addressing is known as static.
Relative means that an address will not always refer to the same cell,
it will refer to a cell relative to a dynamic point.
In most cases, that "relative point" is simply where your tape pointer is at a given moment.

Now, by default basm moves your tape pointer automatically in order to access specific addresses.
Basm won't lose track of where the tape pointer is unless you explcitly tell it to do so.
When you ask to operate on a cell, basm will have remembered where the tape pointer currently is, and it will then try to move the tape pointer by the delta between the current position and the desired position.
The tape pointer position that is remembered by the compiler is also known as the *assumed* tape pointer possition,
as it can get temporarly invalidated when working with relative code.
For example, if we have the tape pointer at position `3` and ask to reach cell `5`, basm will automatically insert `>>`.
There are three instruction made specifically for dealing with the tape pointer
and be able to write relative code from it.

## `BBOX` (Black Box)

**Arguments**


| Name | Type   | Description                                     |
| ------ | -------- | ------------------------------------------------- |
| addr | number | the address for the tape pointer to be moved to |

`BBOX` is probably the most simple instruction in the whole set of built-in ones.
Its one and only purpose is to guarenty the position of the tape pointer at a certain point in the program.
All it does it move the tape pointer to `addr` and does *nothing*.
This behaviour is very useful for other instruction that work off of the assumption that the tape pointer is positioned
at a certain cell.

## `ASUM` (Assume)

**Arguments**


| Name | Type   | Description                                             |
| ------ | -------- | --------------------------------------------------------- |
| addr | number | the address overiting the assumed tape pointer position |

This instruction is tightly integrated with the compiler.
What it does is that it overwrites the assumed tape pointer position with the specified `addr`.
This can serve either to put the program into a relative state (where the assumed pointer position is wrong)
or to set the assumed pointer position back to a valid value.

When combined with `BBOX` it can serve to offset the whole address field.
Think about it, if the real tape pointer position is 1, but you tell the compiler it's 0 through `ASUM`,
then the assumed pointer position will be offset by +1.
Trying to access cell no.0 would lead to accessing cell no.1.
When this happens, I like to call this program state the "relative state", all accesses are relative to 1.
So, when this happens, all addresses become relative to the offset.

```basm
INCR 1 42;
OUT 1; // returns '*'

// offsets the pointer by +1 (so cells by -1)
BBOX 0;
ASUM 1;

OUT 0; // returns '*'
```

This may all see like too much pedantry for something as simple as offsetting all addresses by one.
Yes, in this case it is, but that's not what makes relative code interesting.
Relative programming in basm starts becoming truly interesting when you loop this offset.
Once you start looping, your offset cannot be known at compile time.
Looping offsets are even the main reason why anyone would want to write relative code as it allows
to create "flyers". Flyers are an operation which is applied while moving forwards (or backwards) through the tape.
The word "flyer"/"glider" comes from the glider in Conway's Game of Life where it is a self replicating structure moving through the board.
In basm, flyers can include data cells which move with the shift in assumed pointer position.
I think it is easier to understand them in bf:

```bf
[.>]
```

This is a basic flyer in bf. It outputs every cell until it reaches a cell with a value of 0.
It may seem like it is useless at first, but it can serve to print strings in memory loaded via `LSTR`.
This flyer takes advantage of the fact that strings are not constituted of arbitrary data and that they will not contain a zero cell *(at least not until you the string's end)*.

We can easily create this pattern in basm with `BBOX` and `ASUM`:

```basm
// dynamic print string
// Astart is the first cell
// Aend is the 0 cell right after the last non-zero cell
[@DPSTR Astart Aend] [ // "Dynamic Print String"
    BBOX Astr;
    ASUM 0;

    WHNE 0 0 [
        OUT 0;

        // move forward
        BBOX 1;
        ASUM 0;
    ];

    // we set the pointer back to the valid assumption after we are done
    ASUM Aend;
]
```

*(If you compile the above example with a `[main]` field with only `DPSTR 0 0;`, it will compile to exactly `[.>]`)*

Most notably, this instruction will return the assumed pointer position to a valid value after running.
Logic making use of relative state should always use `ASUM` to set back the assumed pointer position
to a valid state.
Since all basm code relies on the fact that addressing is static,
leaving it in a relative state will no doubt make your program bug!

## `RAW`

**Arguments**


| Name | Type   | Description                                                                                |
| ------ | -------- | -------------------------------------------------------------------------------------------- |
| str  | string | inlines the string in the transpiled code, this can be used to include brainfuck operators |

`RAW` is inherently relative by default, as any moves made in it is not considered by the compiler.
As thus, writing `RAW ">";` has the same effect using `BBOX 1; ASUM 0;` like before.
You have access to the whole of bf, which means that you can very easily cause offsets and write flyers.
While I prefer using the more basm'ic solution, `RAW` is a valid solution for your relative programming needs.

Next chapter will focus on applying relative knowlege to create a dynamically indexable array.

# Synthesis: Making a Bf Interpreter

This chapter will focus entirely on the conception process of a bf interpreter in basm,
which will in turn transpile to bf, to make a bf interpreter in bf!
It is **highly recommended** to have read all other chapters before diggin into this one.

## Meta-instructions "header"
While working on the logic of the interpreter,
I will use meta-instruction which we already defined earlier (notably `GETD` and `ADDD`).
Make sure to add these meta-instructions definitions before any other code that uses them:
```basm
// sets a value to a specific value by zeroing it before writing
[@SET Aaddr Vval] [
ZERO Aaddr;
INCR Aaddr Vval;
]

// copies the content of a cell whilst keeping the source (conservatively)
[@COPC Asrc Adst sp] [
	ALIS Atmp sp;
	COPY Asrc Adst Atmp;
	ADDP Asrc Atmp;
]

// if not equal conditional
[@IFNE Aaddr Vval [scp] sp] [
ALIS Atmp1 sp;
ALIS Atmp2 sp+1;
ALIS sp sp+2;

COPY Aaddr Atmp1 Atmp2;
ADDP Aaddr Atmp2;

WHNE Atmp1 Vval [
    ZERO Atmp1;

    INLN [scp];

    INCR Atmp1 Vval;
];

ZERO Atmp1;
]

// if equal conditional
[@IFEQ Aaddr Vval [scp] sp] [
    ALIS Aflag sp;
    ALIS sp sp+1;
    ALIS Vnot_equal 1;

    IFNE Aaddr Vval [
        INCR Aflag Vnot_equal;
    ] sp;

    IFNE Aflag Vnot_equal [scp] sp;
    ZERO Aflag;
]

// moves the value of the cell at Aarray[value of Aindex] to Adst
[@GETD Aarray Aindex Adst] [
ADDP Aarray+3 Aindex; 

BBOX Aarray+1;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1;
ALIS Aindex 2;
ALIS Aelement 3;

WHNE Aindex 0 [
    DECR Aindex 1
    INCR Areturn 1;

    ADDP Aswap Aelement;

    ADDP Aindex+1 Aindex;
    ADDP Areturn+1 Areturn;

    BBOX 1;
    ASUM 0;
];

ALIS Acell Aindex;

ADDP Acell Aelement;

ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1;

    ADDP Areturn-1 Areturn;
    ADDP Acell-1 Acell;

    BBOX 0;
    ASUM 1;

    ADDP Aswap Aelement;
];

BBOX Aelement;
ASUM Aarray+1;

ADDP Adst Aarray+3;
]

// adds the value of the cell at Asrc to the Aarray[value of Aindex] cell
[@ADDD Aarray Aindex Asrc] [
ADDP Aarray+3 Asrc;
ADDP Aarray+2 Aindex;

BBOX Aarray;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1;
ALIS Aindex 2;
ALIS Acell 3;
ALIS Aelement 4;

WHNE Aindex 0 [
    DECR Aindex 1;
    INCR Areturn 1;

    ADDP Aswap Aelement;

    ADDP Acell+1 Acell;
    ADDP Aindex+1 Aindex;
    ADDP Areturn+1 Areturn;

    BBOX 1;
    ASUM 0;
];

ADDP Aelement Acell;

ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1;

    ADDP Areturn-1 Areturn;

    BBOX 0;
    ASUM 1;

    ADDP Aswap Aelement;
];

BBOX Aelement;
ASUM Aarray;
]
```

## Memory Layout
Our bf interpreter's tape memory will consist of 3 parts:
* Memory for operations, aka operating memory
* Instruction array (with parking)
* Tape array (with parking)

In memory the sections should look like this:
```txt
[operation][instruction][tape]
```

The tape array will be allowed to grow infinitly, as there is no other allocated cells after it.
This is unlike the instruction array, which is stuck inbetween the operation and tape array,
meaning that we will need to reserve it a static size so that it does not overwrite the tape.
We can't expand infinitly before the 0'th cell, as using cells before the 0'th is invalid bf.
(not the one we are going to interpret, but the one our program is made of)

## Taking User Input
We will want the user to be able to input any bf program they find online
and paste it directly into the bf input.
We'll also need a way for the user to tell us that they are done inputting the program,
for this we can check for a specific character like `!` to stop instruction input.


To do this your first though might be to have a counter
representing the number of characters inputted, and add the input with that counter as index.
That is *fine*, with nothing more.
There are some issues, mostly performance (both compute and size), which arise from it.
We'll need all the space we can take,
and I assure you that any performance gain will be very appreciated.

First, using this method, we will end up storing any non-operator characters in our instruction array.
This both takes up space for no reason and slows down `GETD` and `ADDD`
(the more element in the array, the less efficient they become).

Second, we don't take advantage of the limited nature of the operators.
When storing a `+` with this method, we will actually store the `+` character, being 43.
Remember, most operations in bf take `O(n)` time, where `n: value of the cell`
Whereas in most languages, operations are in `O(1)` time, aka constant time.
This means that, if we can reduce the values representing operators,
this will speed up all operations on operators including indexing them,
since array indexing is just shifting them around (shifting is `O(n)`)

To address these issues, we will discard all non-operators characters
and we will map all operators to a value.
We can also use 0 to denote the start/end of the array contents,
since all cells are zero by default we won't need to add it manually.
We need to know the end to know when to stop execution.
(You could also store the array lenght to know when you reach the end of the contents,
but that will not be possible with our implementation)
Here is the mapping:
```txt
start / end -> 0
`+` -> 1
`-` -> 2
`>` -> 3
`<` -> 4
`[` -> 5
`]` -> 6
`,` -> 7
`.` -> 8
```

There is one more issue, although rather small compared to the two first ones.
That being, **`ADDD` is not made for this at all!**
Every time we would add a new operator,
we would need to travel through all the other elements of the array.
It's now *that* bad, since instruction input is a one time process,
but this is the best example where `GETD` and `ADDD` like meta-instruction should be avoided.

### Writing The Flyer
Instead of having a flyer go out for each operator,
we can simply write a specilized flyer to fill the array.
This input flyer, will be able to take advantage of the context to have zero state!
That's right, we don't care about where we are or where we are going, so no index.
Furthermore, since the array it will be constructing does not contain arbitrary data,
we can return without having to store a return cell.
For the return, we will look for 0, which is not mapped to any instructions,
but is present in the parking.

I'll write this logic into a program specific meta-instruction.
A program specific meta-instruction is a meta-instruction which is
only useful within the context of the program.
Custom instructions like `COPC` are not program specific,
because they are generic enough to be used in many circumstances.
This is not the case with what we will write.
When writing program specific meta-instructions, I like to give them snake case names, like aliases.
Don't worry meta-instruction names will not overwrite your aliases.

```basm
[@init_input Aarray] [
BBOX Aarray+4;
ASUM 0;

ALIS Aflag 1;
ALIS Vappended 1;
ALIS Vexit 2;

ALIS sp 2;
// we can still use sp, since we know the cells will be zeroed

WHNE Aflag Vexit [
    IN 0;

    // -- mapping operators --
    IFEQ 0 '+' [
        SET 0 1;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '-' [
        SET 0 2;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '>' [
        SET 0 3;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '<' [
        SET 0 4;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '[' [
        SET 0 5;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 ',' [
        SET 0 7;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '.' [
        SET 0 8;
        INCR Aflag Vappended;
    ] sp;

    // check for end
    IFEQ 0 '!' [
        ZERO 0;
        SET Aflag Vexit;
    ] sp;

    // if we added something, we need to move up one
    IFEQ Aflag Vappended [
        ZERO Aflag;

        BBOX 1;
        ASUM 0;
    ] sp;

    // we don't clear the last character, even if it was invalid,
    // as the next IN will overwrite the cell
];

// NEVER forget cleanup
ZERO Aflag;

// -- going back --
// because the last char will be '!', we need to offset by at least 1
BBOX 0;
ASUM 1;

// moves back until we reached the zeroed cells of the parking
WHNE 0 0 [
    BBOX 0;
    ASUM 1;
];

ASUM Aarray+3;
]
```

As you can see we once again use a flag for control flow.
In this case our flag can have three states:
* `0`: when nothing has happened
* `Vappend`: when a operation is appended to the array
* `Vexit`: when the '!' character is detected
This doesn't count as the flyer having a state properly speaking though.
It's more like operation memory than state, since we won't carry that flag along for each glider shift.

We also, despite having a flyer, use `sp`.
We can safely do this as we can assume that the un-visited parts of the array are still zeroed.
Also, our program safely overwites the flag when getting user input,
meaning that our flag will always comsume our flag, leaving nno trash on the array.
Once again, taking advantage of safe array assumptions saves us from using `ADDD`/`GETD` which operate on no assumtion. 

## Writing `[main]`
First and foremost, before defining the main logic define the cells we'll use.

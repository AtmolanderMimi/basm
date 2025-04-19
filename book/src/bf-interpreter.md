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
    IFEQ 0 ']' [
        SET 0 6;
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

## Memory Layout in `[main]`
First and foremost, before defining the main logic we'll define the cells for our program to use.

```basm
ALIS Voperating_memory 16;
ALIS Aprog_pointer 0;
ALIS Amem_pointer 1;
ALIS Aflag 2;
ALIS Vexit 1;
ALIS Vbracket 2;
ALIS Aoperator 3;
ALIS Acell 4;
ALIS Abracket_jump 5;
ALIS sp 6;

ALIS Vprog_array_cap 256;
ALIS Aprog Voperating_memory;
// we add 4 to take into account the parking of the program array
ALIS Amem Aprog+Vprog_array_cap+4;
```

There's alot to unpack here. Here are the most notable ones:

### `Voperating_memory`
This alias defines how many cells are reserved for general computation (that being all but the array).
The reserved space granted by `Voperating_memory` is required for us to store cells like `Aflag`.
It is also required for meta-instruction using `sp` have zeroed cells.
Otherwise, meta-instructions would reach into the program array, which would be bad for program state.

### `Aflag`
`Aflag` is *once again* a flag cell. It can have one of these values:
* `0`: nothing happened
* `Vexit`: program is done and should exit
* `Vbracket`: a bracket has jumped, so `Aprog_pointer` should be updated to `Abracket_jump`
(this needs to be done after all the other logic, which is why it is stored in the flag)

### `Aoperator` and `Acell`
Those two aliased cells will hold both the current operator and memory cell respectively.
Since we can't edit cells directly in their array, we need to keep them in a static place in memory.

### `Aprog`
This alias defines the start of the program array (at its parking).
The way it is defined is that it comes straight after the operating memory segment,
meaning that its address can be defined has the lenght of the operating memory segment!
This would cause an off by 1 error if basm started indexing cells from 1, but it indexes from 0,
so we are good!

`Aprog` also comes with `Vprog_array_cap` aka the capacity of the program array.
We don't inherently *need* it to define the array,
but we will need to define how far away we want the memory array to be from the program array.
Effectively, choosing how much space we allocated to the program array.
In this example, I set 256 as the array capacity,
because that is the max that you can index with unsigned 8 bit cells.
If you want to use bigger programs, you can increase this capacity,
but beware you will also need to use cells of more than 8 bits!

### `Amem`
All that I specified about `Aprog` applies to `Amem`.
The value is the start of the array (at its parking).
While the memory array is technically not limited by memory, since it cannot ovewrite other memory,
it is still practically limited by the (un)efficiency of indexing
and the maximum index storable in the cells.

### `Abracket_jump`
Is used to store the address of the operation matching the current bracket operation.
Our bracket jumping implementation will not set the program pointer directly to it,
since it still needs to do some cleanup beforehand.
That cleanup includes setting the current operation in `Aoperator` back in the array.
To do that we need to know the index of that operator,
which is always just the index in `Aprog_pointer`.
This is why we don't change the operator pointer right away and we need to store it.
You are probably understand it better when you'll see the code managing this yourself.

## Writing `[main]` Logic
The bread and butter of our logic is of couse going to be a loop.
We'll loop over every operator in the program until we reach the end,
applying the effects of them as we go.
To be more precise here's the broken down version of what the loop:
1. Get the operator at `Aprog_pointer`
2. Execute the operator specific logic
3. Set the operator back
4. Increase `Aprog_pointer`

Notice how we don't get the memory cell for every operator.
This is because it would be incredibly wasteful to get and set the operator for every operator,
because the cell mostly remains over operators. Only `>` and `<` change the cell index.
Think interpreting the very common pattern `+++`.
If we were to get the cell, increase it by one and then put it back 3 times it would be horrendus!
So, to combat this, `Acell` functions more like a cell "cache" then its `Aoperator` counterpart.
We will only get/set the memory cell when moving the tape pointer with `>` and `<`.

With all that said, let's start implementing the easy parts of the logic:
```basm
// .. add the header here
// .. define init_input here

[main] [
ALIS Voperating_memory 16;
ALIS Aprog_pointer 0;
ALIS Amem_pointer 1;
ALIS Aflag 2;
ALIS Vexit 1;
ALIS Vbracket 2;
ALIS Aoperator 3;
ALIS Acell 4;
ALIS Abracket_jump 5;
ALIS sp 6;

ALIS Vprog_array_cap 256;
ALIS Aprog Voperating_memory;
ALIS Amem Aprog+Vprog_array_cap+4;

// get program from user
init_input Aprog;

WHNE Aflag Vexit [
    ALIS Atmp sp;
    ALIS sp sp+1;

    // 1. load in operator
    COPC Aprog_pointer Atmp sp;
    GETD Aprog Atmp Aoperator;

    // 2. operator specific logic
    // + + + + +
    IFEQ Aoperator 1 [
        INCR Acell 1;
    ] sp;

    // - - - - -
    IFEQ Aoperator 2 [
        DECR Acell 1;
    ] sp;

    // > > > > >
    IFEQ Aoperator 3 [
        // store the old cell
        COPC Amem_pointer Atmp sp;
        ADDD Amem Atmp Acell;

        // get the new cell
        INCR Amem_pointer 1;
        COPC Amem_pointer Atmp sp;
        GETD Amem Atmp Acell;
    ] sp;

    // < < < < <
    IFEQ Aoperator 4 [
        // store the old cell
        COPC Amem_pointer Atmp sp;
        ADDD Amem Atmp Acell;

        // get the new cell
        // NOTE: this DECR may underflow the tape pointer
        DECR Amem_pointer 1;
        COPC Amem_pointer Atmp sp;
        GETD Amem Atmp Acell;
    ] sp;

    // TODO: implement '[' and ']'

    // , , , , ,
    IFEQ Aoperator 7 [
        IN Acell;
    ] sp;

    // . . . . .
    IFEQ Aoperator 8 [
        OUT Acell;
    ] sp;

    // check for the end
    IFEQ Aoperator 0 [
        INCR Aflag Vexit;
    ] sp;

    // set the operator back
    COPC Aprog_pointer Atmp sp;
    ADDD Aprog Atmp Aoperator;

    // NOTE: we'll add something here later...

    // increase program pointer
    INCR Aprog_pointer 1;
];
]
```

With that that's *seemingly* most of the bf interpreter done already!
(with the exception of `[` and `]`)
You can take it easy from here we're on the final strech.
There's no looping or conditionals, but we have enough to print some basic characters.
Try the following code: (don't forget to use `!` to end program input!)
```bf
outputs the answer to life the universe and everything
++++++++++++++++++++++++++++++++++++++++++++++++++++.--.
```

### The Needlessly Complicated Part
You may not have understood the part about `Abracket_jump` and `Vbracket` for `Aflag`.
It's time were we need to use them...
Welp, the bracket jumping logic not *that* bad.
My implementation is going to be the simplest one possible, which simply iterates over every operator.

When we get a `[` and have 0 or when we have `]` and have a non-zero value,
we follow this procedure to find the index of the opposing matching bracket:
1. Initiate a counter at 1
2. Initiate a search pointer at the current program pointer
3. Add/Substract the search pointer by 1
4. Get the operator at the search pointer
5. If the operator is a bracket and it is the **same as the original, add 1** to the counter.
Else, if the operator is a bracket and the **bracket is opposite, remove 1** to the counter
6. Set the operator back at the search pointer
7. If the counter is non-zero, repeat from nb.3

With this, our search counter should always end on the matching bracket!
*Or the program will loop forever/read oob, but most bf code has matching brackets.*
*So this is not a bit issue.*
Once we our we have our *searched* search counter,
we don't want to apply it right away for the reasons stated before.
We will instead opt into setting the flag to `Vbracket`
so we can apply it after having set the operator back.

Add these if statements to apply bracket logic:
```basm
// [ [ [ [ [
IFEQ Aoperator 5 [
    IFEQ Acell 0 [
        ALIS Acounter sp;
        INCR Acounter 1; // 1.
        ALIS Asearch_op sp+1;
        ALIS sp sp+2;

        // 2.
        // we don't need to define our search counter here,
        // we can use Abracket_jump directly
        COPC Aprog_pointer Abracket_jump sp;
        WHNE Acounter 0 [
            // 3.
            INCR Abracket_jump 1;

            // 4.
            COPC Abracket_jump Atmp sp;
            GETD Aprog Atmp Asearch_op;

            // 5.
            // if op == '['
            IFEQ Asearch_op 5 [
                INCR Acounter 1;
            ] sp;
            // if op == ']'
            IFEQ Asearch_op 6 [
                DECR Acounter 1;
            ] sp;

            // 6.
            COPC Abracket_jump Atmp sp;
            ADDD Aprog Atmp Asearch_op;
        ]; // 7.

        INCR Aflag Vbracket;
    ] sp;
] sp;

// ] ] ] ] ]
IFEQ Aoperator 6 [
    IFNE Acell 0 [
        ALIS Acounter sp;
        INCR Acounter 1; // 1.
        ALIS Asearch_op sp+1;
        ALIS sp sp+2;
    
        // 2.
        COPC Aprog_pointer Abracket_jump sp;
        WHNE Acounter 0 [
            // 3.
            DECR Abracket_jump 1;
    
            // 4.
            COPC Abracket_jump Atmp sp;
            GETD Aprog Atmp Asearch_op;
    
            // 5.
            // if op == '['
            IFEQ Asearch_op 5 [
                DECR Acounter 1;
            ] sp;
            // if op == ']'
            IFEQ Asearch_op 6 [
                INCR Acounter 1;
            ] sp;
    
            // 6.
            COPC Abracket_jump Atmp sp;
            ADDD Aprog Atmp Asearch_op;
        ]; // 7.

        INCR Aflag Vbracket;
    ] sp;
] sp;
```

And the extra conditional to check the flag and apply the bracket jump:
(you should add it at my *ominous* `NOTE`)
```basm
// if there was a jump, set the pointer to the matching bracket
// (having the pointer increment after is intentional)
IFEQ Aflag Vbracket [
    ZERO Aprog_pointer;
    ADDP Aprog_pointer Abracket_jump;
    ZERO Aflag;
] sp;
```

Whew... that, was, all.
So now, with this enormous blob of bracket logic written, we finally have a 100% functional bf interpreter in basm!!!

Try it out with this fancier "Hello Word!" example:
([taken from Wikipedia](https://en.wikipedia.org/wiki/Brainfuck#Hello_World!))
```bf
++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.
```

## What to Improve
You now own a bf interpreter, problem: it's slooooooooow...
# TODO write possible improvements to make (also fix newlines being taken as characters or interactive fib is just broken?)

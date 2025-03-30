# Meta-Instructions

Wow, only now are we onto meta-instructions!
Meta-instruction like functions in other programming languages, except that they aren't.
They are actually just multiple instructions joined into one, thus the name meta-instruction.
These composite instructions are expanded at compile time into their component built-in instructions.
So, they are actually more similar to macros or functions which always inline.

## The Meta Fields
To define a meta-instruction you need to use the second and only other field other than `[main]`,
which is the `[@META arg1 arg2]` field.
You can have zero or more meta-instructions fields present in a file before the `[main]` one.
This field allows you to create a meta instruction of the name `META` with the arguments following it,
in the case of the example they would be two numeric arguements named `arg1` and `arg2`.

Once defined a meta-instruction can be used anywhere a normal built-in instruction can,
even in other meta-instruction bodies (excluding those who came before and itself of course).

### Arguments
Meta-instructions can take zero or more arguments of numeric or scope type.
To specify the arguments you want your meta-instruction to take in,
you simply write they names (identifiers) in the field header:
* `[@META]`: would take no arguements
* `[@META arg1 arg2]`: would take two numeric arguments
* `[@META arg1 [scope]]`: would take one numeric argument followed by a scope argument

Basm will automatically create aliases named after the arguments reflecting the arguments passed in by the caller.
This means that you can refer to `arg1` like any other numeric alias in the scope of `[@META arg1]`.

## Naming
Meta-instructions cannot share the name of other instructions, must they be built-in's or other meta-instrutions.
Also, quick aside, while I prefer to keep the 4 capital leters naming convention for instruction names,
names abide by the same restrictions as aliases and are thus not limited to 4 letters or less.

## Examples
Meta-instructions are very practical to remove boilerplate.
Since I decided to greatly reduce the amount of built-in instructions,
some quality of life instructions can easily be created.

### `SET`
If I want to set the value of cell, but I know that it is not consumed,
then I would need to zero it before incrementing the cell, causing boilerplate.
It would be much easier to read if both zeroing and incrementing the cell to a value would be the same instruction.
Well, with meta-instructions you can implement that instruction yourself:
```basm
[@SET Acell Vval] [
    ZERO Acell;
    INCR Acell Vval;
]

[main] [
    // instead of:
    ZERO 0;
    INCR 0 12;

    // you can simply write:
    SET 0 12;
]
```

At compile time the `SET` instruction will be expanded and the aliases will be resolved,
leading the main function looking something like this:
```basm
// instead of:
ZERO 0;
INCR 0 12;

// you can simply write:
ZERO 0;
INCR 0 12;
```

As you can see, using meta-instruction only saves you time!

### `COPC` (Copy Conservatively)
It would have been useful in the fibonacci program that we just did if we could have copied the value of `Ab`
to `Ac` without having to explicitly use a temporary `Atmp`, right? We can made an instruction for that!
This meta-instruction is called `COPC` for "copy conservatively".
In this context, being conservative means that this instruction does not consume its inputs,
or rather as we will see while implementing it, it is not consumed when it ends.

```basm
[@COPC Asrc Adst sp] [
    ALIS Atmp sp;
    ALIS sp sp+1;

    COPY Asrc Adst Atmp;
    ADDP Asrc Atmp;
]
```

I am guessing you aren't too chocked for the most part by this implementation, we simply encapsulate
the temporary shuffle in a meta-instruction, like we would encapsulate logic in any other language.
Well, you probably ask are yourself what `sp` means in this context.

We cannot simply use a static memory position for `Atmp` as we don't know which cells are used when it is called.
Instruction which need memory for their operation can include a `sp` argument as the last one
to ask for where it is safe to "allocate" cells for computation.
`sp` meaning "stack pointer" is an address pointing to the first free cell that the instruction can use for
it's operation. This is why `Atmp` is defined from `sp`.

I personally like to always have an `sp` alias in all the scopes which I can update as I allocate more cells.
This is especially true in the `[main]` scope where you want to keep track of where you have not allocated yet.
Allocating based on the stack pointer also allows you to seamlessly add a cell without having to offset all later
cell pointers by one (like we had to do in the fibonacci example).

Here is what fibonacci would look like with `COPC` and the `sp` system:
```basm
[@COPC Asrc Adst sp] [
    ALIS Atmp sp;
    ALIS sp sp+1;

    COPY Asrc Adst Atmp;
    ADDP Asrc Atmp;
]

[main] [
ALIS Aa 0;
ALIS Ab 1;
INCR Ab 1;
ALIS Adesired_index 2;
IN Adesired_index;
ALIS sp 3; // set the sp to the next free cell

WHNE Adesired_index 0 [
    // "allocate" cells where the is free space
    // this allows us to easily add another cell before without having to touch this
    ALIS Ac sp;
    ALIS Atmp sp+1;
    ALIS sp sp+2; // update sp

    // we use sp here
    COPC Ab Ac sp;
    ADDP Ac Aa;

    ADDP Aa Ab;
    ADDP Ab Ac;

    DECR Adesired_index 1;
];
// aliases being bound by scope helps us here by giving us back
// our old sp from before the WHNE scope, basically freeing a part of the stack

OUT Aa;
]
```

### TWIC (Twice)
There aren't many interesting examples of using scopes as arugments
other than trying to make conditional execution, so I wanted to give an little simple example here.
`TWIC` is a instruction that takes a scope and duplicates it, causing it to execute twice.

```basm
[@TWIC [scope]] [
    INLN [scope];
    INLN [scope];
]
```

I mean, I have a whole chapter written about using scope as arguments and it's the next one... so let's read it!

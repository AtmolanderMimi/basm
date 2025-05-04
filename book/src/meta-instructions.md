# Meta-Instructions

Wow, only now are we onto meta-instructions!
Meta-instruction are like functions in other programming languages, except that they aren't.
They are actually just multiple instructions joined into one, thus the name meta-instruction.
These composite instructions are expanded at compile time into their component built-in instructions.
So, they are actually more similar to macros or always inlining functions.

## The Meta Fields

To define a meta-instruction you need to use the second and only other field other than `[main]`,
which is the `[@META arg1 arg2]` field.
A basm source file can have zero or more meta-instructions fields present in a file before the `[main]` one.
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

Basm will automatically create aliases named after the arguments which are binded to the arguments passed in by the caller.
This means that you can refer to `arg1` like any other numeric alias in the scope of `[@META arg1]`.

## Naming

Meta-instructions cannot share the name of other instructions, must they be built-in's or user defined meta-instrutions.
Also, quick aside, while I prefer to keep the 4 capital leters naming convention for instruction names,
names abide by the same restrictions as aliases and are thus not limited to 4 capital letters or less.

## Examples

Meta-instructions are very practical to remove boilerplate.
Since I decided to greatly reduce the amount of built-in instructions,
you will probably need some to define some quality of life instructions.
Thankfully, doing so is both easy on the meta-instruction definition end and in the useage end.

### `SET`

If I want to set the value of cell by incrementing, but I know that it may be not consumed,
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

At compile time, the `SET` instruction will be expanded and the aliases will be resolved,
leading the main field looking something like this after meta-instruction inlining:

```basm
// instead of:
ZERO 0;
INCR 0 12;

// you can simply write:
ZERO 0;
INCR 0 12;
```

As you can see, using meta-instruction saves you time and brain!

### `COPC` (Copy Conservatively)

It would have been useful in the fibonacci program that we just wrote if we could have copied the value of `Ab`
to `Ac` without having to explicitly use a temporary `Atmp`, right? We can made an instruction for that!
This meta-instruction is called `COPC` for "copy conservatively".
In this context, being conservative means that this instruction does not consume its inputs,
or rather as we will see while implementing it, they are not consumed when the instruction ends.

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
Everything is normal to one exception, you are probably asking are yourself what `sp` is doing here and what it means.

We cannot simply use a static memory position for `Atmp` as we don't know which cells are free to be used when when `COPC` is called.
Just using a static address for `Atmp` would probably just end up creating a situation where `COPC` and another operation are both using that static cell for something.

Instruction which need memory for their operation can include a `sp`argument as the last one to ask for where it is safe to "allocate" cells for computation.`sp`meaning "stack pointer" it is an address pointing to the first free cell that the instruction can use for its operations.
In the context of basm, allocating a cell is synonymous to "reserving for a cell for an operation".
This is why`Atmp`is defined from`sp`. We know that cell including and after `sp` are free to use.

I personally like to always have an alias named `sp`, which increment as I allocate more cells, in all scopes.
When I want to allocate a new cells, I create an alias for the address of all cells and then increase `sp` by each cell I've alocated.
In the example above, this logic comes in. I create 1 alias for a temporary cell named `Atmp` and then increase `sp` by 1.

The `sp` alias also benefits greatly from alias lifetimes.
When you increase the value of `sp` what you really do is create a new copy of `sp` that shadows the last one.
This means that when you are done with a specific operation and the scope ends, `sp` will decrease naturally.
This practially *frees* the cells that were allocated in the scope!

Using `sp` is especially useful in the `[main]` scope where you want to keep track of where you have not allocated yet.
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
other than trying to make conditional execution, so I wanted to give an little simple example here before actually getting into conditinals,
because those are a bit complicated.
`TWIC` is a instruction that takes a scope and inlines it twice, causing it to execute twice.

```basm
[@TWIC [scope]] [
INLN [scope];
INLN [scope];
]
```

I mean, I have a whole chapter written about using scope as arguments for conditinals, and it's the next one... so let's read it!

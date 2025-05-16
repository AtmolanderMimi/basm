# Dynamically Indexing

We are now here, all the content chapters are after us and we are on the last stretch to making
the final project, being a bf interpreter written in basm.
But, before being able to go into that directly, I'd like to isolate a part of that bigger program
and built it from scratch prior.
To make a bf interpreter we will need a dynamically indexable array.

## Parking

Yes, I do say dynamically indexable **array** and not meta-instruction for dynamically indexing.
While it would be possible to index any array dynamically without changing its layout,
for the sake of conciseness, we will need to modify the layout of a basic array somewhat to allow
for our dynamic get/set operations.
To hold meta-instruction specific logic we'll reserve a little amount of memory before the actual array content.
That memory region in the array is called the "parking".

Since we are going to use a flyer to move through the array,
we need somewhere to create it before it starts moving.
The reserved space is going to be used for flyer construction.
Our implementation of a dynamically indexable array will need **4 cells zeroed cells** for parking before the main contents of the array.
Getting and setting will use these spaces in different ways.

## `GETD` (Get Dynamic)

**Arguments**


| Name   | Type   | Description                                                   |
| -------- | -------- | --------------------------------------------------------------- |
| Aarray | number | the address of the start of the array (including the parking) |
| Aindex | number | the address of the cell to be gotten in the array             |
| Adst   | number | the address that will be used to store the gotten cell        |

As mentioned prior, we will be using a flyer with state to move through the array.
The data it will need to hold are the index (aka the number of cells it needs to move),
the return index (aka the number of cells it needs to move to get back to the parking)
and, finally, when on the return, it will of course need to contain the gotten cell.
We will also need to reserve an empty space (which we will call `[swap]`) to swap array content from the front to the back of our flyer.

### Flyer Layout

A diagram of our flyer would look like so:

```text
[swap].[return][index].[element] ->
```

And then on the return, once index is depleted:

```text
<- [element].[return][cell].[swap]
```

**`[index]`** holds the distance left until the flyer reaches the index.
It counts down for everytime the glider moves forward.

**`[return]`** holds `og_index - [index]`.
Basically everytime `[index]` is decreased, we increase `[return]`.
Since we slowly consume `[index]` to go forward, we need another counter to go backward.
That counter comes in the form of the `[return]` cell to be able to return.
In the best of worlds we would just move back until we reach a certain flag, rather than having to store that `[return]` index,
but unlike strings, arrays can hold any arbitrary data.
This means that we can't just move back until we meet a cell with a specific value,
as that specific value could be contained in the array itself!

**`[cell]`** holds the value of the cell at the index.
Once `[index]` is zeroed by the decrements, `[cell]` takes its place.

**`[swap]`** is the destination cell for `[element]`.
Since array needs to be moved out of the way in order for the glider to move forward,
an empty cell needs to be reserved for the element to be moved into.
Note that `[swap]` needs to be there in the parking so that the cell no.0 can be moved out,
although `[[swap]` is not part of the glider.
The `[swap]`cell on the flyer should always be zero.
By default,`[swap]`is included in our parking space and, as we move forward,
it gets zeroed by the glider moving operation, so`[swap]` should always be zero
If it is not, it will corrupt the array when flying through it
as adding into a non-zero cell is no longer a move, like we expect, but an actual *addition*.

**`[element]`** represents the element of the array right in front of the flyer,
it is not part of the flyer.
It is the first cell of content of the array in the initial glider state.
*(Not much to add here...)*

*(Right now, not all parking cells are used, don't worry we will need all the 4 of them in `ADDP`)*

### Making our Flyer Move

Moving our gliders (yes, I will also call flyers gliders) around is going to be a 4 step operation:

1. End if `[index] == 0`, decrease the `[index]` by 1,
2. Move the cell in front of us (`[element]` to our back `[swap]`)
3. Move the whole of the glider cells up by one
4. Offset the assumed pointer position by one and repeat

This procedure will shift all the cells before the cell at *index* to our back,
this is fixed on the way back, where we do the same shifting to go backwards,
effectively cancelling the shift.

### A Simpler `INCD` (Increment Dynamic)

Before getting too much into the weeds of implementing this whole thing,
let's just stand back a little and write the simplest part of it:
a glider that can move to an dynamic index and increase the cell by 1.

That glider will be much simpler! We won't need `[return]` or `[cell]`, making it a flyer with 1 cell of state.

For context, diagram of our flyer would look like so:

```text
[swap].[index].[element] ->
```

We won't be able to move backwards without `[return]`.
This also means that the assumed pointer position will be busted once the glider has done its thing.
`INCD` as written here should be the last thing you do execute your program.

```basm
[@INCD Aarray Aindex] [
// we still assume the array has 4 cells of parking
// we load the parking like so: [empty][empty][swap][index]
ADDP Aarray+3 Aindex; // Aindex -> [index] 

// entering relative mode
// (since we won't have free unallocated space, we won't be able to use instructions with sp)
BBOX Aarray+2;
ASUM 0;

ALIS Aswap 0;
ALIS Aindex 1;
ALIS Aelement 2;

WHNE Aindex 0 [
    DECR Aindex 1; // step 1
    ADDP Aswap Aelement; // step 2

    ADDP Aindex+1 Aindex; // step 3

    // step 4
    // (it's very important do this before the end of the scope,
    // because we need Aindex to point to the actual index cell so that the WHNE can check it)
    BBOX 1;
    ASUM 0;
];

// we are here, this means that we moved all we need
// the cell we want is right in front of us at Aelement
INCR Aelement 1;

// normally this is here we would come back to unshift all the array
// and fix the assumed tape pointer position, but we can't since we didn't store a return index!
]
```

Let's try it by giving it an array with some already initialized values!

```basm
// add the INCD definition here...

[main] [
ALIS Aindex 0;
INCR Aindex 3;

ALIS Aarray 1;
ALIS Aarray_contents Aarray+4;
INCR Aarray_contents 1;
INCR Aarray_contents+1 2;
INCR Aarray_contents+2 3;
INCR Aarray_contents+3 4; // the one that will be increased
INCR Aarray_contents+4 5;
INCR Aarray_contents+5 6;

INCD Aarray Aindex;
]
```

If we use the `-d` flag to dump the memory after execution we can see the program state:

```txt
-- TAPE STATE NUMERIC --
0: 0
1: 0
2: 0
3: 1
4: 2
5: 3
6: 0
7: 0
8: 5
9: 5
10: 6
```

Good news, the cell containing 4 was successfully increased to 5!
Bad news, we can see the effects of the array shift: every element before the one we operated on got
offset by 2.
Notably, you can also see the parking (cells 1 to 4) has been partially overwritten by the shifted elements
of the array and the cells 6 and 7, being those of our flyer, being completely zeroed as expected.

Don't expect to write these kind of meta-instruction right on the first try,
there is always going to be little typos and unforeseen behaviour!
Myself, while writing this example, had made many typos and oversights on the first try.

### Back to `GETD`

Alright, enough dilly dallying now! Let's get to actually implementing `GETD` with what we have learnt.

The first notable difference of `GETD` from what we just wrote is the memory layout,
we already saw what we needed extensively in the [layout section](#flyer-layout).
Just keep in mind that our parking layout should look like this:

```text
[empty][swap][return][index]
```

Just adding the `[return]` cell (and later the `[cell]` cell) to our logic isn't too difficult:

```basm
[@GETD Aarray Aindex Adst] [
ADDP Aarray+3 Aindex; // Aindex -> [index] 

// switch +2 -> +1 because our flyer is bigger
BBOX Aarray+1;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1; // add return
ALIS Aindex 2;
ALIS Aelement 3;

WHNE Aindex 0 [
    DECR Aindex 1; // step 1
    INCR Areturn 1;

    ADDP Aswap Aelement; // step 2

    ADDP Aindex+1 Aindex; // step 3
    // it's important to correctly order the moves,
    // so that your glider doesn't overlap itself
    ADDP Areturn+1 Areturn;

    // step 4
    BBOX 1;
    ASUM 0;
];

// we arrived!
// we don't have a [index] anymore, but we have a [cell] at its place
ALIS Acell Aindex;

ADDP Acell Aelement; // taking the element

// not done yet...
```

We are not done here!
The second notable difference from the `INCD` is going to be that we want to go back
both to un-shift the cells and reset the assumed pointer position to the correct value.
If we don't do both of these things, `GETD` becomes practically the last valid instruction in the program.
To do that we will have to copy the code of going forward and modify it to go back.
It is as simple as making our loop read from `[return]`instead of`[index]` and reordering the flyer state moves.

```basm
// ... rest of GETD implementation

// since we move back, we'll swap [element] and [swap]
ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1; // step 1

    // we now have <- [element][return][cell][swap]
    // we move back so -1 instead of +1
    ADDP Areturn-1 Areturn; // step 3
    ADDP Acell-1 Acell;

    // step 4
    // we inverted BBOX and ASUM values so we can go back
    BBOX 0;
    ASUM 1;

    // step 2 (because of off by 1 reasons, we need it after the move)
    ADDP Aswap Aelement;
];

// we are back at parking here and since we know where parking is,
// we can set the assumed pointer back to a valid value!
BBOX Aelement;
ASUM Aarray+1;

ADDP Adst Aarray+3;
]
```

We can now try the same `[main]` as with `INCD` before, but with `INCD Aarray Aindex;`
changed to `GETD Aarray Aindex 0;`:

```text
-- TAPE STATE NUMERIC --
0: 4
1: 0
2: 0
3: 0
4: 0
5: 1
6: 2
7: 3
8: 0
9: 5
10: 6
```

That's exactly what we wanted, the 3rd element of the array has been gotten and moved to cell 0.
Notice how none of the array is shifted anymore because of the backtracking we did, perfect!

## `ADDD` (Add Dynamic)

**Arguments**


| Name   | Type   | Description                                                   |
| -------- | -------- | --------------------------------------------------------------- |
| Aarray | number | the address of the start of the array (including the parking) |
| Aindex | number | the address of the cell to be gotten in the array             |
| Asrc   | number | the address of the cell that will be added to the dynamic one |

Now that we are done with `GETD`, we can move onto `ADDD`.
This meta-instruction is going to serve as our setter, we won't call it `SETD` though,
because it will not zero the cell it adds to, making it more of an add than a set.
This means that pretty much all the rules that we saw for moving with `ADDP`'s will apply to this, except now in a dynamic fashion.

### Flyer Layout

The interesting (and annoying) part of the `ADDD` glider, is that rather than carrying a cell value
on the return trip, we do it on the going trip.
This means we can't take advantage of the index being zeroed in the return trip to store the cell value where index stood.
So, sadly, this requires us to make the flyer have **3 reserved data cells** at the start
(plus the swap, maxing out our 4 spots of parking).

A diagram of our flyer would look like so:

```text
[swap].[return][index][cell].[element] ->
```

And then on the return, once index is depleted:

```text
<- [element].[return][empty][empty].[swap]
```

It's going to be important to keep the `[empty]` cells empty, so that we properly un-shift all the cells on the return.

### Implementing

`ADDD` is very similar to `GETD` outside of the flyer layout.
The only notable difference between the two being that the flyer has 3 data cells, and that rather
than taking a cell, it gives one of its cells.
Other than that, you can probably take `GETD` and make an `ADDD` out of it with ease
like we made `GETD` from our knowledge of `INCD`
(despite being similar in name,
`ADDD` is very different from our arguably incomplete `INCD` implementation).

With that said, this means that a `ADDD` implementation should not be foreign to you.
So, here is its definition:

```basm
[@ADDD Aarray Aindex Asrc] [
// this layout forward: [swap].[return][index][cell].[element]
ADDP Aarray+3 Asrc;   // Asrc   -> [cell]
ADDP Aarray+2 Aindex; // Aindex -> [index] 

// we'll remove the +1, because we'll use the full parking
BBOX Aarray;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1;
ALIS Aindex 2;
ALIS Acell 3; // new!
ALIS Aelement 4; // .. which offset this by one

WHNE Aindex 0 [
    DECR Aindex 1; // step 1
    INCR Areturn 1;

    ADDP Aswap Aelement; // step 2

    ADDP Acell+1 Acell; // step 3 (also, new Acell move!)
    ADDP Aindex+1 Aindex;
    ADDP Areturn+1 Areturn;

    // step 4
    BBOX 1;
    ASUM 0;
];

// we arrived!
// adding the element out of the flyer
ADDP Aelement Acell;

ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1; // step 1

    // we now have <- [element][return][empty][empty][swap]
    // (we won't need to move both empty cells)
    ADDP Areturn-1 Areturn; // step 3

    // step 4
    BBOX 0;
    ASUM 1;

    ADDP Aswap Aelement;
];

BBOX Aelement;
ASUM Aarray; // removed the +1, again
]
```

Running the following program: ..

```basm
// .. ADDD definition here

[main] [
ALIS Aindex 0;
INCR Aindex 3;
// we need to allocate one more cell
ALIS Acell 1;
INCR Acell 40;

ALIS Aarray 2;
ALIS Aarray_contents Aarray+4;
INCR Aarray_contents 1;
INCR Aarray_contents+1 2;
INCR Aarray_contents+2 3;
INCR Aarray_contents+3 4;
INCR Aarray_contents+4 5;
INCR Aarray_contents+5 6;

ADDD Aarray Aindex Acell;
]
```

.. would expectedly give us this result:

```txt
-- TAPE STATE NUMERIC --
0: 0
1: 0
2: 0
3: 0
4: 0
5: 0
6: 1
7: 2
8: 3
9: 44
10: 5
11: 6
```

With that out of the way, we are ready to tackle the last chapter of this book!
In the next chapter, we will use what we just wrote to handle the
memory tape array and instruction array reading/setting of our bf interpreter.
So, don't throw away these meta-instructions just yet!

## Note

These dynamic get/set implementation can probably be improved by you!
Due to the nature of flyers (specifically moving the front element to the back swap),
using long arrays/a lot of dynamic array addresing is not good for performance.
If you can implement something while forgoing using these `GETD`/`ADDP`, prefer the unglided way.

Furthermore, there is a way to make dynamic indexing for regular arrays without parking.
This would require to move some cells out of the way to create a temporary parking and then set them back after the instruction is done.
The issue with this, and why this is not the version used here, is because it is a bit more complex.
You need to check wether the index refers to one of the cells that you are going to move away for example.

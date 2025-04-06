# Dynamically Indexing

We are now here, all the content chapters are after us and we are on the last strech to making
the final project, being a bf intepreter written in basm.
But, before being able to into that directly, I'd like to isolate a part of that bigger program
and built it from scratch here prior.
To make a bf interpreter we will need a dyamically indexable array.


## Parking
Yes, I do say dynamically indexable **array** and not meta-instruction for dynamically indexing.
While it would be possible to index any array dynamically without changing it's layout,
for the sake of conciseness, we will need to modify the layout of a basic array somewhat to allow
for our dynamic get/set operations.

Since we are going to use a flyer to move through the array,
we need somewhere to create it before it starts moving.
This reserved space for flyer construction is called the "parking".
Our implementation will need **4 cells zeroed cells** for parking before the main content of the array.
Getting and setting will use these spaces in different ways.

## `GETD` (Get Dynamic)
**Arguments**
| Name | Type | Description |
| - | - | - |
| Aarray | numeric | the address of the start of the array (including the parking) |
| Aindex | numeric | the address of the cell to be gotten in the array |
| Adst | numeric | the address that will be used to store the gotten cell |

As mentioned prior, we will be using a flyer with state to move through the array.
The data it will need to hold are the index (aka the number of cells it needs to move),
the return index (aka the number of cells it needs to move to get back to the parking)
and, finally, when on the return, it will of course need to contain the gotten cell.
We will also need to reserve an empty space (which we will call `[swap]`) to move forward.

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
Since we slowly consume `[index]` to go forward, we will need another counter to go backward.
We need to rely on a `[return]` cell to be able to return,
because, unlike strings, arrays can hold any arbitrary data.
This means that we can't just move back until we meet a cell with a specific value,
as that specific value could be contained in the array itself!

**`[cell]`** holds the value of the cell at the index.
Once `[index]` is zeroed by the decrements, `[cell]` takes its place.

**`[swap]`** is the destination cell for `[element]`.
Since array needs to be moved out of the way in order for the glider to move forward,
an empty cell needs to be reserve for it.
Note that `[swap]` needs to be there in the parking so that the cell no.0 can be moved out,
although it is not part of the glider.
The `[swap]` cell on the flyer should always be zero.
By default, `[swap]` is included in our parking space and,
as we move forward, it gets zeroed by the glider moving operation,
so `[swap]` should always be zero
If it is not, it will corrupt the array once we flew through it.

**`[element]`** represents the element of the array right in front of the flyer,
it is not part of the flyer.
*(Not much to add here...)*

*(Don't worry we will need all the 4 spaces in `ADDP`)*

### Making our Flyer Move
To move our glider (yes, I will also call flyers gliders) around we'll:
1. End if `[index] == 0`, decrease the `[index]` by 1, 
2. Move the cell in front of us (`[element]` to our back `[swap]`)
3. Move the whole glider up by one
4. Offset the assumed pointer position by one and repeat

This procedure will shift all the cells before the index to our back,
this is fixed on the way back, where we do the same shifting to go backwards,
effectively cancelling the shift.

### A Simpler `INCD` (Increment Dynamic)
Before getting too much into the weeds of implementing this whole thing,
let's just stand back a little and write the simplest part of it:
a glider that can move to an dynamic index and increase the cell by 1.

That glider will be much simpler, we won't need `[return]` or `[cell]`, making it a 1 cell flyer.

For context the layout would look like this:
A diagram of our flyer would look like so:
```text
[swap].[index].[element] ->
```

We won't be able to move backwards without `[return]`.
This also means that the assumed pointer position will be busted once the glider has done its thing.
```basm
[@INCD Aarray Aindex] [
// we still assume the array has 4 cells of parking
// we load the parking like so: [empty][empty][swap][index]
ADDP Aarray+3 Aindex; // Aindex -> [index] 

// entering relative mode
// (since we won't have free unallocated space, we can't instructions with sp)
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
    // because we need Aindex to be valid so that the WHNE can check it)
    BBOX 1;
    ASUM 0;
];

// we are here, this means that we moved all we need
// the cell we wanted is at Aelement
INCR Aelement 1;

// normally this is here we would come back to unshift all the array
// and fix the assumed tape pointer position, but we can't since we didn't store a return index!
]
```

Let's try it by giving it an array with some already initialized values!
```basm
// add the INCD declaration here...

[main] [
ALIS Aindex 0;
INCR Aindex 3;

ALIS Aarray 1;
ALIS Aarray_contents Aarray+4;
INCR Aarray_contents 1;
INCR Aarray_contents+1 2;
INCR Aarray_contents+2 3;
INCR Aarray_contents+3 4;
INCR Aarray_contents+4 5;
// the one that is going to be increased
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
We can see the effects of the array shift, were every element before the one we operated on got
offset by 2.
Notably, you can see the parking (cells 1 to 4) has been partially overwriten by the shifted elements
of the array and the cells 6 and 7, being those of our flyer, being completely zeroed as expected.

Don't expect to write these kind of meta-instruction right on the first try,
there is always going little typos and unforseen behaviour!
Myself, while writing this example, had made many a typos and oversights on the first try.

### Back to `GETD`
Alright, enough dilly dallying now! Let's get to actally implement `GETD` with what we have learnt.

The first notable difference of `GETD` from what we just wrote is the memory layout,
we already saw what we needed extencively in the [layout section](#flyer-layout).
Just keep in mind that our parking layout should look like this:
```text
[empty][swap][return][index]
```

Just adding the `[return]` cell (and later `[cell]`) to our logic isn't too difficult:
```basm
[@GETD Aarray Aindex Adst] [
ADDP Aarray+3 Aindex; // Aindex -> [index] 

// switch to +2 -> +1 because our flyer is bigger
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
To do that we will have to copy the code of going forward and modify it to go back.
We'll need to read from `[return]` instead of `[index]` now.
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
    // we inversed BBOX and ASUM values so we can go back
    BBOX 0;
    ASUM 1;

    // step 2 (because of off by 1 reasons, we need it after the move)
    ADDP Aswap Aelement;
];

// we are back at parking here and we know where parking is,
// so we can set the assumed pointer back to a valid value!
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


## Note
These dynamic get/set implementation can probably be improved by you!
Due to the nature of flyers (specifically moving the front element to the back swap),
using long arrays/a lot of dynamic array adressing is not good for performance.
If you can implement something while forgoing using these `GETD`/`ADDP`, prefer the unglided way.

# Creating Custom Conditionals

When we have one type of conditional execution in a language, we can derive all the other types from it.
Same for looping, if we have a looping element we can make any looping element.
In the case of basm, that any conditional/looping element is `WHNE` which we can derive all conditionals and loops we desire from.
In this chapter, we're going to first implement `IFNE` (If Not Equal), then from it `IFEQ` (If Equal).

First off, to make these meta-instruction conditionals we need to think of their arguments.
We are going to want a cell address and a value so we can compare them together,
a scope to be conditionally executed, and lastly,
we are going to make use of the `sp` pattern to use some operation memory.

So, the argument table for all of our conditionals should look like this:


| Name  | Type    | Description                      |
| ------- | --------- | ---------------------------------- |
| addr  | number | address of cell compared to`val` |
| val   | number | value compared to cell at`addr`  |
| [scp] | scope   | code to be conditionally executed |
| sp    | number | address to the next free cell    |

## IFNE (If Not Equal)

`IFNE` is the easiest conditional to define if we consider that right now we only have `WHNE`.
When you think about it, an if statement is simply a conditional loop that loops once.
With that in mind, we'll want to purposefully make `WHNE` loop once while also keeping its comparason ability.

Here is how I would go about implementing a one iteration `WHNE` loop:

```basm
[@IFNE Aaddr Vval [scp] sp] [
    WHNE Aaddr Vval [
        INLN [scp];

        // force Aaddr to be equal to Vval
        ZERO Aaddr;
        INCR Aaddr Vval;
    ];
]
```

You will notice that this implementation works!
But, there is some pretty big downsides that we will want to avoid.

We don't want these implementations to consume their inputs, so `IFNE`'s can be chained.
In this case this implementation sets `Aaddr`to the value of`Vval`, which is no good.
If we want to chain `IFNE`'s we will need to manually copy `Aaddr`each time we pass it in, which is tedious.

Apart from that, there should be no non-zero allocated cells when inlining the scope as it might corrupt the behaviour of the scope,
which does not expect`IFNE` to allocate cells while it is running.
The specifics of what I just said are important, we don't want non-zero allocated cells.
What this means, is that we can allocate cells, but when we inline the scope argument, all of our cells should be zero.
If all our cells are zero, it is as if, to the inlined scope, that there are no allocated cells.
Allocation is only a concept, what matters is whether the cells are zero or not when the scope is inlined.
This notion is important as it will allow the caller to use the same `sp` both in the scope and in the `IFNE` argument.
If `IFNE` required a cell to be allocated when inlining the scope, then all the mentions of `sp` in the scope should be increased by 1.

So, let's solve these issues via extra cells that are granted by `sp`:

```basm
[@IFNE Aaddr Vval [scp] sp] [
ALIS Atmp1 sp;
ALIS Atmp2 sp+1;
ALIS sp sp+2;

// copy to Atmp1 so we can consume it
COPY Aaddr Atmp1 Atmp2;
ADDP Aaddr Atmp2;

WHNE Atmp1 Vval [
    // we don't care about Atmp1, it needs to be consumed before the scope
    ZERO Atmp1;

    // at this point, all of the values allocated in IFNE are zero,
    // so it's like we allocated nothing, scp can use the same cells as we just did without causing bugs
    INLN [scp];

    INCR Atmp1 Vval;
];

// cleanup
ZERO Atmp1;
]
```

Now let's try it and see if it works:

```basm
// .. add the IFNE definition

[main] [
    ALIS Aval 0;
    INCR Aval 42;
    ALIS sp 1;
  
    // this will print ..
    // (IFNE and PSTR can use the same sp)
    IFNE Aval 33 [
        PSTR sp "Aval is not equal to 33!";
    ] sp;

    // .. but not this
    IFNE Aval 42 [
        PSTR sp "Aval is not equal to 42!";
    ] sp;
]
```

Notice how both `PSTR` uses the same cells as `IFNE`? Yet, they don't collide as we purposefully
zeroed all cells from the `IFNE` scope before running the scope argument!
Try it yourself: Move the `ZERO Atmp1;` instruction after the inlining
and see what happens when values are still non-zero.

## IFEQ (If Equal)

`IFEQ` is very much linked to `IFNE`. `IFEQ` executes only when `IFNE` doesn't.
We can make use of relation to easily derive `IFEQ` from a couple of `IFNE`'s.
Rather than copying the value around,
this implementation will make use of a flag representing wheter or not `IFNE` was executed.

```basm
[@IFEQ Aaddr Vval [scp] sp] [
    ALIS Aflag sp;
    ALIS sp sp+1;
    ALIS Vnot_equal 1;

    IFNE Aaddr Vval [
        INCR Aflag Vnot_equal;
    ] sp;

    // this reads like "if is not equal to not equal"
    // both "not equals" cancel out and we get "if is equal"

    // once again all cells are zero here,
    // as the flag needs to be of 0 for this to execute
    IFNE Aflag Vnot_equal [scp] sp;

    // cleanup (only useful if it did not execute)
    ZERO Aflag;
]
```

Once again I encourage you to test it:

```basm
// .. IFEQ and IFNE definition here

[main] [
    ALIS Aval 0;
    INCR Aval 42;
    ALIS sp 1;
  
    // this will print ..
    IFEQ Aval 42 [
        PSTR sp "Aval is equal to 42!";
    ] sp;

    // .. but not this
    IFEQ Aval 60 [
        PSTR sp "Aval is equal to 60!";
    ] sp;
]
```

With that out of the way, it's going to be much faster to write actually useful programs
and now that you know the trade, I am sure you will be able to create your own conditionals!
*(If you want to challenge yourself, try making a `WHEQ`)*

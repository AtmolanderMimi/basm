# Calculating Fibonacci

Now that you've learnt the basic building blocks of the language,
you are probably eager to do something useful with them,
but like a kid with unasorted lego blocks it may not be as easy as it seems to build something coherent with them.
This synthesis chapter is a walk-through of creating a program
which calculates the nth number of the fibonacci sequence.
We will first start by creating the basic logic then add user input.

If you want to go ahead and try to build a fib program yourself before reading this explaination,
I higly recommend it!
Getting your hands dirty is the best way to learn and memorise querks of the language
*(and any skill for that matter)*.

## Building the Inputless Version
We are going to be implementing the iterative version of the fibonacci calculator.
*(In opposition to the recusive one, because there is no recusion in basm)*

If you have never built an iterative fibonacci calculator in another language before, here is pseudo-code for it:
```txt
a = 0
b = 1

for i in range 0..nth:
    c = a + b
    a = b
    b = c

output a
```

What's nice here is that most of the features present in that pseudo-code example are present in basm.
We can store values like `a`, `b` and `c` in cells and we of course have addition and output.
There one pain point though, which is that unlike in the pseudo-code basm instructions consume their inputs.
This means that we can't feasably use `b` to add and to move without having to copy it.
That will unfortunatly require adding a bit more copying logic boilerplate.

Here's the non-functional pseudo-code/basm hybrid with the pieces which we can easily replace:
```basm
// unlike in the example above, we need to set the value of the variables
ALIS Aa 0;              // a = 0 (all values are 0 by default)
ALIS Ab 1;
INCR Ab 1;              // b = 1

// this for loops is invalid in basm
for i in range 0..nth:
    ALIS Ac 2;
    ALIS Atmp 3;

    COPY Ab Ac Atmp;    // c = b and tmp = b
    ADDP Ac Aa;         // c += a

    ADDP Aa Atmp;       // a = tmp (which is b)
    ADDP Ab Ac;         // b = c

OUT Aa;                 // output a
```

Notice how `ADDP` had the double purpose of both adding and moving values around?
We could be sure that these moves are safe to do because the adresses where consumed by prior instructions.
An example of this is with `Ab`, we knew it was zeroed from the `COPY` instruction,
so it was safe to use `ADDP` as a move to move `Ac` to `Ab`.

It is very important to keep track of which cell has been consumed when writing basm code.
That, not only for being able to move cells to other cells without having to preemptively `ZERO` them,
but also for what I like to call "scope hygine". What it means is that a scope, once executed,
will not leave non-zeroed cells. It is the duty of the scope to zero the cells it uses for operation.
In the case of this example, the cells used for operation in the `for` loop scope are `Ac` and `Atmp`.
Looking at the code we can see that they both get consumed before the end of the scope. Perfect!
If you betray scope hygine, you might end up trying to use a cell with an arbitrary value,
whereas you expected all cells to be zero by default. By default, in basm,
we assume all cells untouched by the current scope to be zero, this allows us to save operations
(like having to `ZERO` all cells before using them).

Here is an example where not zeroing all cells before the end of the scope is problematic:
```basm
[
    // .. some operation ..
    ALIS Atmp 1;
    INCR Atmp 2;
    COPY Atmp 0 2;
]

ALIS Aval2 0; // we expect Aval2 == 0
INCR Aval2 '!';
OUT Aval2; // we should get '!', but get '#'
```
(In this case the problem is very easy to spot, it as the project size increases error caused these kinds of
bugs become harder and harder to spot)

For the last step in transforming our pseudo-code example into real basm code is to deal the `for` loop.
There is no inbuilt `for` loop instruction, so we will have to manually handle our loop and its exit condition.
Normally other languages would do this for you, but being basm being low-level we will need to explicitly
increase and check for the index.

```basm
ALIS Aa 0;
ALIS Ab 1;
INCR Ab 1;
// we add a cell to store the index
ALIS Ai 2;
// the index we want to reach (it's an constant for now)
ALIS Vdesired_index 11;

// this reads like: while (index != desired_index)
WHNE Ai Vdesired_index [
    // because of the extra cell used for the index we need to offset those by 1
    ALIS Ac 3; 
    ALIS Atmp 4;

    COPY Ab Ac Atmp;
    ADDP Ac Aa;

    ADDP Aa Atmp;
    ADDP Ab Ac;

    // increase the index for each iteration
    INCR Ai 1;
];

OUT Aa;
```

If you did everything well, now the program should output 89!
(or `Y`, if it does output this you can use the `-m` flag on the basm interpreter to output cells as numbers)
You can try to simply fiddle with `Vdesired_index` to get different numbers from the fibonacci sequence.
Be weary though, the bf interpreter that comes with the basm cli
defaults to using cells of unsigned 8 bit integers, meaning that numbers are limited from 0-255!
To change the size of the cells use `-c 16` flag to get 16 bit unsigned cells.


## Adding User Control
We will now be adding a very small amount of user input to our program:
we'll let the user decide which number they want from the fibonacci sequence.
This may seem simple, and it is! Rather than increasing a cell until we reach the desired value,
we can simply decrease the cell containing the desired value until we reach zero.
It's a bit different from the last version thought,
as in the desired value will be stored in a cell rather than be a constant.

```basm
ALIS Aa 0;
ALIS Ab 1;
INCR Ab 1;
// we set a cell to take user input (Value -> Address)
ALIS Adesired_index 2;
IN Adesired_index;

WHNE Adesired_index 0 [
    ALIS Ac 3;
    ALIS Atmp 4;

    COPY Ab Ac Atmp;
    ADDP Ac Aa;

    ADDP Aa Atmp;
    ADDP Ab Ac;

    // decrease the index for each iteration
    DECR Adesired_index 1;
];

OUT Aa;
```

Now, when running the program you will be prompted to input something, which will be treated as the index.
Make sure you use the `-n` flag while using the inbuilt basm interpreter
to treat input as numbers rather than as characters.
The prompt will repeat until you enter a valid number, meaning that it can parsed and contained by the cell.
Input is special in bf as it is the only thing that can straight up overwrite cell data
without caring about what it contained before. *(it is also special in that its implementation varies a lot from interpreter to interpreter, but whatever)*

Now you have a working interactive fibonacci calculator!
You check out the generated bf by printing it out via the `-p` flag
and make my interpreter sweat by asking it anything above `fib(30)`
(you will need the `-c 32` flag for the result to be correct).

# Aliases

Aliases are by far the most common abstraction feature you are going to use while writing basm code,
no questions asked.
They allow to alias number values or scopes to identifiers to representing the value.
You can think of aliases like variables in other programming langues, except they are purely immutable constants.
Once an alias is defined, it can be used to substitude having the write the literals forms of values anywhere the alias is valid.
Defining an alias is done via the `ALIS` "instruction" (`ALIS` can hardly be called an instruction as it does not compile into anything).

For example, if I have a cell located at `42` containing the index of an array, rather than having to memorise its location,
I could simply create an alias to the address like `Aindex`with the instruction`ALIS Aindex 42;`.

## The `ALIS` Instruction

At the exception of meta-instructions, the `ALIS` instruction is the only way to create aliases.
Since I baited you in the last chapter, below is its nicely formatted specification!

**Arguments**


| Name       | Type          | Description                         |
| ------------ | --------------- | ------------------------------------- |
| alias_name | identifier    | the name of the alias               |
| value      | numeric/scope | the value to be binded to the alias |

*(don't worry about the "identifier" type, it's not actually a type. Just that `ALIS` is a special boy and can take names as argument)*

`ALIS` allows creating aliases in the current scope by binding numeric or scope expressions to an identifier.
As an instruction, `ALIS` does not compile to anything, it is purely abstraction.

To define numeric aliases and then use the alias in place of any numeric value do as so:

```basm
ALIS Aindex 1;         // adress of the index cell
ALIS Vshrob 2;         // some random value
ALIS Vvalue 40+Vshrob; // value of the index

ALIS Vshrob 33;        // overwriting the last Vshrob alias

INCR Aindex Vvalue;
```

Aliases are purely abstraction for writing the **value** of the binded expressions where the alias is.
So, to help visualise, here is what the above program looks like after the aliases have been normalised out:

```basm
INCR 1 42;
```

Notice how `Vvalue` is replaced by the value of the expression passed into `ALIS` when the alias is created.
This is why `Vvalue` got replaced by 42 (40+2) rather than 73 (40+33).
The alias was bound to the value of the expression rat
her than the expression itself,
if it were bound to the expression changes to `Vshrob` would change `Vvalue`.

## Alias Validity

Aliases are valid for the duration of the scope they are defined in.
This means that an alias defined in a higher scope cannot reach a lower one.
That alias behviour is the main reason why you would want to insert a scope within another,
to have operation specific aliases. Here is an example of alias validity over the program:

```basm
ALIS Vsome 1; 

// Vsome: 1
[
    // Vsome: 1
    ALIS Vsome 2;
    // Vsome: 2
]
// Vsome: 1
```

Whilst we are talking about this, might as well mention that, as in the example above,
aliases can shadow eachother and that not only through scope boundaries!
They can also refer to the value they will shadow when defined.
Here's an example of a fairly common pattern using aliases:

```basm
ALIS Vnum_cats 5;
// ... some operations ...
ALIS Vnum_cats Vnum_cats+3;
```

That example firsts set `Vnum_cats` to 5
then does all the operations between the two `ALIS` with 5 as `Vnum_cats`.
Lastly, the `Vnum_cats` alias is shadowed by the second `ALIS` which sets the alias to the value of `Vnum_cats+5`.
The compiler normalises the expression and gets the result of 8 (5+3) which it sets to be the new `Vnum_cats`.
Using this, we can reach the what would be expected of variables in other programming languages, but don't get it
twisted, aliases are not variables. They are replaced by their value at compile time
and are totally gone at runtime.

So, don't expect something like this to increment forever:

```basm
// THIS DOES NOT WORK!

ALIS Vincrement 0;
WHNE 0 1 [
    ALIS Vincrement Vincrement+1;
];
```

That example would completely fail as the alias in the looped scope doesn't persist over loops.
It will get thrown out at the end of its scope like any other alias,
as thus the value of `Vincrement` will always be 1 (0+1) in the looped scope.

## Alias Types

There are currently two types which are supported by aliases: number and scope.
Yes, this means you can't alias a string, sadly.
So far, we have only made aliases of numeric values as these are the most common,
but we can also create scope aliases.
Aliases with different types won't overwrite eachother, even if they have the same name.
They are treated as completely different.
This means you can have both a numeric alias and scope alias named `my_alias` at the same time with no issue.
Basm will automatically use the alias with the fitting type when `my_alias` is used.

You create aliases of different types by matching identifiers to different expressions.
If you give a numeric value to `ALIS` you will create a numeric alias,
on the other hand if you give it a scope you will get a scope alias.

### Scope Aliases

Now, scope aliases are interesting to talk about as they allow you to share and reuse code around your source file,
unlike numeric aliases which are only about values.
Right now, they are not really interesting, but as you'll learn about meta-instruction
you will see one of most interesting usecase, as a *callback-like thingamabob*.

Since there hasn't been an example of alias scopes yet, here is one:

```basm
// defining "my_scope" scope alias
ALIS my_scope [
	INCR 0 1;
	OUT 0;
];

// using my_scope with the scope identifier syntax
WHNE 0 128 [my_scope];
```

You have probably noticed a difference to how we use numeric aliases,
being that we need to use a "scope identifier" to specify our scope alias.
A scope identifier is simply an identifier (aka the name of the alias) wrapped by square brackets.
This syntax tells the compiler to checks for the scope alias rather than the numeric alias
(also it allows the person reading it to know that this is a scope alias being passed as an argument,
which is *cool*).

If you don't use the scope identifier syntax you will get this error:

```txt
------------------ [ ERRORS ] ------------------
Error: from Ln 7, Col 12 in "/some/path/to/the/file.basm"
 → alias was not defined
[...] [main] [
ALIS my_scope [
        INCR 0 1;
        OUT 0;
];

WHNE 0 128 my_scope;
]
 [...]
```

*(the second `my_scope` is highlighted and underscored in the terminal)*

This means that the compiler tried to search for the numeric alias `my_scope`,
which obviously doesn't exist, and failed.

There is one more thing I want to add to scope aliases and that's inlining them.
Unlike scope literals you can't simply write the alias down to inline it!
So, you will need to use an instruction I made specifically for that called `INLN`.
It takes one scope argument and inlines it, as if you would have written a literal in the file at its place.

Here is an example displaying what I just talked about and an extra property that may not be obvious:

```basm
ALIS Vscale 7;
ALIS increment [
    INCR 0 Vscale;
];

// we repeat the scope three times
INLN [increment];
INLN [increment];
INLN [increment];

// then three more
ALIS Vscale 12;
INLN [increment];
INLN [increment];
INLN [increment];

OUT 0;
```

This example, when ran, outputs 42 (7*6) or the `*` character if your interpreter outputs as text.
That may seem odd, after all we set `Vscale` to 12 halfway through which should give us 57 (7\*3 + 12\*3), right?
Wrong! Whilst, you can include aliases within a scope alias, they will be immediatly normalized (aka replaced)
by their binded value. So, the `increment` alias actually looks more like this after being defined:

```basm
ALIS increment [
    // Vscale was replaced by its value
    INCR 0 7;
];
```

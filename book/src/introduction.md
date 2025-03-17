# Introduction

![](../../resources/logo.png)

This book serves as a short introduction and reference to the Brain Aneurysm programming language v1.0,
which I will refer to as `basm` throughout the book.
It will also touch on the compagnion cli tool of the same name.
The basm transpiler is fully open-sourced at [https://github.com/AtmolanderMimi/basm].

Basm is a very simple, assembly-like esoteric programming language made to transpile into Brainfuck (aka `bf` for short).
My target in creating basm was to prove (although it had already been done) that bf can be used to write any program
a user can think of. Whilst bf is turing-complete, it is rare to see that turing-complete*ness* actually exploited to
write general programs (and understandably so). Hopefully by providing tooling to simplify writing bf, it can become
feasable to write anything in it and be freed from the cage of being that language with a *funny name*.

By aiming to be transpiled into bf, basm shared many similarities with bf. Certain instructions are transpiled pretty much
1:1 into bf. We even allow writing raw bf in a basm source file. Basm does seperate itself from bf where it matters.
I've summed up my goal to solve three pain points of writing bf programs:
* The relative nature of memory
* Lack of organisation or code-reuse features
* Difficulties with text manipulation

## The Relative Nature of Memory
In bf you can't just say: `"add cell no.3 to cell no.2"`. You need to actually manage the tape pointer and most likely
do mental arithmetic to calculate how many `>` and `<` you'll need to get to the cell you want.
So don't don't mess up your bf tape pointer or you might end up with a silent bug killing
both you program and any hope of writing anything productive with bf.
You may even end up in a situation where you don't know where you are, because you conditinally moved the tape pointer
(in a loop like this for example: `[>]`).
You could say that bf dynamically adresses its memory by default. It says: `"get the cell -1 from me"`.

Basm tries to solve this by having a static adressing system.
This means that, in a basm source file, you can safely refer to the 3rd cell and it will always be the third cell
no matter the operations you did before.
The tape pointer is automatically managed by the compiler.
Because of the abstraction over raw bf operations, basm makes it impossible to reach a "relative state" without you wanting to. You cannot accidentally write `[>]`!
You must implicitly use a `RAW` or `ASUM` operation with relative effects to enter a relative state.
Reaching a relative state with these two operation is valid
as it is needed to write things like dynamic array get/setting.
*(Implementing these operations is really interesting and involves a lot of language features that I talk more about in the `Writing Relative Code` chapter)*

## Lack of Organisation or Code-reuse Features
Bf as virtually nothing to help developers, maybe that's why it got so popular, *maybe people like torture...*
Welp, I don't! So, I filled basm with plenty of ways to help you abstract over bf.

We have aliases with the `ALIS` operation, which allow you to
give name to values, adresses or blocks of code we call scopes.

We also have "meta-instructions" which are kinda similar to functions, but in implementation moreso similar to macros.
These meta-instructions, once defined can be used like any in-built instruction in the program.
Since bf doesn't have a jump operator and data is seperated from code, it's practically impossible to create functions.
So, meta-instructions, as the name implies, are simply instructions made up of other lower-level instructions,
hence why they are more similar to macros then functions as they are all inlined when used.

## Difficulties With Text Manipulation
Bf works with text like any other programming language would, it stores one character by cell in ASCII format.
The problem with that is that bf is not like any other programming language in pretty much all regards!
First and foremost, there is no way to set a cell to a specific value directly. If you want the caracter 'c',
then hold on tight, because you will need to type '+' 99 times from 0! Basm implements both a character literal
and a string literal, thus with basm you only need to type `INCR 0 'c';` to set cell no.0 to the value of 'c'.
There are also in-built operations for loading and printing strings, which is a tedious task in bf.

## Quick Note on Code Formating Throughout This Book
This book will contain and promote how I prefer to write code in basm.
There almost nothing in the language forces a specific formatting standard.
For example, to save colums, I like to omit putting a level of tab in the upmost scope.
Like so:
```basm
[main] [
// my code here
INCR 0 12;
WHNE 0 0 [
    // but i raise for scopes after
    DECR 0 1;
];
]
```

There is nothing in the language forcing you to omit whitespaces, in fact there is nothing in the 
language dictating how you should use whitespaces (except for instruction's arguments).
So, keep in mind that whilst I like to follow certain naming standards,
the striped down nature of basm allows you to write however your heart desires!

---

Now that you know wheter or not basm is for you, let's hop in!

## Disclamer
Both the language and transpiler are my first foray into language development and are
serving as my first real major project in Rust.
There have been better implementations of bf transpilable languages which are both more efficient are easier to use.
**This project is only for my own personal pleasure as a creator and is not aimed at creating efficient bf programs.
If you are seeking useable and well informed implementations of bf transpilers look [here](https://esolangs.org/wiki/Brainfuck_code_generation).**
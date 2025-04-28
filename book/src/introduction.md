# Introduction

![](../../resources/logo.png)

This book serves as a short introduction and reference to the Brain Aneurysm programming language v1.0,
which I will refer to as `basm` throughout the book.
This book will also touch on the compagnion cli tool of the same name.
The basm transpiler is fully open-sourced at
[https://github.com/AtmolanderMimi/basm](https://github.com/AtmolanderMimi/basm),
if you encounter any issues while using it please report it there so that I can fix the problem!

Basm is a very simple, assembly-like esoteric programming language made to transpile into Brainfuck (aka `bf` for short).
My goal in creating basm was to prove (although it had already been done before) that bf can be used to write any program
a user can think of. Whilst bf is turing-complete, it is rare to see that turing-complete*ness* actually exploited to
write generic, everyday programs *understandably so*. Hopefully, by providing tooling to abstract over writing bf, it can become
feasable to write anything in it.
Making us finally free from the cage of that is this language with a *funny name*.

By aiming to be transpiled into bf, basm shares many similarities with bf. Certain pieces of logic, like instructions, are transpiled pretty much
1:1 into bf. We even allow writing raw bf in a basm source file. Basm does seperate itself from bf where it matters however.
I've summed up the painpoints I've aimed to solve when writing bf programs:

* Relative nature of memory
* Lack of organisation or code-reuse features
* Difficulties with text handleing

## Relative Nature of Memory

In bf, you can't just say: `add cell no.3 to cell no.2`. You need to actually manage the tape pointer and most likely
do mental arithmetic to calculate how many `>` and `<` you'll need to get to the cells you want.
So, you'd better not mess up your bf tape pointer or you might end up with a silent bug killing
both your program and any hope of creating anything productive with bf that may have inhabited you before.
You may even end up in a situation where you don't know exactly where you are, because you conditinally moved the tape pointer.
This can happen in a loop like this for example: `[>]`.

You could say that bf dynamically addresses its memory by default. It says: `get the cell -1 from me`. Basm tries to solve this by having a static addressing system.
This means that, in a basm source file, you can safely refer to the 3rd cell and it will always be the third cell
no matter the operations you performed before.
That's possible thanks to the tape pointer being automatically managed by the compiler.
Also, because of the abstraction over raw bf operations, basm makes it impossible to reach a "relative state" without you wanting to. You cannot accidentally write `[>]`!
You must implicitly use a `RAW` or `ASUM` operation with the intent of entering a relative state to enter a relative state. Reaching a relative state with these two operation is valid as it is needed to write things like dynamic array gettin/setting. *(Implementing these operations is really interesting and involves a lot of language features that I describe throghly???? in the `Writing Relative Code` chapter)*

In bref, relative is when you don't know where the cell pointer is in the program.
As a rule of thumb when programming in basm: relative bad *(mostly)*, static good *(mostly)*.

## Lack of Organisation or Code-reuse Features

Bf has virtually nothing to help developers, maybe that's why it got so popular, *maybe people like torture...*
Welp, I don't! So, I filled basm with plenty of ways to help you abstract over bf.

Firstly, in basm, the base building block of our program are not operators, but instead instructions.
This gives the language a bigger indivisible piece of logic compared to bf.
`INCR 0 16;` is much easier to conceptualise than `++++++++++++++++`, don't you think?

Secondly, we have aliases with the `ALIS` operation, which allow you to
give name to values, addresses or blocks of code.
*In this household (programming language), we call blocks of code "scopes".*

Lastly, there are "meta-instructions" which are kinda similar to functions, but in implementation are moreso similar to macros.
These meta-instructions, once defined can be used like any in-built instruction in the program.
Since bf doesn't have a jump operator and data is seperated from code, it's practically impossible to create functions.
So, meta-instructions, as the name implies, are simply instructions made up of other lower-level instructions.
Hence why they are more similar to macros than functions, they are all inlined when used.

## Difficulties With Text Handleing

Bf works with text like any other programming language would, it stores one character by cell in Unicode format.
The problem with that is that bf is not like any other programming language in pretty much all regards!
First and foremost, there is no way to set a cell to a specific value directly. If you want the caracter 'c',
then hold on tight, because you will need to type `+` 99 times!
Basm implements both a character literal and a string literal, thus with basm you only need to type `INCR 0 'c';` to set cell no.0 to the value of 'c'.
There are also in-built operations for loading and printing strings, which is a normally a very tedious task in bf.

## Quick Note on Code Formating Throughout This Book

This book will contain and promote how I prefer to write code in basm.
Almost nothing in the language forces a specific formatting standard.
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
language dictating how you should use whitespaces (except for seperating identifiers).
So, keep in mind that whilst I like to follow certain naming and formatting standards,
the flexible nature of basm allows you to write it however your heart desires!

---

Now that you know wheter or not basm is for you, let's hop in!

## Disclamer

Basic knowlege of bf is required to understand this book. That is, you need to know the operators do and little else.
If you read up to here I'm guessing, you're probably fine on that.

Both the language and transpiler are my first foray into language development and are
serving as my first real major project in Rust.
There have been better implementations of bf transpilable languages which are both more efficient are easier to use.
**This project is only for my own personal pleasure as a creator and is not aimed at creating efficient bf programs.
If you are seeking useable and well informed implementations of bf transpilers look [here](https://esolangs.org/wiki/Brainfuck_code_generation).**

# Brain Aneurysm

![basm-logo](./resources/logo.png)

Brain Aneurysm (or just `basm` for short) is a very-simple assembly-like esoteric programming language transpiling to brainfuck.

The purpose of this language is to abstract of the parts of BrainFuck that makes it *fucky* like:
* The relative nature of memory,
* Difficulty working with text,
* The lack of organisation features,

Mitigating these problems will hopefully allow writing complex programs.


My main motivation for this project is to prove that if it is turing complete, it can do anything any other language could.
Whilst bf is very well known for being one of the smallest languages that is turing complete,
I believe that not many people have actually used it to create works that take advantage of the theoratically
all-purpose nature of bf. Whilst I say this, i know fully that this language is not and will never be the right
tool for the job of creating programs in this niche language that is bf. It is moreso a learning experience for myself.

## Disclamer
**This project is only for my own personal pleasure as a creator and is not aimed at creating efficient bf programs.
If you are seeking useable and well informed implementations of bf transpilers look [here](https://esolangs.org/wiki/Brainfuck_code_generation).**

## Example:
Here is how you would write a little program to output the 12th number is the fibonacci sequence (144):
```basm
// this demo assumes that the interpreter interprets output as number values rather than characters.

// defines a meta-instruction with arguments Asrc, Adst and sp (stack pointer) named COPC for "COPy Conservative"
// (meta-instructions are not forced to have 4 letter names or be all uppercase)
[@COPC Asrc Adst sp] [
// the syntax is as follows: INSTRUCTION (zero or more OPERAND/SCOPE) ;
ALIS Atmp sp;
ALIS sp sp+1;

COPY Asrc Adst Atmp;
ADDP Asrc Atmp;
]

[main] [
// defines aliases to integers
ALIS Aindex 0;
ALIS Aa 1;
ALIS Ab 2;
ALIS sp 3;

ALIS Vlimit 11;

// sets the default value of cells
INCR Aa 1;
INCR Ab 0;

// while the iteration index is not equal to the limit we move forward in the sequence
WHNE Aindex Vlimit [
	ALIS Aold_a 3;
	ALIS sp sp+1;

	INCR Aindex 1;
	COPC Aa Aold_a sp;

	// we add both cell, giving us the result in Aa
	ADDP Aa Ab;

	ADDP Ab Aold_a;
];

OUT Aa;
]
```

And that source code would give us this brainfuck code:
```b
>+<-----------[+>[->>+>+<<<]>[-<+>]>>[-<<<+>>>]<[-<+>]<<<]+++++++++++>.
```

(if you want a more complex example you can inspect [the working bf interpreter in basm that i wrote](./test-resources/bf-interpreter.basm).)

started as of 2024-10-12

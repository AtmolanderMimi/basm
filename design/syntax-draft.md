# Brainfuck Unfucked Syntax

## Prelude

I am going to try to design this as to be as easy as possible to parse and transpile into bf, this means:

* no for loops
* no methods (but yes compound types later)
* no syntactic distinction between constant and variable

## Example

```rs // would use bfu, but they are fairly similar
// this is simply a macro that pastes the content of the file here
// in this case std is reserved for a standard library
import "std"

// all bfu programs should have an entry point with this function signature
// because of the distinction between code memory and variable memory it
// would be harder to not have an entry point
fn main() { // all variables need to have their type defined, sorry, no sorry
	let variable_name: Num = 4;

	// all functions are inlined, because, well bf
	// (although it may not be impossible, it would require setting up stuff
	// on the tape, which would greatly increase complexity and inefficiency)
	// this has the side effect of making functions unhable to recurse
	let fib_result: Num = fib(&variable_name);

	// print is provided by a standard library
	// i won't go into much detail about strings, but i would implement it
	// like in c, an array of Char
	print_str("The ");
	print_num(variable_name);
	print_str("th element of the fibonacci sequence has a value of ")
	print_num(fib_result);
	// Char's hold the same space as Num's (1 slot), but they have different
	// sementic meaning and thus different functions associated with them.
	// printing a Num of value 99 would get you "99"
	// printing a Char of value 'c' (99) would get you "c"
	print_char('\n');
}

fn fib(n: Num): Num {
	let a: Num = 0;
	let b: Num = 1;
	while n != 0 {
		// &b: "the value of" b
		// by default all operations are destructive
		// including moves, because bf
		let c: Num = &b;
		b = a + b; // here these destroy both a and b making them invalid
		a = c;

		// maybe i can switch this to "n--" to simplify to the translating
		n = n - 1;
	}

	n // no early return is allowed
}
```

The example above should print out "The 4th element of the fibonacci sequence has a value of 5\n"

This example allows us to better understand what →I (ME)← want as syntactic items.

## Expressions

### Literals

Num lit ex:  0, 72, 255
Str lit ex:  "Hello, World" -> [Char; 13] (because of the added nul)
arr lit ex: ['H', 'e', 'l', 'l', 'o']
Char lit ex: 'c', '\n'
Bool lit ex: true, false, maybe(?)

### Binary Expressions

note: there won't be unary expressions because we simply don't
need them, for example "!bool" could be written "bool == false"
they can also be made into functions if i get to add them later
Num -> Num expressions: +, -, *, /(?), % and |- (abs diff)
Num -> Bool expressions: ==, !=, >, <, >= and <=
Char -> Bool expressions: == and !=
Bool -> Bool expressions: == and !=

### Unary Experession

actually small caviat to the note above:

\* -> * expressions: &var (clone)

### Function calls

Function calls function like expressions,
to be more precise the last expressions of a function body (if any)
gets returned as the result of that expression.

## Statements

### Variable Declaration

Variables can be declared like this:
`let ident = value`

### Functions Declaration

Function declaration are made in this format:
`fn function_name(func_arg1: type) -> return_val { body }`

### Loops/Branches

Loops and branches are in this format:
`while expr { body }` & `if expr { body }`

## Syntax Tokens

With all of that we can set down basic tokens to define the syntax
of the language:

| Written Form                                                  | Token   |
| --------------------------------------------------------------- | --------- |
| fn                                                            | FnDecl  |
| let                                                        | VarDecl |
| [ident]**:** [type] | TypeDecl
|+, -, *, %, ==, !=, >...|BinaryOp
|=|AssignOp|
|&|CloneOp|
|;|StatementDelimiter|
|72|NumLit|
|"Hello World"|StrLit|
|'c'|CharLit|
|true/false|BoolLit|
|import|Import|
|while|While|
|if|If|
|,|ElementSeperator|
|[|LSquare|
|]|RSquare|
|{|LCurly|
|}|LCurly|
|(|LParentheses|
|)|RParentheses|
|*(any other alpha numeric squence that starts with a letter)* |Ident|

written 2024-10-12


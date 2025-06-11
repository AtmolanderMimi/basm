# Addendum - [setup] scope
After having fully written this book and as I started working on an improved version of the current bf interpreter
(the naive version of which is the one discussed in this very book),
I finally caved to implement something which had been bugging my mind for a long time: a way to have global aliases.
Such a feature always was on the list of things to introduce into the language had I had enough dedication and time,
and as things went forth and as I made, quite honestly, fairly simple programs I thought that such a thing would not be required for v1.0.
The full breath of consequences of this retrospectively earnestly stupid decision of mine only came down onto me
as finally undertook a true "project of size", which was as previously stated the improved version of the bf interpreter in basm.
Such an undertaking required to refactor and modify many values which were repeated all over,
like the ids associated with each bf operator and the size of dynamically readable arrays.

So, to solve this I added one more field: the `[setup]` field.
Such a field had always been in my notes, only it is now that I see that this field is truely required to make a workable language.

## What is it?
The `[setup]` field is very similar to `[main]` in function. It's scope contains zero or more scopes or instructions.
Any code included within `[setup]` will be put right before `[main]`.
There can only be one `[setup]` field.
Right about now this field sounds pretty useless: "Just another place to put code in..."
Well, not really. Where `[setup]` shines is in it's two unique characteristics:
* It is normalized before any other fields, and
* **All aliases contained within the lowest scope are available from anywhere in the file**

### "It is normalized before any other fields"
This statement may not seem like too much, but it's ramifications are immense.
At least if you can understand it. In the case of basm, normalizing means replacing aliases by their value.
This means that meta-instructions are not normalized before `[setup]`.
Then logically, meta-instructions cannot be used in `[setup]`.
To be more precise, meta-instructions are not declared when `[setup]` is normalized,
so if there are any meta-instructions they will raise an error stating that they are undefined.
Despite the fact that they may be defined elsewhere in the file, they are not defined at the time of `[setup]` normalization.
A meta-instruction cannot be inlined as it is not defined!

This clause is limits quite a lot your ability to put logic within this field, however it is required for the second much, more empowering characteristic, of `[setup]`.

### "All aliases contained within the lowest scope are available from anywhere in the file"
This does what it says on the tin.
Any scope, including most importantly the scopes of meta-instructions, can access aliases defined in `[setup]`
unless that alias has been shadowed in scopes lower than the current one.
That behaviour allows the creation of aliases available through the whole program, practically creating global aliases.
Although, the wording on this is a little clumsy.
"anywhere in the file", in this context, means that they are available in any scope other than the ones contained within `[setup]`.
Of course, `[setup]` could not reference an alias which it has not yet created.

Here's an example of the global aliases at work:
```basm
[setup] [
// the idiomatic way to name global aliases is to prefix with "G"
ALIS GVfrob 42;
ALIS GVdefault 10;

// any built-in instruction can be inserted here,
// but it is not recommended to do so for anything other than value setting
]

[@FROB Acell] [
    INCR Acell GVfrob;
]

[main] [
    ALIS Acell 0;
    INCR Acell GVdefault;
    FROB Acell;

    OUT Acell; // should return 52


    // overwriting the global (only affects this scope and subscopes)
    ALIS GVdefault 5;
    // this will overwrite the global in this scope, but not in the meta
    // since meta-instructions aliases are independent from the caller's
    ALIS GVfrob 0;

    ALIS Acell 1;
    INCR Acell GVdefault;
    FROB Acell;

    OUT Acell; // should return 47 (despite the fact that we set GVfrob to 0 in main)
]
```
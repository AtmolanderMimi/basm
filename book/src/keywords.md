# Keywords
Here is a list of keywords specific to basm and their meaning.

* `static`: an access to memory which the address is known to the compiler at compile time. It does not vary over executions.
* `dynamic/relative`: an access to memory which the address is not known to the compiler at compile time. It can vary over executions.
* `inlining`: expanding the code refered to in the caller's body, all meta-instructions inline in basm.
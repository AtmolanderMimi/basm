# Keywords
Here is a list of keywords specific to basm and their meaning.

* `allocated`: a cell is said to be "allocated" when it is reserved/used by an operation
* `dynamic/relative`: an access to memory which the address is not known to the compiler at compile time. It can vary over executions.
* `static`: an access to memory which the address is known to the compiler at compile time. It does not vary over executions.
* `inlining`: expanding the code refered to in the caller's body, all meta-instructions inline in basm.
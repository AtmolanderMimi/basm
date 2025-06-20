[x] fix char parsing bug
[] improve error reporting on parsing
[x] improve error reporting on inlined scope, ex: `IFEQ .. [ error here ] ..;` gets reported at rather than at the IFEQ: `INLN [scp];`
[x] bf-interpreter.basm in test-ressources should be updated to match book version or better
[x] compiled files with "basm compile" should output in cwd (like gcc) rather than source file directory


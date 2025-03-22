# Built-in Instructions

| Name | Arguments | Function |
|-|-|-|
| **ZERO** | addr | sets the value of `addr` to 0 |
| **INCR** | addr, value | increments the value of the `addr` cell by `value` |
| **DECR** | addr, value | decrements the value of the `addr` cell by `value` |
| **ADDP** | addr1, addr2 | adds `addr2` to `addr1` in place |
| **SUBP** | addr1, addr2 | substract `addr2` from `addr1` in place |
| **COPY** | addr1, addr2, addr3 | copies the value of `addr1` into `addr2` and `addr3` |
| **WHNE** | addr, value, [scope] | while the value of `addr` cell is not equal to `value` runs the `[scope]`. `addr` is not consumed |
| **IN**   | addr | takes input form the user and sets it in `addr`, behaviour will vary between bf implementations |
| **OUT**  | addr | send `addr` to the output, `addr` is not consumed |
| **LSTR** | start_addr, "str" | loads the string character by character into cells from the `start_addr` |
| **PSTR** | addr, "str" | prints the string character by character using the cell at `addr` to buffer the bytes |
| **ALIS** | ident, value | aliases a value or scope to an identifier, this instruction is purely abstraction |
| **INLN** | [scope] | inlines a scope |
| **RAW**  | "str" | inlines the string in the transpiled code, this can be used to include brainfuck operators |
| **BBOX** | addr | moves the tape pointer to the address of `addr` |
| **ASUM** | addr | tells to compiler to assume that the tape pointer is at `addr`, if that assumption is wrong all cells accesses will be offset |
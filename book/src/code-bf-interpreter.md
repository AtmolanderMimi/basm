# Full Code
For your copy-pasting convinience, here is the full file for the bf interpreter
as implemented in this book:
```basm
// sets a value to a specific value by zeroing it before writing
[@SET Aaddr Vval] [
ZERO Aaddr;
INCR Aaddr Vval;
]

// copies the content of a cell whilst keeping the source (conservatively)
[@COPC Asrc Adst sp] [
	ALIS Atmp sp;
	COPY Asrc Adst Atmp;
	ADDP Asrc Atmp;
]

// if not equal conditional
[@IFNE Aaddr Vval [scp] sp] [
ALIS Atmp1 sp;
ALIS Atmp2 sp+1;
ALIS sp sp+2;

COPY Aaddr Atmp1 Atmp2;
ADDP Aaddr Atmp2;

WHNE Atmp1 Vval [
    ZERO Atmp1;

    INLN [scp];

    INCR Atmp1 Vval;
];

ZERO Atmp1;
]

// if equal conditional
[@IFEQ Aaddr Vval [scp] sp] [
    ALIS Aflag sp;
    ALIS sp sp+1;
    ALIS Vnot_equal 1;

    IFNE Aaddr Vval [
        INCR Aflag Vnot_equal;
    ] sp;

    IFNE Aflag Vnot_equal [scp] sp;
    ZERO Aflag;
]

// moves the value of the cell at Aarray[value of Aindex] to Adst
[@GETD Aarray Aindex Adst] [
ADDP Aarray+3 Aindex; 

BBOX Aarray+1;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1;
ALIS Aindex 2;
ALIS Aelement 3;

WHNE Aindex 0 [
    DECR Aindex 1;
    INCR Areturn 1;

    ADDP Aswap Aelement;

    ADDP Aindex+1 Aindex;
    ADDP Areturn+1 Areturn;

    BBOX 1;
    ASUM 0;
];

ALIS Acell Aindex;

ADDP Acell Aelement;

ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1;

    ADDP Areturn-1 Areturn;
    ADDP Acell-1 Acell;

    BBOX 0;
    ASUM 1;

    ADDP Aswap Aelement;
];

BBOX Aelement;
ASUM Aarray+1;

ADDP Adst Aarray+3;
]

// adds the value of the cell at Asrc to the Aarray[value of Aindex] cell
[@ADDD Aarray Aindex Asrc] [
ADDP Aarray+3 Asrc;
ADDP Aarray+2 Aindex;

BBOX Aarray;
ASUM 0;

ALIS Aswap 0;
ALIS Areturn 1;
ALIS Aindex 2;
ALIS Acell 3;
ALIS Aelement 4;

WHNE Aindex 0 [
    DECR Aindex 1;
    INCR Areturn 1;

    ADDP Aswap Aelement;

    ADDP Acell+1 Acell;
    ADDP Aindex+1 Aindex;
    ADDP Areturn+1 Areturn;

    BBOX 1;
    ASUM 0;
];

ADDP Aelement Acell;

ALIS Aswap Aelement;
ALIS Aelement 0;

WHNE Areturn 0 [
    DECR Areturn 1;

    ADDP Areturn-1 Areturn;

    BBOX 0;
    ASUM 1;

    ADDP Aswap Aelement;
];

BBOX Aelement;
ASUM Aarray;
]

[@init_input Aarray] [
BBOX Aarray+4;
ASUM 0;

ALIS Aflag 1;
ALIS Vappended 1;
ALIS Vexit 2;

ALIS sp 2;
// we can still use sp, since we know the cells will be zeroed

WHNE Aflag Vexit [
    IN 0;

    // -- mapping operators --
    IFEQ 0 '+' [
        SET 0 1;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '-' [
        SET 0 2;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '>' [
        SET 0 3;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '<' [
        SET 0 4;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '[' [
        SET 0 5;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 ']' [
        SET 0 6;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 ',' [
        SET 0 7;
        INCR Aflag Vappended;
    ] sp;
    IFEQ 0 '.' [
        SET 0 8;
        INCR Aflag Vappended;
    ] sp;

    // check for end
    IFEQ 0 '!' [
        ZERO 0;
        SET Aflag Vexit;
    ] sp;

    // if we added something, we need to move up one
    IFEQ Aflag Vappended [
        ZERO Aflag;

        BBOX 1;
        ASUM 0;
    ] sp;

    // we don't clear the last character, even if it was invalid,
    // as the next IN will overwrite the cell
];

// NEVER forget cleanup
ZERO Aflag;

// -- going back --
// because the last char will be '!', we need to offset by at least 1
BBOX 0;
ASUM 1;

// moves back until we reached the zeroed cells of the parking
WHNE 0 0 [
    BBOX 0;
    ASUM 1;
];

ASUM Aarray+3;
]

[main] [
ALIS Voperating_memory 16;
ALIS Aprog_pointer 0;
ALIS Amem_pointer 1;
ALIS Aflag 2;
ALIS Vexit 1;
ALIS Vbracket 2;
ALIS Aoperator 3;
ALIS Acell 4;
ALIS Abracket_jump 5;
ALIS sp 6;

ALIS Vprog_array_cap 256;
ALIS Aprog Voperating_memory;
ALIS Amem Aprog+Vprog_array_cap+4;

// get program from user
init_input Aprog;

WHNE Aflag Vexit [
    ALIS Atmp sp;
    ALIS sp sp+1;

    // 1. load in operator
    COPC Aprog_pointer Atmp sp;
    GETD Aprog Atmp Aoperator;

    // 2. operator specific logic
    // + + + + +
    IFEQ Aoperator 1 [
        INCR Acell 1;
    ] sp;

    // - - - - -
    IFEQ Aoperator 2 [
        DECR Acell 1;
    ] sp;

    // > > > > >
    IFEQ Aoperator 3 [
        // store the old cell
        COPC Amem_pointer Atmp sp;
        ADDD Amem Atmp Acell;

        // get the new cell
        INCR Amem_pointer 1;
        COPC Amem_pointer Atmp sp;
        GETD Amem Atmp Acell;
    ] sp;

    // < < < < <
    IFEQ Aoperator 4 [
        // store the old cell
        COPC Amem_pointer Atmp sp;
        ADDD Amem Atmp Acell;

        // get the new cell
        // NOTE: this DECR may underflow the tape pointer
        DECR Amem_pointer 1;
        COPC Amem_pointer Atmp sp;
        GETD Amem Atmp Acell;
    ] sp;

    // [ [ [ [ [
IFEQ Aoperator 5 [
    IFEQ Acell 0 [
        ALIS Acounter sp;
        INCR Acounter 1; // 1.
        ALIS Asearch_op sp+1;
        ALIS sp sp+2;

        // 2.
        // we don't need to define our search counter here,
        // we can use Abracket_jump directly
        COPC Aprog_pointer Abracket_jump sp;
        WHNE Acounter 0 [
            // 3.
            INCR Abracket_jump 1;

            // 4.
            COPC Abracket_jump Atmp sp;
            GETD Aprog Atmp Asearch_op;

            // 5.
            // if op == '['
            IFEQ Asearch_op 5 [
                INCR Acounter 1;
            ] sp;
            // if op == ']'
            IFEQ Asearch_op 6 [
                DECR Acounter 1;
            ] sp;

            // 6.
            COPC Abracket_jump Atmp sp;
            ADDD Aprog Atmp Asearch_op;
        ]; // 7.

        INCR Aflag Vbracket;
    ] sp;
] sp;

// ] ] ] ] ]
IFEQ Aoperator 6 [
    IFNE Acell 0 [
        ALIS Acounter sp;
        INCR Acounter 1; // 1.
        ALIS Asearch_op sp+1;
        ALIS sp sp+2;
    
        // 2.
        COPC Aprog_pointer Abracket_jump sp;
        WHNE Acounter 0 [
            // 3.
            DECR Abracket_jump 1;
    
            // 4.
            COPC Abracket_jump Atmp sp;
            GETD Aprog Atmp Asearch_op;
    
            // 5.
            // if op == '['
            IFEQ Asearch_op 5 [
                DECR Acounter 1;
            ] sp;
            // if op == ']'
            IFEQ Asearch_op 6 [
                INCR Acounter 1;
            ] sp;
    
            // 6.
            COPC Abracket_jump Atmp sp;
            ADDD Aprog Atmp Asearch_op;
        ]; // 7.

        INCR Aflag Vbracket;
    ] sp;
] sp;

    // , , , , ,
    IFEQ Aoperator 7 [
        IN Acell;
    ] sp;

    // . . . . .
    IFEQ Aoperator 8 [
        OUT Acell;
    ] sp;

    // check for the end
    IFEQ Aoperator 0 [
        INCR Aflag Vexit;
    ] sp;

    // set the operator back
    COPC Aprog_pointer Atmp sp;
    ADDD Aprog Atmp Aoperator;

    IFEQ Aflag Vbracket [
        ZERO Aprog_pointer;
        ADDP Aprog_pointer Abracket_jump;
        ZERO Aflag;
    ] sp;

    // increase program pointer
    INCR Aprog_pointer 1;
];
]
```

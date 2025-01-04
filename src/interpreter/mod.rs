use std::{any::Any, io::{self, Write}, usize};

use num::{traits::{ConstOne, ConstZero, SaturatingAdd, SaturatingSub, WrappingAdd, WrappingSub}, CheckedAdd, CheckedSub, Num, NumCast};
use thiserror::Error;


/// Interpreter for brainfuck programs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interpreter<T> {
    config: InterpreterConfig,
    tape: Vec<T>,
    instructions: Vec<char>,
    tape_pointer: usize,
    instruction_pointer: usize,
}

impl<T> Interpreter<T> {
    fn new(instructions: Vec<char>, config: InterpreterConfig) -> Interpreter<T> {
        Interpreter {
            config,
            instructions,
            tape: Vec::new(),
            tape_pointer: 0,
            instruction_pointer: 0,
        }
    }
}

impl<T> Interpreter<T> {
    /// Creates a builder for [`Interpreter`].
    pub fn builder(instructions: &str) -> InterpreterBuilder {
        InterpreterBuilder::new(instructions)
    }
}

impl<T: Default + Clone> Interpreter<T> {
    fn get_mut_cell_or_insert_default(&mut self) -> Result<&mut T, InterpreterError> {
        if self.tape.len() > self.tape_pointer {
            return Ok(self.tape.get_mut(self.tape_pointer).unwrap());
        }

        // if we get here then the tape is not long enough for our index
        let extention = (self.tape_pointer+1) - self.tape.len();
        let lenght_limit = self.config.lenght_limit.unwrap_or(usize::MAX);
        if self.tape_pointer+1 > lenght_limit {
            return Err(InterpreterError::TapeLimitExceded { limit: lenght_limit, tried: self.tape_pointer+1 });
        }

        self.tape.extend(vec![T::default(); extention]);

        Ok(self.tape.get_mut(self.tape_pointer).unwrap())
    }
}

/// Trait for [`Interpreter`] behaviour.
pub trait InterpreterTrait {
    /// Advances for one instruction.
    /// Returns `false` if the pointer is out of the instruction list. (aka) it is done.
    fn advance(&mut self) -> Result<bool, InterpreterError>;

    /// Runs until it either runs into an error or completes the program.
    fn complete(&mut self) -> Result<(), InterpreterError> {
        while self.advance()? {}

        Ok(())
    }

    /// References the tape.
    /// 
    /// # **THIS IS A REFERENCE TO `Vec<T>`**.
    // I HATE ANY, WHY IS IT SO HARD TO DOWNCAST
    fn tape(&self) -> &dyn Any;
}

impl<T: NumOpsPlus + Default + Clone + 'static> InterpreterTrait for Interpreter<T> {
    fn advance(&mut self) -> Result<bool, InterpreterError> {
        let instruction = if let Some(i) = self.instructions.get(self.instruction_pointer) {
            i
        } else {
            return Ok(false);
        };

        match instruction {
            '+' => {
                let overflow_behaviour = self.config.overflow_behaviour.clone();

                let value = self.get_mut_cell_or_insert_default()?;
                match overflow_behaviour {
                    OverflowBehaviour::Abort => {
                        if let Some(nval) = value.checked_add(&T::ONE) {
                            *value = nval;
                        } else {
                            return Err(InterpreterError::AbortedDueToOverflow { at: self.tape_pointer })
                        }
                    },
                    OverflowBehaviour::Saturate => *value = value.saturating_add(&T::ONE),
                    OverflowBehaviour::Wrap => *value = value.wrapping_add(&T::ONE),
                }
            },

            '-' => {
                let overflow_behaviour = self.config.overflow_behaviour.clone();

                let value = self.get_mut_cell_or_insert_default()?;
                match overflow_behaviour {
                    OverflowBehaviour::Abort => {
                        if let Some(nval) = value.checked_sub(&T::ONE) {
                            *value = nval;
                        } else {
                            return Err(InterpreterError::AbortedDueToOverflow { at: self.tape_pointer })
                        }
                    },
                    OverflowBehaviour::Saturate => *value = value.saturating_sub(&T::ONE),
                    OverflowBehaviour::Wrap => *value = value.wrapping_sub(&T::ONE),
                }
            },

            '>' => self.tape_pointer += 1,

            '<' => {
                if let Some(npoint) = self.tape_pointer.checked_sub(1) {
                    self.tape_pointer = npoint;
                } else {
                    return Err(InterpreterError::TapePointerOob)
                }
            },

            '.' => {
                let value = self.get_mut_cell_or_insert_default()?;
                let ch = char::from_u32(value.to_u32().unwrap_or(65_533)).unwrap_or('�');
                print!("{ch}");
                let _ = io::stdout().flush();
            },

            ',' => {
                let cell = self.get_mut_cell_or_insert_default()?;

                loop {
                    let something = read_char_input();
                    if let Some(nch) = something {
                        if let Some(ncell) = T::from(nch as u32) {
                            *cell = ncell;
                            break
                        };
                    }
                }
            },

            '[' => {
                if *self.get_mut_cell_or_insert_default()? != T::ZERO {
                
                } else {
                    let mut bracket_count = 1;
                    self.instruction_pointer += 1;
                    while let Some(nch) = self.instructions.get(self.instruction_pointer) {
                        match *nch {
                            ']' => bracket_count -= 1,
                            '[' => bracket_count += 1,
                            _ => (),
                        }

                        if bracket_count == 0 {
                            break
                        }

                        self.instruction_pointer += 1;
                    }
                }
            },

            ']' => {
                if *self.get_mut_cell_or_insert_default()? == T::ZERO {
                
                } else {
                    let mut bracket_count = 1;
                    if let Some(npoint) = self.instruction_pointer.checked_sub(1) {
                        self.instruction_pointer = npoint;
                    } else {
                        return Err(InterpreterError::InstructionPointerOob)
                    };

                    while let Some(nch) = self.instructions.get(self.instruction_pointer) {
                        match *nch {
                            ']' => bracket_count += 1,
                            '[' => bracket_count -= 1,
                            _ => (),
                        }

                        if bracket_count == 0 {
                            break
                        }

                        if let Some(npoint) = self.instruction_pointer.checked_sub(1) {
                            self.instruction_pointer = npoint;
                        } else {
                            return Err(InterpreterError::InstructionPointerOob)
                        };
                    }
                }
            },

            _ => panic!("Unrecognised"),
        }

        self.instruction_pointer += 1;

        Ok(true)
    }

    fn tape(&self) -> &dyn Any {
        &self.tape
    }
}

/// Builder for [`Interpreter`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InterpreterBuilder {
    instructions: Vec<char>,
    inner: InterpreterConfig,
}

impl InterpreterBuilder {
    /// Creates a new [`InterpreterBuilder`].
    pub fn new(instructions: &str) -> Self {
        let instructions = instructions.chars().filter(|c| {
            *c == '+'
            || *c == '-'
            || *c == '<'
            || *c == '>'
            || *c == ','
            || *c == '.'
            || *c == '['
            || *c == ']'
        }
        ).collect();

        InterpreterBuilder {
            instructions,
            inner: InterpreterConfig::default(),
        }
    }

    /// Sets the cell type to unsigned integers of 8 bits (`u8`).
    pub fn with_u8(mut self) -> Self {
        self.inner.cell_kind = CellKind::U8;
        self
    }

    /// Sets the cell type to unsigned integers of 16 bits (`u16`).    
    pub fn with_u16(mut self) -> Self {
        self.inner.cell_kind = CellKind::U16;
        self
    }
    
    /// Sets the cell type to unsigned integers of 32 bits (`u32`).
    pub fn with_u32(mut self) -> Self {
        self.inner.cell_kind = CellKind::U32;
        self
    }
    
    /// Sets the cell type to signed integers of 8 bits (`u8`).
    pub fn with_i8(mut self) -> Self {
        self.inner.cell_kind = CellKind::I8;
        self
    }
    
    /// Sets the cell type to signed integers of 16 bits (`u16`).
    pub fn with_i16(mut self) -> Self {
        self.inner.cell_kind = CellKind::I16;
        self
    }
    
    /// Sets the cell type to signed integers of 32 bits (`u32`).
    pub fn with_i32(mut self) -> Self {
        self.inner.cell_kind = CellKind::I32;
        self
    }

    /// Sets a limit to the tape lenght of `lenght` cells.
    pub fn with_tape_leght(mut self, lenght: usize) -> Self {
        self.inner.lenght_limit = Some(lenght);
        self
    }

    /// Does not set a limit to the tape lenght. It will grow as much as needed.
    pub fn without_tape_lenght(mut self) -> Self {
        self.inner.lenght_limit = None;
        self
    }

    /// Sets the overflow behaviour to wrapping. 
    pub fn with_wrapping_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Wrap;
        self
    }

    /// Sets the overflow behaviour to saturating.     
    pub fn with_saturating_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Saturate;
        self
    }

    /// Sets the overflow behaviour to aborting. 
    pub fn with_aborting_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Abort;
        self
    }

    /// Finishes the building process.
    pub fn finish(self) -> Box<dyn InterpreterTrait> {
        match self.inner.cell_kind {
            CellKind::U8 => Box::new(Interpreter::<u8>::new(self.instructions, self.inner)),
            CellKind::U16 => Box::new(Interpreter::<u16>::new(self.instructions, self.inner)),
            CellKind::U32 => Box::new(Interpreter::<u32>::new(self.instructions, self.inner)),
            CellKind::I8 => Box::new(Interpreter::<i8>::new(self.instructions, self.inner)),
            CellKind::I16 => Box::new(Interpreter::<i16>::new(self.instructions, self.inner)),
            CellKind::I32 => Box::new(Interpreter::<i32>::new(self.instructions, self.inner)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct InterpreterConfig {
    cell_kind: CellKind,
    lenght_limit: Option<usize>,
    overflow_behaviour: OverflowBehaviour,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum CellKind {
    #[default]
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
}

#[derive(Debug, Clone, PartialEq, Default)]
enum OverflowBehaviour {
    #[default]
    Wrap,
    Saturate,
    Abort,
}

trait NumOpsPlus: WrappingAdd + WrappingSub + CheckedAdd + CheckedSub + SaturatingAdd + SaturatingSub
    + Num + ConstOne + ConstZero + NumCast {}
impl<T> NumOpsPlus for T 
where T: WrappingAdd + WrappingSub + CheckedAdd + CheckedSub + SaturatingAdd + SaturatingSub
    + Num + ConstOne + ConstZero + NumCast {}

/// An error that is relative to interpreting a brainfuck program.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum InterpreterError {
    /// The tape size limitor is smaller that what the program tried to allocate.
    /// Allocation is implicit, it happens when trying to write or read for a cell. (e.g.: every operation execept `>` and `<`)
    #[error("the tape size limit was exceded, tried to expand to {tried:?}, but limit is {limit:?}")]
    TapeLimitExceded {
        /// The tape size limit.
        limit: usize,
        /// The cell index that was requested for reading/writing.
        tried: usize,
    },
    /// The tape pointer is not within the range of valid values. (e.g `0..=usize::MAX`)
    #[error("the tape pointer has gone into negatives")]
    TapePointerOob,
    /// The instruction pointer is not within the range of valid values. (e.g `0..=usize::MAX`)
    /// It is most likely that this is because there was an unmatched `]` cause the instruction pointer to go into negatives.
    #[error("the instruction pointer has gone into negatives")]
    InstructionPointerOob,
    /// The program over/under-flowed a cell.
    /// This error only happens if the interpreter was configured to abort on overflow.
    #[error("aborted due to overflow at cell {at:?}")]
    AbortedDueToOverflow {
        /// The index of the cell that over/under-flowed.
        at: usize,
    },
}

fn read_char_input() -> Option<char> {
    print!("\n?: ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);

    // we skip the first few we printed ourselves
    buf.chars().nth(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreter_multiply() {
        let mut inter = InterpreterBuilder::new("++++++++++++>++++<[->>+<<]>[>[->+<<<+>>]>[-<+>]<<-]")
            .with_u8()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[0], 48);
    }

    #[test]
    fn interpreter_fibonacci() {
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
            .with_u8()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[1], 144);
    }

    #[test]
    fn interpreter_cell_types() {
        // u8
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
            .with_u8()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[1], 144_u8);

        // i8 (should overflow)
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
            .with_i8()
            .with_aborting_behaviour()
            .finish();
        inter.complete().unwrap_err();
        
        // u16
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
        .with_u16()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u16>>().unwrap();
        assert_eq!(tape[1], 144_u16);

        // i16
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
        .with_i16()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<i16>>().unwrap();
        assert_eq!(tape[1], 144_i16);

        // u32
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
        .with_u32()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u32>>().unwrap();
        assert_eq!(tape[1], 144_u32);

        // i32
        let mut inter = InterpreterBuilder::new(include_str!("../../test-resources/fib.bf"))
        .with_i32()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<i32>>().unwrap();
        assert_eq!(tape[1], 144_i32);
    }

    #[test]
    fn interpreter_behaviour() {
        // -- wrapping
        // overflow
        let plus_256: String = ["+"; 257].concat();
        let mut inter = InterpreterBuilder::new(&plus_256)
            .with_u8()
            .with_wrapping_behaviour()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[0], 1);

        // underflow
        let mut inter = InterpreterBuilder::new("---")
            .with_u8()
            .with_wrapping_behaviour()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[0], 253);

        // -- abort
        // overflow
        let plus_256: String = ["+"; 256].concat();
        let mut inter = InterpreterBuilder::new(&plus_256)
            .with_u8()
            .with_aborting_behaviour()
            .finish();
        inter.complete().unwrap_err();

        // underflow
        let mut inter = InterpreterBuilder::new("-")
            .with_u8()
            .with_aborting_behaviour()
            .finish();
        inter.complete().unwrap_err();

        // -- saturating
        // overflow
        let plus_256: String = ["+"; 1024].concat();
        let mut inter = InterpreterBuilder::new(&plus_256)
            .with_u8()
            .with_saturating_behaviour()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[0], u8::MAX);

        // underflow
        let mut inter = InterpreterBuilder::new("---")
            .with_u8()
            .with_saturating_behaviour()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[0], 0);
    }

    #[test]
    fn interpreter_limitor() {
        let mut heavy_program = [">"; 1293].concat();
        heavy_program.push('+');

        // no limitor
        let mut inter = InterpreterBuilder::new(&heavy_program)
            .finish();
        inter.complete().unwrap();

        // with limitor, but big enough
        let mut inter = InterpreterBuilder::new(&heavy_program)
            .with_tape_leght(4096)
            .finish();
        inter.complete().unwrap();

        // with small limitor
        let mut inter = InterpreterBuilder::new(&heavy_program)
            .with_tape_leght(256)
            .finish();
        match inter.complete() {
            Err(InterpreterError::TapeLimitExceded { .. }) => (), // good
            other => panic!("got other {other:?}"),
        };
    }

    #[test]
    fn interpreter_tape_only_takes_necessary() {
        let mut program = [">"; 1293].concat();
        program.push_str(&["<"; 1200].concat());

        // takes nothing
        let mut inter = InterpreterBuilder::new(&program)
            .with_tape_leght(0)
            .finish();
        inter.complete().unwrap();

        // allocates what is needed, and is big enough
        program.push('+');
        let mut inter = InterpreterBuilder::new(&program)
            .with_tape_leght(256)
            .finish();
        inter.complete().unwrap();

        // allocates what is needed, but is too small
        let mut inter = InterpreterBuilder::new(&program)
            .with_tape_leght(64)
            .finish();
        match inter.complete() {
            Err(InterpreterError::TapeLimitExceded { .. }) => (), // good
            other => panic!("got other {other:?}"),
        };
    }
}
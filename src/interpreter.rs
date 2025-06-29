//! A fairly naive implementation of a brainfuck interpreter.

use std::{any::Any, collections::VecDeque, fmt::Debug, io::{self, Write}, str::FromStr};

use colored::Colorize as _;
use num::{traits::{ConstOne, ConstZero, SaturatingAdd, SaturatingSub, WrappingAdd, WrappingSub}, CheckedAdd, CheckedSub, Num, NumCast};
use thiserror::Error;


/// Interpreter for brainfuck programs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interpreter<T> {
    config: InterpreterConfig,
    tape: Vec<T>,
    instructions: Vec<ByteCode>,
    tape_pointer: usize,
    instruction_pointer: usize,

    input_buffer: VecDeque<T>,

    #[cfg(test)]
    captured_output: String,
}

// Generic, non-cell related methods.
impl<T> Interpreter<T> {
    fn new(instructions: Vec<ByteCode>, config: InterpreterConfig) -> Interpreter<T> {
        let mut interpreter = Interpreter {
            config,
            instructions,
            tape: Vec::new(),
            tape_pointer: 0,
            instruction_pointer: 0,

            input_buffer: VecDeque::new(),

            #[cfg(test)]
            captured_output: String::new(),
        };

        interpreter.calculate_bracket_matches();

        interpreter
    }

    /// Creates a builder for [`Interpreter`].
    pub fn builder(instructions: &str) -> InterpreterBuilder {
        InterpreterBuilder::new(instructions)
    }

    #[cfg(test)]
    /// Gives a string copying the output created by the program.
    pub fn captured_output(&self) -> &str {
        &self.captured_output
    }

    /// Finds and sets the matches for all brackets bytecodes.
    fn calculate_bracket_matches(&mut self) {
        let left_bracket_indexes = self.instructions.iter()
            .enumerate()
            .filter_map(|(i, b)| if let ByteCode::LeftBracket(_) = b { Some(i) } else { None })
            .collect::<Vec<_>>();

        for idx in left_bracket_indexes {
            // find the matching right bracket
            let mut bracket_count = 0;
            let mut matching_index = None;
            for (rel_jdx, instruction) in self.instructions[idx..].iter_mut().enumerate() {
                let jdx = rel_jdx + idx;

                match (bracket_count, instruction) {
                    (1, ByteCode::RightBracket(index_to_fill)) => {
                        matching_index = Some(jdx);
                        // we set the index for the matching left to the right bracket while we are at it
                        *index_to_fill = Some(idx);
                        break
                    },
                    (_, ByteCode::LeftBracket(_))=> bracket_count += 1,
                    (_, ByteCode::RightBracket(_)) => bracket_count -= 1,
                    _ => (),
                }
            }

            // set the index of what we found, (if we found nothing then matching_index = None)
            let left_bracket = self.instructions.get_mut(idx).unwrap();
            if let ByteCode::LeftBracket(index_to_fill) = left_bracket {
                *index_to_fill = matching_index;
            } else {
                panic!("should be LeftBracket always")
            };
        }
    }
}

// Utility methods for dealing with cells.
#[allow(private_bounds)]
impl<T: NumOpsPlus + TryFrom<i8>> Interpreter<T>
where <T as TryFrom<i8>>::Error: Debug {
    /// Mutably gets the cell currently pointed to by the tape pointer.
    /// If the pointer outside of the tape, inserts `T::default` until the array is big enough to be index at that location.
    fn get_mut_cell_or_insert_default(&mut self) -> Result<&mut T, InterpreterError> {
        unsafe {
            if self.tape.len() > self.tape_pointer {
                return Ok(self.tape.get_unchecked_mut(self.tape_pointer));
            }
        }

        // if we get here then the tape is not long enough for our index
        let lenght_limit = self.config.lenght_limit.unwrap_or(usize::MAX);
        if self.tape_pointer+1 > lenght_limit {
            return Err(InterpreterError::TapeLimitExceded { limit: lenght_limit, tried: self.tape_pointer+1 });
        }
        
        let extention = (self.tape_pointer+1) - self.tape.len();
        self.tape.extend(vec![T::default(); extention]);

        unsafe {
            Ok(self.tape.get_unchecked_mut(self.tape_pointer))
        }
    }

    /// Prints a dump of the tape state and pointer state to stdout.
    fn print_dump(&self) {
        println!("{}", "-- TAPE STATE STRING --".yellow().underline().bold());
        for (i, cell) in self.tape.iter().enumerate() {
            let ch = char::from_u32(cell.to_u32().unwrap_or(65_533)).unwrap_or('�');
            let ch_str = if ch == '\n' {
                "\\n".to_string()
            } else if ch.is_control() {
                "�".to_string()
            } else {
                ch.to_string()
            };

            println!("{} {ch_str}", format!("{i}:").to_string().black());
        }
        println!();

        println!("{}", "-- TAPE STATE NUMERIC --".green().underline().bold());
        for (i, cell) in self.tape.iter().enumerate() {
            println!("{} {cell:?}", format!("{i}:").to_string().black());
        }
        println!();

        println!("{}: {}", "LAST VALID TAPE POINTER POSITION".blue().underline().bold(), self.tape_pointer);
    }

    /// Converts a string to a cell value. Behaviour changes based on whether input is number or character.
    /// Returns `None` if `string` could not be parsed into a value.
    fn string_to_value(&self, string: &str) -> Option<T> {
        if self.config.input_as_number {
            if let Ok(v) = string.trim().parse() {
                Some(v)
            } else {
                None
            }
        } else {
            let ch = string.chars().next()?;
            T::from(ch as u32)
        }
    }

    /// Adds items to `Interpreter::input_buffer`.
    /// Does nothing if there the interpreter is configurated to not use bulked input.
    /// Returns `true` if the addition to the buffer was successful.
    fn add_to_input_buffer(&mut self, string: &str) -> bool {
        if !self.config.bulk_input {
            return false;
        }

        let mut values = {
            let elements = if self.config.input_as_number {
                string.trim().split(' ').map(|e| e.to_string()).collect::<Vec<_>>()
            } else {
                // it would be nice if there was a method returning slices to these chars
                string.chars().map(|c| c.to_string()).collect()
            };

            let maybe_values = elements.iter().map(|e| self.string_to_value(&e));
            if maybe_values.clone().find(|e| e.is_none()).is_some() {
                return false;
            }
            let values = maybe_values.filter_map(|e| e).collect::<VecDeque<_>>();

            values
        };

        self.input_buffer.append(&mut values);

        true
    }
}

#[allow(private_bounds)] // these are not meant to be used outside here
impl<T: NumOpsPlus + TryFrom<i8>> Interpreter<T>
where <T as TryFrom<i8>>::Error: Debug {
    #[inline]
    fn increment(&mut self, recurence: i8) -> Result<(), InterpreterError> {
        let recurence_t = T::try_from(recurence)
            .expect("Since recurence is i8 and should be positive, it should be always be able to be stored as T");

        let overflow_behaviour = self.config.overflow_behaviour.clone();

        let value = self.get_mut_cell_or_insert_default()?;
        match overflow_behaviour {
            OverflowBehaviour::Abort => {
                if let Some(nval) = value.checked_add(&recurence_t) {
                    *value = nval;
                } else {
                    return Err(InterpreterError::AbortedDueToOverflow { at: self.tape_pointer })
                }
            },
            OverflowBehaviour::Saturate => *value = value.saturating_add(&recurence_t),
            OverflowBehaviour::Wrap => *value = value.wrapping_add(&recurence_t),
        }

        Ok(())
    }

    #[inline]
    fn decrement(&mut self, recurence: i8) -> Result<(), InterpreterError> {
        let recurence_t = T::try_from(recurence)
            .expect("Since recurence is i8 and should be positive, it should be always be able to be stored as T");

        let overflow_behaviour = self.config.overflow_behaviour.clone();

        let value = self.get_mut_cell_or_insert_default()?;
        match overflow_behaviour {
            OverflowBehaviour::Abort => {
                if let Some(nval) = value.checked_sub(&recurence_t) {
                    *value = nval;
                } else {
                    return Err(InterpreterError::AbortedDueToOverflow { at: self.tape_pointer })
                }
            },
            OverflowBehaviour::Saturate => *value = value.saturating_sub(&recurence_t),
            OverflowBehaviour::Wrap => *value = value.wrapping_sub(&recurence_t),
        }

        Ok(())
    }

    #[inline]
    fn pointer_increment(&mut self, recurence: i8) {
        let recurence_t = recurence as usize;
        self.tape_pointer += recurence_t;
    }

    #[inline]
    fn pointer_decrement(&mut self, recurence: i8) -> Result<(), InterpreterError> {
        let recurence_t = recurence as usize;

        if let Some(npoint) = self.tape_pointer.checked_sub(recurence_t) {
            self.tape_pointer = npoint;
        } else {
            return Err(InterpreterError::TapePointerOob)
        }

        Ok(())
    }

    #[inline]
    fn input(&mut self) -> Result<(), InterpreterError> {
        if let Some(bulk) = self.input_buffer.pop_front() {
            let cell = self.get_mut_cell_or_insert_default()?;
            *cell = bulk; 

            return Ok(())
        }

        // ask until we get a valid response
        let value = loop {
            let Some(string) = ask_bf_input() else {
                continue
            };

            let (value, rest) = if !self.config.input_as_number {
                let (first, rest) = string.split_at(1);
                let value = self.string_to_value(first);
                if let Some(value) = value {
                    (value, rest)
                } else {
                    continue
                }
            } else {
                let (first, rest) = string.split_once(' ').unwrap_or((&string, ""));
                let value = self.string_to_value(first);
                if let Some(value) = value {
                    (value, rest)
                } else {
                    continue
                }
            };

            if !self.add_to_input_buffer(rest) && self.config.bulk_input {
                continue
            }
            
            break value;
        };

        let cell = self.get_mut_cell_or_insert_default()?;
        *cell = value;

        Ok(())
    }

    #[inline]
    fn output(&mut self) -> Result<(), InterpreterError> {
        let value = self.get_mut_cell_or_insert_default()?.clone();
        if self.config.output_as_number {
            print!("{value:?} ");

            #[cfg(test)]
            self.captured_output.push_str(&format!("{value:?} "));
        } else {
            let ch = char::from_u32(value.to_u32().unwrap_or(65_533)).unwrap_or('�');
            print!("{ch}");

            #[cfg(test)]
            self.captured_output.push_str(&format!("{ch}"));
        }

        let _ = io::stdout().flush();

        Ok(())
    }

    #[inline]
    fn left_bracket(&mut self, matching_idx: Option<usize>) -> Result<(), InterpreterError> {
        if *self.get_mut_cell_or_insert_default()? != T::ZERO {
            return Ok(())
        }

        if matching_idx.is_none() {
            return Err(InterpreterError::UnmatchedBracket);
        } else {
            self.instruction_pointer = matching_idx.unwrap();
        }

        Ok(())
    }

    #[inline]
    fn right_bracket(&mut self, matching_idx: Option<usize>) -> Result<(), InterpreterError> {
        if *self.get_mut_cell_or_insert_default()? == T::ZERO {
            return Ok(())
        }

        if matching_idx.is_none() {
            return Err(InterpreterError::UnmatchedBracket);
        } else {
            self.instruction_pointer = matching_idx.unwrap();
        }

        Ok(())
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

    /// Prints a dump of the tape state and pointer state to stdout.
    /// Reflects `Interpreter::print_dump`.
    fn print_dump(&self);

    /// Adds items to `Interpreter::input_buffer`.
    /// Does nothing if there the interpreter is configurated to not use bulked input.
    /// Returns `true` if the addition to the buffer was successful.
    fn add_to_input_buffer(&mut self, string: &str) -> bool;

    #[cfg(test)]
    /// Reflects `Interpreter::captured_output`.
    fn captured_output(&self) -> &str;
}

impl<T> InterpreterTrait for Interpreter<T>
where T: NumOpsPlus + TryFrom<i8>, 
    <T as TryFrom<i8>>::Error: Debug {
    fn advance(&mut self) -> Result<bool, InterpreterError> {
        // thanks clippy i didn't know i could do that, that's... kind of weird syntax, but very useful
        let Some(instruction) = self.instructions.get(self.instruction_pointer) else {
            return Ok(false);
        };

        match instruction {
            ByteCode::Add(n) => self.increment(*n)?,
            ByteCode::Sub(n) => self.decrement(*n)?,
            ByteCode::PointerAdd(n) => self.pointer_increment(*n),
            ByteCode::PointerSub(n) => self.pointer_decrement(*n)?,
            ByteCode::Out => self.output()?,
            ByteCode::In => self.input()?,
            ByteCode::LeftBracket(j) => self.left_bracket(*j)?,
            ByteCode::RightBracket(j) => self.right_bracket(*j)?,
        }
        self.instruction_pointer += 1;

        Ok(true)
    }

    fn tape(&self) -> &dyn Any {
        &self.tape
    }

    fn print_dump(&self) {
        self.print_dump();
    }

    fn add_to_input_buffer(&mut self, string: &str) -> bool {
        self.add_to_input_buffer(string)
    }

    #[cfg(test)]
    fn captured_output(&self) -> &str {
        self.captured_output()
    }
}

/// Builder for [`Interpreter`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct InterpreterBuilder {
    instructions: Vec<ByteCode>,
    inner: InterpreterConfig,
}

impl InterpreterBuilder {
    /// Creates a new [`InterpreterBuilder`].
    pub fn new(instructions: &str) -> Self {
        let instructions = brainfuck_to_bytecode(instructions);

        InterpreterBuilder {
            instructions,
            inner: InterpreterConfig::default(),
        }
    }

    /// Sets the cell type to unsigned integers of 8 bits (`u8`).
    #[must_use]
    pub fn with_u8(mut self) -> Self {
        self.inner.cell_kind = CellKind::U8;
        self
    }

    /// Sets the cell type to unsigned integers of 16 bits (`u16`).
    #[must_use]
    pub fn with_u16(mut self) -> Self {
        self.inner.cell_kind = CellKind::U16;
        self
    }
    
    /// Sets the cell type to unsigned integers of 32 bits (`u32`).
    #[must_use]
    pub fn with_u32(mut self) -> Self {
        self.inner.cell_kind = CellKind::U32;
        self
    }
    
    /// Sets the cell type to signed integers of 8 bits (`u8`).
    #[must_use]
    pub fn with_i8(mut self) -> Self {
        self.inner.cell_kind = CellKind::I8;
        self
    }
    
    /// Sets the cell type to signed integers of 16 bits (`u16`).
    #[must_use]
    pub fn with_i16(mut self) -> Self {
        self.inner.cell_kind = CellKind::I16;
        self
    }
    
    /// Sets the cell type to signed integers of 32 bits (`u32`).
    #[must_use]
    pub fn with_i32(mut self) -> Self {
        self.inner.cell_kind = CellKind::I32;
        self
    }

    /// Sets a limit to the tape lenght of `lenght` cells.
    #[must_use]
    pub fn with_tape_leght(mut self, lenght: usize) -> Self {
        self.inner.lenght_limit = Some(lenght);
        self
    }

    /// Does not set a limit to the tape lenght. It will grow as much as needed.
    #[must_use]
    pub fn without_tape_lenght(mut self) -> Self {
        self.inner.lenght_limit = None;
        self
    }

    /// Sets the overflow behaviour to wrapping. 
    #[must_use]
    pub fn with_wrapping_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Wrap;
        self
    }

    /// Sets the overflow behaviour to saturating.  
    #[must_use]   
    pub fn with_saturating_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Saturate;
        self
    }

    /// Sets the overflow behaviour to aborting. 
    #[must_use]
    pub fn with_aborting_behaviour(mut self) -> Self {
        self.inner.overflow_behaviour = OverflowBehaviour::Abort;
        self
    }

    /// Sets the input mode to treat the input as an integer number.
    #[must_use]
    pub fn with_input_as_number(mut self) -> Self {
        self.inner.input_as_number = true;
        self
    }

    /// Sets the input mode to treat the input as an character.
    #[must_use]
    pub fn with_input_as_character(mut self) -> Self {
        self.inner.input_as_number = false;
        self
    }

    /// Sets the output mode to treat the output as an integer number.
    #[must_use]
    pub fn with_output_as_number(mut self) -> Self {
        self.inner.output_as_number = true;
        self
    }

    /// Sets the output mode to treat the output as an character.
    #[must_use]
    pub fn with_output_as_character(mut self) -> Self {
        self.inner.output_as_number = false;
        self
    }

    /// Sets the input mode to bulk.
    /// This means that an input, for example: "cats are rats"
    /// would take the first character and give that to the tape and then store the rest in a buffer.
    /// If a bf input is requested and the buffer is not empty it will take it from the buffer.
    #[must_use]
    pub fn with_bulk_input(mut self) -> Self {
        self.inner.bulk_input = true;
        self
    }

    /// Sets the input mode to single item.
    #[must_use]
    pub fn without_bulk_input(mut self) -> Self {
        self.inner.bulk_input = false;
        self
    }

    /// Finishes the building process.
    #[must_use]
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

    bulk_input: bool,
    input_as_number: bool,
    output_as_number: bool,
}

/// Clumped operation byte codes.
/// We use i8 since it is the smallest supported cell type.
/// This means that we can cast the value without fearing that it cannot
/// fit within the cell's type whichever it is.
/// The recurence value is should always be above zero, despite the type.
#[derive(Debug, Clone, PartialEq)]
enum ByteCode {
    /// The '>' operator `self.0` times.
    PointerAdd(i8),
    /// The '<' operator `self.0` times.
    PointerSub(i8),
    /// The '+' operator `self.0` times.
    Add(i8),
    /// The '-' operator `self.0` times.
    Sub(i8),
    /// The '[' operator.
    /// Contains the position of the bytecode of the matching ']' (`None` if there is none).
    LeftBracket(Option<usize>),
    /// The ']' operator.
    /// Contains the position of the bytecode of the matching '[' (`None` if there is none).
    RightBracket(Option<usize>),
    /// The ',' operator.
    In,
    /// The '.' operator.
    Out,
}

/// Transforms the slice of brainfuck into byte code.
fn brainfuck_to_bytecode(bf: &str) -> Vec<ByteCode> {
    macro_rules! increment_bytecode_or_stash {
        ($state:ident, $acc:ident, $bc:ident) => {
            if let Some($bc(val)) = $state {
                // we check if we overflow, if we do
                // we simply push the current instruction and
                // create another one to hold what would overflow the last
                if let Some(nval) = val.checked_add(1) {
                    $state = Some($bc(nval));
                } else {
                    $acc.push($bc(val));
                    $state = Some($bc(1));
                }
            } else if let Some(b) = $state {
                $acc.push(b);
                $state = Some($bc(1));
            } else {
                $state = Some($bc(1));
            }
        };
    }

    macro_rules! stash_bytecode {
        ($state:ident, $acc:ident, $bc:expr) => {
            if let Some(b) = $state {
                $state = None;
                
                $acc.push(b);
                $acc.push($bc);
            } else {
                $acc.push($bc);
            }
        };
    }

    let (mut instructions, remaining) = bf.chars().filter(|c| {
        *c == '+' || *c == '-'
        || *c == '<' || *c == '>'
        || *c == ',' || *c == '.'
        || *c == '[' || *c == ']'
    }
    ).fold((Vec::new(), None),|(mut acc, mut state), ch| {
        // transforming the operator string into clumped byte codes to reduce redundent operations
        match ch {
            '+' => {
                use ByteCode::Add as Add;
                increment_bytecode_or_stash!(state, acc, Add);
            },
            '-' => {
                use ByteCode::Sub as Sub;
                increment_bytecode_or_stash!(state, acc, Sub);
            },
            '>' => {
                use ByteCode::PointerAdd as PointerAdd;
                increment_bytecode_or_stash!(state, acc, PointerAdd);
            },
            '<' => {
                use ByteCode::PointerSub as PointerSub;
                increment_bytecode_or_stash!(state, acc, PointerSub);
            },
            '[' => {
                use ByteCode::LeftBracket as LeftBracket;
                stash_bytecode!(state, acc, LeftBracket(None));
            },
            ']' => {
                use ByteCode::RightBracket as RightBracket;
                stash_bytecode!(state, acc, RightBracket(None));
            },
            ',' => {
                use ByteCode::In as In;
                stash_bytecode!(state, acc, In);
            },
            '.' => {
                use ByteCode::Out as Out;
                stash_bytecode!(state, acc, Out);
            },
            _ => unreachable!()
        }

        (acc, state)
    });

    if let Some(b) = remaining {
        instructions.push(b);
    }

    instructions
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
    + Num + ConstOne + ConstZero + NumCast + Default + Debug + Clone + FromStr + 'static {}
impl<T> NumOpsPlus for T 
where T: WrappingAdd + WrappingSub + CheckedAdd + CheckedSub + SaturatingAdd + SaturatingSub
    + Num + ConstOne + ConstZero + NumCast + Default + Debug + Clone + FromStr + 'static {}

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
    /// The program tried to reach the matching bracket of a bracket that does not have a match.
    /// This error only occurs when the bracket tried to jump to its match, not when it is passed over
    /// because the condition is not satisfied.
    #[error("an unmatched braket tried to jump to its matching bracket")]
    UnmatchedBracket,
}

/// Asks for user input in bf
fn ask_bf_input() -> Option<String> {
    print!("\n?: ");
    let _ = io::stdout().flush();
    let mut buf = String::new();
    let _ = std::io::stdin().read_line(&mut buf);

    // remove the extra newlines that comes with entering a line
    if buf.ends_with('\n') {
        let _ = buf.pop();
    }

    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::{source::SourceFile, transpile};

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
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
            .with_u8()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[1], 144);
    }

    #[test]
    fn interpreter_cell_types() {
        // u8
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
            .with_u8()
            .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u8>>().unwrap();
        assert_eq!(tape[1], 144_u8);

        // i8 (should overflow)
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
            .with_i8()
            .with_aborting_behaviour()
            .finish();
        inter.complete().unwrap_err();
        
        // u16
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
        .with_u16()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u16>>().unwrap();
        assert_eq!(tape[1], 144_u16);

        // i16
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
        .with_i16()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<i16>>().unwrap();
        assert_eq!(tape[1], 144_i16);

        // u32
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
        .with_u32()
        .finish();
        inter.complete().unwrap();
        let tape = inter.tape().downcast_ref::<Vec<u32>>().unwrap();
        assert_eq!(tape[1], 144_u32);

        // i32
        let mut inter = InterpreterBuilder::new(include_str!("../test-resources/fib.bf"))
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

    #[test]
    fn interpreter_captured_output_is_accurate() {
        let source = include_str!("../test-resources/custom-conditionals.basm").to_string();
        let sf = SourceFile::from_raw_parts("conditionals.gay".into(), source);
        let conditional = transpile(sf.leak())
            .unwrap();
        let mut inter = InterpreterBuilder::new(&conditional)
            .with_output_as_character()
            .finish();
        inter.complete().unwrap();
        assert_eq!(inter.captured_output(), "right");

        let source = include_str!("../test-resources/fib.basm").to_string();
        let sf = SourceFile::from_raw_parts("fib.gay".into(), source);
        let fib = transpile(sf.leak())
            .unwrap();
        let mut inter = InterpreterBuilder::new(&fib)
            .with_output_as_number()
            .finish();
        inter.complete().unwrap();
        assert_eq!(inter.captured_output(), "1 2 3 5 8 13 21 34 55 89 144 ");
    }

    #[test]
    fn adding_to_buffer() {
        let source = "+[,.]".to_string();

        // numbers
        let mut inter = InterpreterBuilder::new(&source)
            .with_input_as_number()
            .with_output_as_number()
            .with_bulk_input()
            .finish();

        assert!(!inter.add_to_input_buffer("732"));
        assert!(!inter.add_to_input_buffer("openup"));
        assert!(inter.add_to_input_buffer("3 1 2 0"));
        inter.complete().unwrap();
        assert_eq!(inter.captured_output(), "3 1 2 0 ");

        // characters
        let mut inter = InterpreterBuilder::new(&source)
            .with_bulk_input()
            .finish();

        assert!(inter.add_to_input_buffer("732"));
        assert!(inter.add_to_input_buffer("openup\0"));
        inter.complete().unwrap();
        assert_eq!(inter.captured_output().trim(), "732openup\0");
    }
}
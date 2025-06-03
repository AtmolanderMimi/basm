//! The basm compiler.

mod instruction;
use instruction::{InstructionError, MetaInstruction, SendSyncInstruction};
pub use normalized_items::NormalizedScope;
use thiserror::Error;
mod normalized_items;
mod expressions_eval_impl;
mod aliases;
use aliases::Aliases;
pub use aliases::{AliasValue, AliasesTrait};
mod context;
pub use context::{ContextTrait, MainContext, ScopeContext};

use std::fmt::Debug;

use crate::{parser::{Ident, Instruction as ParsedInstruction, LanguageItem, MetaField, ParsedFile}, CompilerError as CompilerErrorTrait, Lint};

/// Compiles a [`ParsedProgram`] into a brainfuck program in string format.
pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
    Compiler::compile(program)
}

/// The heart of the compilation logic. 732
pub struct Compiler {
    /// The program being built.
    program_buffer: String,
    context: MainContext,
}

impl Compiler {
    /// Compiles a [`ParsedFile`] into a string representation of the brainfuck program.
    /// Errors if the program does not contain a main field.
    pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
        let mut compiler = Compiler {
            program_buffer: String::new(),
            context: MainContext::new(),
        };

        if let Some(setup_field) = program.setup_field.clone() {
            let normalized_setup = NormalizedScope::new(setup_field.contents, &mut compiler.context)?;
            normalized_setup.compile(&mut compiler.context, &mut compiler.program_buffer)?;
        }

        for meta in &program.meta_instructions {
            compiler.walk_meta_instruction_declaration(meta)?;
        }

        let Some(main_field) = program.main_field.clone() else {
            return Err(CompilerError::MissingMain)
        };
        let normalized_main = NormalizedScope::new(main_field.contents, &mut compiler.context.build_subscope_context())?;
        normalized_main.compile(&mut compiler.context, &mut compiler.program_buffer)?;

        Ok(compiler.program_buffer)
    }

    /// Evaluates a meta-instruction.
    pub fn walk_meta_instruction_declaration(&mut self, meta: &MetaField) -> Result<(), CompilerError> {
        let meta_ins = MetaInstruction::new(meta.clone());

        if self.context.add_instruction(meta_ins.name(), meta_ins.clone()) {
            return Err(CompilerError::DoubleDeclaration(meta.clone()))
        };

        Ok(())
    }
}

/// An argument passed into an instruction.
#[derive(Debug, Clone)]
pub enum Argument {
    /// An operand passed into an instruction
    Operand(u32),
    /// A scope passed into an instruction
    Scope(NormalizedScope),
    /// A string passed into an instruction
    String(String),
}

impl Argument {
    /// Returns the inner `u32` if self is `Self::Operand`, else panic.
    pub fn unwrap_operand(self) -> u32 {
        if let Argument::Operand(v) = self {
            v
        } else {
            panic!("Failed unwrap into operand.")
        }
    }

    /// Returns the inner [`NormalizedScope`] if self is `Self::Scope`, else panic.
    pub fn unwrap_scope(self) -> NormalizedScope {
        if let Argument::Scope(s) = self {
            s
        } else {
            panic!("Failed unwrap into scope.")
        }
    }

    /// Returns the inner `String` if self is `Self::String`, else panic.
    pub fn unwrap_string(self) -> String {
        if let Argument::String(s) = self {
            s
        } else {
            panic!("Failed unwrap into string.")
        }
    }

    /// Returns `true` if self is `Self::Operand`.
    pub fn is_operand(&self) -> bool {
        matches!(self, Argument::Operand(_))
    }

    /// Returns `true` if self is `Self::Scope`.
    pub fn is_scope(&self) -> bool {
        matches!(self, Argument::Scope(_))
    }

    /// Returns `true` if self is `Self::String`.
    pub fn is_string(&self) -> bool {
        matches!(self, Argument::String(_))
    }
}

/// Error happening during the compilation process.
#[derive(Debug, Clone, Error)]
pub enum CompilerError {
    /// A meta instruction was defined twice,
    /// or a meta instruction had the same name as a built-in one.
    #[error("instruction was already defined")]
    DoubleDeclaration(MetaField),
    /// An error relative to an instruction. See the inner [`InstructionError`].
    #[error("{0}")]
    Instruction(InstructionError, ParsedInstruction),
    /// An alias which was is not defined in scope is present.
    #[error("alias was not defined")]
    AliasNotDefined(Ident),
    /// An instruction which is was not defined, yet or will never be defined is present.
    /// Note that meta-instructions can use another meta-instruction, but only if it was defined higher.
    #[error("instruction is not defined")]
    InstructionNotDefined(Ident),
    /// A program which is compiled, needs a main field definied in a file.
    /// If there is no main field, this error will be thrown.
    #[error("the program is missing a [main] field")]
    MissingMain,
}

impl CompilerErrorTrait for CompilerError {
    fn lint(&self) -> Option<crate::Lint> {
        let slice = match self {
            CompilerError::AliasNotDefined(e) => e.slice(),
            CompilerError::Instruction(ie, instruction) => {
                match ie {
                    InstructionError::ArgumentScopeError(_, e) => return e.lint(),
                    _ => instruction.slice(),
                }
            },
            CompilerError::InstructionNotDefined(i) => i.slice(),
            CompilerError::DoubleDeclaration(f) => f.name.slice(),
            CompilerError::MissingMain => return None,
        };

        Some(Lint::new_error_range(slice.source(), slice.range()).unwrap())
    }

    fn compiler_source(&self) -> Option<&dyn CompilerErrorTrait> {
        match self {
            Self::Instruction(e, _) => e.compiler_source().map(|e| e as &dyn CompilerErrorTrait),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn myexpectedlifetimeisdecreasingveryrapidely() {
        // this test is here just to check if modification to the lifetimes would prevent compiling
        let main = MainContext::new();
        let mut parent = main.build_subscope_context();
        parent.add_alias("a".to_string(), AliasValue::Numeric(32));
        {   
            let mut child = parent.build_subscope_context();
            child.add_alias("b".to_string(), AliasValue::Numeric(64));

            assert!(child.find_numeric_alias("a").is_some());
            assert!(child.find_numeric_alias("b").is_some());
            drop(child);
        }
        assert!(parent.find_numeric_alias("a").is_some());
        assert!(parent.find_numeric_alias("b").is_none());

        parent.add_alias("c".to_string(), AliasValue::Numeric(128));
        assert!(parent.find_numeric_alias("c").is_some())
    }
}
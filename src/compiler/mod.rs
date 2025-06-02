//! The basm compiler.

mod instruction;
use instruction::{built_in, InstructionError, MetaInstruction, SendSyncInstruction};
pub use normalized_items::NormalizedScope;
use thiserror::Error;
mod normalized_items;
mod expressions_eval_impl;
mod aliases;
use aliases::Aliases;
pub use aliases::AliasValue;
use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

use crate::{parser::{Ident, Instruction as ParsedInstruction, LanguageItem, MetaField, ParsedFile}, CompilerError as CompilerErrorTrait, Lint};

/// Compiles a [`ParsedProgram`] into a brainfuck program in string format.
pub fn compile(program: &ParsedFile) -> Result<String, CompilerError> {
    Compiler::compile(program)
}

struct InnerMainContext {
    pointer: u32,
    instructions: HashMap<String, Rc<dyn SendSyncInstruction>>,
}

impl Debug for InnerMainContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerMainContext")
            .field("pointer", &self.pointer)
            .field("instructions", &self.instructions.keys())
            .finish()
    }
}

impl Default for InnerMainContext {
    fn default() -> Self {
        Self { pointer: Default::default(), instructions: built_in() }
    }
}

/// Provides context about the current program's state.
#[derive(Debug, Default)]
pub struct MainContext(Mutex<InnerMainContext>);

impl MainContext {
    /// Builds a new [`MainContext`].
    pub fn new() -> MainContext {
        MainContext::default()
    }

    /// Builds a new blank [`ScopeContext`].
    pub fn build_scope(&self) -> ScopeContext {
        ScopeContext {
            main: self,
            parent: None,
            local_aliases: Aliases::default(),
        }
    }

    /// Gets the current pointer position.
    pub fn pointer(&self) -> u32 {
        self.0.lock().unwrap().pointer
    }

    /// Sets the current pointer to `position`.
    pub fn set_pointer(&self, position: u32) {
        self.0.lock().unwrap().pointer = position;
    }

    /// Adds an instruction to the list of instructions.
    /// Returns true if an instruction of the same name was overwritten.
    pub fn add_instruction(&self, ident: &str, instruction: impl SendSyncInstruction + 'static) -> bool {
        let mut lock = self.0.lock().unwrap();
        lock.instructions.insert(ident.to_string(), Rc::new(instruction)).is_some()
    }

    /// Tries to find a defined instruction with the matching identifier.
    pub fn find_instruction(&self, ident: &str) -> Option<Rc<dyn SendSyncInstruction>> {
        self.0.lock().unwrap().instructions.get(ident).map(|i| {
            Rc::clone(i)
        })
    }
}

/// Provides context about the current scope's state.
#[derive(Debug)]
pub struct ScopeContext<'a> {
    main: &'a MainContext,
    parent: Option<&'a ScopeContext<'a>>,
    local_aliases: Aliases,
}

impl<'a> ScopeContext<'a> {
    /// Creates a new "sub-context" from this [`ScopeContext`].
    /// A sub-context is a context extending this current context.
    /// In this context "extending" means adding new aliases whilst keeping the parent's aliases.
    /// Aliases added to this children context cannot be accessed by it's parent.
    /// ```
    /// use basm::compiler::{MainContext, AliasValue};
    /// 
    /// let mut main = MainContext::new();
    /// let mut parent = main.build_scope();
    /// parent.add_alias("a".to_string(), AliasValue::Numeric(32));
    /// 
    /// let mut child = parent.sub_scope();
    /// child.add_alias("b".to_string(), AliasValue::Numeric(64));
    /// 
    /// assert!(child.find_numeric_alias("a").is_some());
    /// assert!(child.find_numeric_alias("b").is_some());
    /// drop(child);
    /// assert!(parent.find_numeric_alias("a").is_some());
    /// assert!(parent.find_numeric_alias("b").is_none());
    /// ```
    pub fn sub_scope(&'a self) -> ScopeContext<'a> {
        ScopeContext {
            main: self.main,
            parent: Some(self),
            local_aliases: Aliases::default(),
        }
    }

    /// Adds an alias of any type, aliases do not overwrite themselves when they are of different types.
    /// Only this [`ScopeContext`] and those created from this one will be able to see it.
    pub fn add_alias(&mut self, ident: String, value: AliasValue) {
        self.local_aliases.add_alias(ident, value);
    }

    /// Adds an numeric alias, aliases do not overwrite themselves when they are of different types.
    /// Only this [`ScopeContext`] and those created from this one will be able to see it.
    pub fn add_numeric_alias(&mut self, ident: String, value: u32) {
        self.add_alias(ident, AliasValue::Numeric(value));
    }

    /// Adds a scope alias, aliases do not overwrite themselves when they are of different types.
    /// Only this [`ScopeContext`] and those created from this one will be able to see it.
    pub fn add_scope_alias(&mut self, ident: String, value: NormalizedScope) {
        self.add_alias(ident, AliasValue::Scope(value));
    }

    /// Finds the newest alias matching the `ident`.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    pub fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        let local_find = self.local_aliases.find_numeric_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            self.parent.and_then(|p| p.find_numeric_alias(ident))
        } else {
            local_find
        }
    }

    /// Finds the newest alias matching the `ident`.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    pub fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        let local_find = self.local_aliases.find_scope_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            self.parent.and_then(|p| p.find_scope_alias(ident))
        } else {
            local_find
        }
    }
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

        for meta in &program.meta_instructions {
            compiler.walk_meta_instruction_declaration(meta)?;
        }

        let Some(main_field) = program.main_field.clone() else {
            return Err(CompilerError::MissingMain)
        };
        let normalized_main = NormalizedScope::new(main_field.contents, &mut compiler.context.build_scope())?;
        normalized_main.compile(&compiler.context, &mut compiler.program_buffer)?;

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
        let mut parent = main.build_scope();
        parent.add_alias("a".to_string(), AliasValue::Numeric(32));
        {   
            let mut child = parent.sub_scope();
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
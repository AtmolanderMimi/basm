//! The basm compiler.

mod instruction;
use instruction::{built_in, InstructionError, MetaInstruction, SendSyncInstruction};
pub use normalized_items::NormalizedScope;
use thiserror::Error;
mod normalized_items;
use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

use crate::{parser::{Expression, Ident, Instruction as ParsedInstruction, LanguageItem, MetaField, ParsedProgram}, CompilerError as CompilerErrorTrait, Lint};

/// Compiles a [`ParsedProgram`] into a brainfuck program in string format.
pub fn compile(program: &ParsedProgram) -> Result<String, CompilerError> {
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
            local_value_aliases: Vec::new(),
            local_scope_aliases: Vec::new(),
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
    local_value_aliases: Vec<(String, u32)>,
    local_scope_aliases: Vec<(String, NormalizedScope)>,
}

impl<'a> ScopeContext<'a> {
    /// Creates a new "sub-context" from this [`ScopeContext`].
    /// A sub-context is a context extending this current context.
    /// In this context "extending" means adding new aliases whilst keeping the parent's aliases.
    /// Aliases added to this children context cannot be accessed by it's parent.
    /// ```
    /// use basm::compiler::MainContext;
    /// 
    /// let mut main = MainContext::new();
    /// let mut parent = main.build_scope();
    /// parent.add_value_alias("a".to_string(), 32);
    /// 
    /// let mut child = parent.sub_scope();
    /// child.add_value_alias("b".to_string(), 64);
    /// 
    /// assert!(child.find_value_alias("a").is_some());
    /// assert!(child.find_value_alias("b").is_some());
    /// drop(child);
    /// assert!(parent.find_value_alias("a").is_some());
    /// assert!(parent.find_value_alias("b").is_none());
    /// ```
    pub fn sub_scope(&'a self) -> ScopeContext<'a> {
        ScopeContext {
            main: &self.main,
            parent: Some(self),
            local_value_aliases: Vec::new(),
            local_scope_aliases: Vec::new(),
        }
    }

    /// Adds an numeric alias onto the alias stack.
    /// Only this [`ScopeContext`] and those created from this one will be able to see it.
    pub fn add_value_alias(&mut self, ident: String, value: u32) {
        self.local_value_aliases.push((ident, value));
    }

    /// Adds an scope alias onto the alias stack.
    /// Only this [`ScopeContext`] and those created from this one will be able to see it.
    pub fn add_scope_alias(&mut self, ident: String, scope: NormalizedScope) {
        self.local_scope_aliases.push((ident, scope));
    }

    /// Finds the newest alias matching the `ident`.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    pub fn find_value_alias(&self, ident: &str) -> Option<u32> {
        // we reverse so that we can prioritise oldest match
        let local_find = self.local_value_aliases.iter().rev()
            .find(|(a, _)| a == ident)
            .map(|(_, v)| *v);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            // goofy unwrap_or(None), but it works
            self.parent.map(|p| p.find_value_alias(ident)).unwrap_or(None)
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
        // we reverse so that we can prioritise oldest match
        let local_find = self.local_scope_aliases.iter().rev()
            .find(|(a, _)| a == ident)
            .map(|(_, v)| v);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            // goofy unwrap_or(None), but it works
            self.parent.map(|p| p.find_scope_alias(ident)).unwrap_or(None)
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
    /// Compiles a [`ParsedProgram`] into a string representation of the brainfuck program.
    pub fn compile(program: &ParsedProgram) -> Result<String, CompilerError> {
        let mut compiler = Compiler {
            program_buffer: String::new(),
            context: MainContext::new(),
        };

        for meta in &program.meta_instructions {
            compiler.walk_meta_instruction_declaration(meta)?;
        }

        let normalized_main = NormalizedScope::new(program.main_field.contents.clone(), &mut compiler.context.build_scope())?;
        normalized_main.compile(&compiler.context, &mut compiler.program_buffer)?;

        Ok(compiler.program_buffer)
    }

    /// Evaluates a meta-instruction.
    pub fn walk_meta_instruction_declaration(&mut self, meta: &MetaField) -> Result<(), CompilerError> {
        let meta_ins = MetaInstruction::new(meta.clone())?;

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

    /// Returns `true` if self is `Self::Operand`.
    pub fn is_operand(&self) -> bool {
        if let Argument::Operand(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if self is `Self::Scope`.
    pub fn is_scope(&self) -> bool {
        if let Argument::Scope(_) = self {
            true
        } else {
            false
        }
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
    AliasNotDefined(Expression),
    /// An instruction which is was not defined, yet or will never be defined is present.
    /// Note that meta-instructions can use another meta-instruction, but only if it was defined higher.
    #[error("instruction is not defined")]
    InstructionNotDefined(Ident),
}

impl CompilerErrorTrait for CompilerError {
    fn lint(&self) -> Option<crate::Lint> {
        let slice = match self {
            CompilerError::AliasNotDefined(e) => e.slice(),
            CompilerError::Instruction(ie, instruction) => {
                match ie {
                    InstructionError::CouldNotInlineMeta(_, e) => return e.lint(),
                    InstructionError::ArgumentScopeError(_, e) => return e.lint(),
                    _ => instruction.slice(),
                }
            },
            CompilerError::InstructionNotDefined(i) => i.slice(),
            CompilerError::DoubleDeclaration(f) => f.name.slice(),
        };

        Some(Lint::new_error_range(slice.source(), slice.char_range()).unwrap())
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
        parent.add_value_alias("a".to_string(), 32);
        {   
            let mut child = parent.sub_scope();
            child.add_value_alias("b".to_string(), 64);

            assert!(child.find_value_alias("a").is_some());
            assert!(child.find_value_alias("b").is_some());
            drop(child);
        }
        assert!(parent.find_value_alias("a").is_some());
        assert!(parent.find_value_alias("b").is_none());

        parent.add_value_alias("c".to_string(), 128);
        assert!(parent.find_value_alias("c").is_some())
    }
}
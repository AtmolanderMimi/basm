use super::instruction;
use super::SendSyncInstruction;
use super::NormalizedScope;
use super::Aliases;
use super::{AliasValue, AliasesTrait};

use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

pub trait ContextTrait: AliasesTrait {
    /// Returns the [`MainContext`] linked to this context.
    /// This may be the type itself, if the type is [`MainContext`].
    fn main_ctx(&self) -> &MainContext;

    /// Creates a new "sub-context" from this context.
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
    fn build_subscope_context(&self) -> ScopeContext;
}

/// Provides information about the program's context.
pub struct MainContext {
    pointer: u32,
    instructions: HashMap<String, Rc<dyn SendSyncInstruction>>,
    global_aliases: Aliases,
}

impl Debug for MainContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnerMainContext")
            .field("pointer", &self.pointer)
            .field("instructions", &self.instructions.keys())
            .finish()
    }
}

impl Default for MainContext {
    fn default() -> Self {
        Self {
            pointer: Default::default(),
            instructions: instruction::built_in(),
            global_aliases: Aliases::default(),
        }
    }
}

impl AliasesTrait for MainContext {
    fn add_alias(&mut self, ident: String, value: AliasValue) {
        self.global_aliases.add_alias(ident, value)
    }

    fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        self.global_aliases.find_numeric_alias(ident)
    }

    fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        self.global_aliases.find_scope_alias(ident)
    }
}

impl MainContext {
    /// Builds a new [`MainContext`].
    pub fn new() -> MainContext {
        MainContext::default()
    }

    /// Gets the current pointer position.
    pub fn pointer(&self) -> u32 {
        self.pointer
    }

    /// Sets the current pointer to `position`.
    pub fn set_pointer(&mut self, position: u32) {
        self.pointer = position;
    }

    /// Adds an instruction to the list of instructions.
    /// Returns true if an instruction of the same name was overwritten.
    pub fn add_instruction(&mut self, ident: &str, instruction: impl SendSyncInstruction + 'static) -> bool {
        self.instructions.insert(ident.to_string(), Rc::new(instruction)).is_some()
    }

    /// Tries to find a defined instruction with the matching identifier.
    pub fn find_instruction(&mut self, ident: &str) -> Option<Rc<dyn SendSyncInstruction>> {
        self.instructions.get(ident).map(|i| {
            Rc::clone(i)
        })
    }
}

impl ContextTrait for MainContext {
    fn main_ctx(&self) -> &MainContext {
        self
    }

    fn build_subscope_context(&self) -> ScopeContext {
        ScopeContext {
            main: self,
            parent: None,
            local_aliases: Aliases::default(),
        }
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
}

impl<'a> AliasesTrait for ScopeContext<'a> {
    fn add_alias(&mut self, ident: String, value: AliasValue) {
        self.local_aliases.add_alias(ident, value);
    }

    fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        let local_find = self.local_aliases.find_numeric_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            self.parent.and_then(|p| p.find_numeric_alias(ident))
        } else {
            local_find
        }
    }

    fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        let local_find = self.local_aliases.find_scope_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_none() {
            self.parent.and_then(|p| p.find_scope_alias(ident))
        } else {
            local_find
        }
    }
}
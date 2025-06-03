use super::instruction;
use super::SendSyncInstruction;
use super::NormalizedScope;
use super::Aliases;
use super::{AliasValue, AliasesTrait};

use std::{collections::HashMap, fmt::Debug, rc::Rc};

/// Trait abstracting over the logic of aliasing and subscoping of contexts.
pub trait ContextTrait: AliasesTrait {
    /// Returns the [`MainContext`] linked to this context.
    /// This may be the type itself, if the type is [`MainContext`].
    fn main_ctx(&self) -> &MainContext;

    /// Creates a new "sub-context" from this context.
    /// A sub-context is a context extending this current context.
    /// In this context "extending" means adding new aliases whilst keeping the parent's aliases.
    /// Aliases added to this children context cannot be accessed by it's parent.
    /// ```
    /// use basm::compiler::{MainContext, AliasValue, ContextTrait, AliasesTrait};
    /// 
    /// let mut main = MainContext::new();
    /// let mut parent = main.build_subscope_context();
    /// parent.add_alias("a".to_string(), AliasValue::Numeric(32));
    /// 
    /// let mut child = parent.build_subscope_context();
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
            global_aliases: Aliases::new(),
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
    pub fn find_instruction(&self, ident: &str) -> Option<Rc<dyn SendSyncInstruction>> {
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
            local_aliases: Aliases::new(),
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

impl<'a> AliasesTrait for ScopeContext<'a> {
    fn add_alias(&mut self, ident: String, value: AliasValue) {
        self.local_aliases.add_alias(ident, value);
    }

    fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        let local_find = self.local_aliases.find_numeric_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_some() {
            return local_find;
        }

        let child_find = self.parent.and_then(|p| p.find_numeric_alias(ident));
        if child_find.is_some() {
            return child_find;
        }

        self.main.find_numeric_alias(ident)
    }

    fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        let local_find = self.local_aliases.find_scope_alias(ident);

        // if we did not find an alias in the current scope with keep searching down recusively
        if local_find.is_some() {
            return local_find;
        }

        let child_find = self.parent.and_then(|p| p.find_scope_alias(ident));
        if child_find.is_some() {
            return child_find;
        }

        self.main.find_scope_alias(ident)
    }
}

impl<'a> ContextTrait for ScopeContext<'a> {
    fn main_ctx(&self) -> &MainContext {
        self.main
    }

    fn build_subscope_context(&self) -> ScopeContext {
        ScopeContext {
            main: self.main,
            parent: Some(self),
            local_aliases: Aliases::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_aliases() {
        let mut main = MainContext::new();
        main.add_numeric_alias("Vtruth".to_string(), 42);
        main.add_numeric_alias("Vlies".to_string(), 732);
        let subscope1 = main.build_subscope_context();
        assert_eq!(main.find_numeric_alias("Vtruth"), Some(42));
        assert_eq!(main.find_numeric_alias("Vlies"), Some(732));

        assert_eq!(subscope1.find_numeric_alias("Vtruth"), Some(42));

        let mut subscope2 = subscope1.build_subscope_context();
        subscope2.add_numeric_alias("Vtruth".to_string(), 1);
        let subscope3 = subscope2.build_subscope_context();
        assert_eq!(subscope2.find_numeric_alias("Vtruth"), Some(1));
        assert_eq!(subscope2.find_numeric_alias("Vlies"), Some(732));

        assert_eq!(subscope3.find_numeric_alias("Vtruth"), Some(1));
        assert_eq!(subscope3.find_numeric_alias("Vlies"), Some(732));
    }
}

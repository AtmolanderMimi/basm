use std::collections::HashMap;

use super::NormalizedScope;

/// Represents the value of an alias.
#[derive(Debug, Clone)]
pub enum AliasValue {
    /// The alias is a numeric alias.
    Numeric(u32),
    /// The alias is a scope alias.
    Scope(NormalizedScope),
}

/// Represents the aliases contained within a specific context, may they be one local to a scope or global ones.
/// Aliases are overwritten when redefined, this means that alias definitions can shadow one another.
#[derive(Debug, Default, Clone)]
pub struct Aliases {
    value_aliases: HashMap<String, u32>,
    scope_aliases: HashMap<String, NormalizedScope>,
}

impl Aliases {
    /// Creates a new instance of [`Aliases`].
    pub fn new() -> Self {
        Aliases::default()
    }

    /// Adds an alias onto the alias stack.
    /// If this alias overwrites another in the same collection, the return value is that of the old alias value.
    pub fn add_alias(&mut self, ident: String, value: AliasValue) -> Option<AliasValue> {
        match value {
            AliasValue::Numeric(n) => {
                self.value_aliases.insert(ident, n)
                    .map(|n| AliasValue::Numeric(n))
            },
            AliasValue::Scope(s) => {
                self.scope_aliases.insert(ident, s)
                    .map(|s| AliasValue::Scope(s))
            },
        }
    }

    /// Finds the newest alias matching the `ident`.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    pub fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        self.value_aliases.get(ident).cloned()
    }

    /// Finds the newest alias matching the `ident`.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    pub fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        self.scope_aliases.get(ident)
    }
}
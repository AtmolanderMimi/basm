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
}

impl AliasesTrait for Aliases {
    fn add_alias(&mut self, ident: String, value: AliasValue){
        match value {
            AliasValue::Numeric(n) => {
                self.value_aliases.insert(ident, n);
            },
            AliasValue::Scope(s) => {
                self.scope_aliases.insert(ident, s);
            },
        }
    }

    fn find_numeric_alias(&self, ident: &str) -> Option<u32> {
        self.value_aliases.get(ident).cloned()
    }

    fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope> {
        self.scope_aliases.get(ident)
    }
}

/// A trait for types that contain aliases to implement so that we can interface with it's aliases and aliases of sub-collections.
pub trait AliasesTrait {
    /// Adds an alias into the alias collection.
    /// If an alias of the same type and with the same identifier exists, it will be overwritten
    fn add_alias(&mut self, ident: String, value: AliasValue);

    /// Finds the newest numeric alias matching the `ident` in this collection and subcollections.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    fn find_numeric_alias(&self, ident: &str) -> Option<u32>;

    /// Finds the newest scope alias matching the `ident` in this collection and subcollections.
    /// This means that if a `x` was aliased twice only the latest alised `x` will be taken.
    /// (newer aliases shadow older ones)
    /// 
    /// May return `None` if there was no alias defined matching the ident.
    /// This returns `None` even if a numeric alias was already defined earlier,
    /// if it was overshadowed by an alias of other type.
    fn find_scope_alias(&self, ident: &str) -> Option<&NormalizedScope>;

    /// Adds an alias of numeric type into the alias collection.
    /// If an alias of the same type and with the same identifier exists, it will be overwritten
    fn add_numeric_alias(&mut self, ident: String, value: u32) {
        self.add_alias(ident, AliasValue::Numeric(value))
    }

    /// Adds an alias of scope type into the alias collection.
    /// If an alias of the same type and with the same identifier exists, it will be overwritten
    fn add_scope_alias(&mut self, ident: String, value: NormalizedScope) {
        self.add_alias(ident, AliasValue::Scope(value))
    }
}
//! Defines code block literals.

use std::mem;

use crate::compiler::argument::ArgumentType;
use crate::compiler::directive::Directive;
use crate::parser::{Block as ParsedBlock, Directive as ParsedDirective, LanguageItem};

use crate::newtype_wrapper;
newtype_wrapper!(Block, ParsedBlock);

impl Block {
    /// Returns the name of the arguments.
    /// Argument of code blocks are not strictly value typed.
    pub fn argument_names(&self) -> Vec<&str> {
        let Some((argument_part, _)) = &self.0.arguments else {
            return Vec::new();
        };

        argument_part.arguments.iter().map(|(_, _, name)| {
            name.slice_str()
        }).collect()
    }

    /// Returns the type of the arguments, one argument by type.
    /// Argument of code blocks are not strictly value typed.
    pub fn argument_types(&self) -> Vec<ArgumentType> {
        let Some((argument_part, _)) = &self.0.arguments else {
            return Vec::new();
        };

        argument_part.arguments.iter().map(|(_, output, _)| {
            if output.is_some() {
                ArgumentType::Emplacement
            } else {
                ArgumentType::Expression
            }
        }).collect()
    }

    /// Returns directives in the block.
    pub fn directives(&self) -> &[Directive] {
        unsafe { mem::transmute::<&[ParsedDirective], &[Directive]>(&self.0.body.directives) }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Pattern;

use super::*;

    fn block_from_str(string: &str) -> Block {
        ParsedBlock::solve_str(string).unwrap().into()
    }

    #[test]
    fn block_without_argument_list_returns_empty_vec() {
        let block = block_from_str("{}");
        let arguments = block.argument_names();

        assert!(arguments.is_empty());
    }

    #[test]
    fn block_with_empty_argument_list_returns_empty_vec() {
        let block = block_from_str("[] => {}");
        let arguments = block.argument_names();

        assert!(arguments.is_empty());
    }

    #[test]
    fn block_with_expression_argument() {
        let block = block_from_str("[my_arg] => {}");
        let arguments = block.argument_names();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0], "my_arg");
    }

    #[test]
    fn block_with_emplacement_argument() {
        let block = block_from_str("[&my_arg] => {}");
        let arguments = block.argument_types();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0], ArgumentType::Emplacement);
    }

    #[test]
    fn block_with_many_arguments() {
        let block = block_from_str("[&arg1, arg2, arg3, &arg4] => {}");
        let arguments = block.argument_types();

        assert_eq!(arguments.len(), 4);
        assert_eq!(arguments[0], ArgumentType::Emplacement);
        assert_eq!(arguments[1], ArgumentType::Expression);
        assert_eq!(arguments[2], ArgumentType::Expression);
        assert_eq!(arguments[3], ArgumentType::Emplacement);
    }
}

//! Defines code block literals.

use std::mem;

use crate::compiler::argument::{Argument, ArgumentType, NormalizedArgument};
use crate::compiler::directive::{Directive, DirectiveError};
use crate::compiler::scope::Scope;
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

    /// Inlines the block.
    pub fn inline(&self, ctx: &mut Scope, arguments: Vec<(&Argument, NormalizedArgument)>) -> Result<(), DirectiveError> {
        // add the arguments in the scope
        ctx.sub_scope(); // creates a new scope
        for (i, argument_name) in self.argument_names().iter().enumerate() {
            // we can unwrap because blocks have no way to ask for no value
            let value = arguments[i].1.value().unwrap().clone();

            ctx.declare(argument_name.to_string(), value);
        }

        // runs the directives in the block scope.
        for directive in self.directives() {
            directive.inline(ctx)
                .map_err(|err| DirectiveError::InlineBlockError {
                    inner: Box::new(err),
                    block: self.clone(),
                })?;
        }

        // -- sets the output variables
        let output_argument_names = arguments.iter()
            .map(|(_, n)| n)
            .zip(self.argument_names())
            .filter_map(|(no, na)| no.emplacement().map(|_| na));

        // gets the values of the output variables in the scope of the block
        let mut emplacement_values_in_block = Vec::new();
        for argument_name in output_argument_names {
            // we can unwrap because the variable should always exist, we declare it in this function
            let value = ctx.get(argument_name).unwrap();
            emplacement_values_in_block.push(value);
        }

        // sets the values of the output variables to the emplacements in the parent scope
        let output_arguments = arguments.iter()
            .filter_map(|(arg, norm)| norm.emplacement().map(|emp| (arg, emp)));

        ctx.parent_scope();
        for ((argument, emplacement), value) in output_arguments.zip(emplacement_values_in_block) {
            emplacement.set(ctx, value)
                .map_err(|err| DirectiveError::FailedToSet {
                    inner: err,
                    argument: (*argument).clone(),
                })?;
        }

        Ok(())
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

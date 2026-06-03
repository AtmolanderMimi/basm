//! Defines what is a file and how to interact with it logically.

use std::mem;

use crate::compiler::CompilerError;
use crate::compiler::directive::{Directive};
use crate::compiler::scope::Scope;
use crate::newtype_wrapper;

use crate::parser::{ParsedFile, Directive as ParsedDirective};
newtype_wrapper!(File, ParsedFile);

impl File {
    /// Gets the directives in the file.
    pub fn directives(&self) -> &[Directive] {
        unsafe { mem::transmute::<&[ParsedDirective], &[Directive]>(&*self.0.directives) }
    }

    /// Interprets all the directives in the file.
    pub fn interpret_directives(&self, ctx: &mut Scope) -> Result<(), CompilerError> {
        for directive in self.directives() {
            directive.inline(ctx)
                .map_err(|err| CompilerError::DirectiveError { inner: err, directive: directive.clone() })?;
        }

        Ok(())
    }
}

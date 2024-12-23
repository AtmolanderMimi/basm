use std::{fmt::Debug, rc::Rc};

use either::Either;

use crate::{lexer::token::TokenType, parser::{Instruction as ParsedInstruction, LanguageItem as _, Scope as ParsedScope}};

use super::{instruction::{Instruction, SendSyncInstruction}, CompilerError, ScopeContext};

/// An instruction with all arguments normalized.
#[derive(Clone)]
pub struct NormalizedInstruction<'a> {
    from: ParsedInstruction<'a>,
    kind: Rc<dyn SendSyncInstruction>, // not too glad about using dynamic dyspatch
    arguments: Vec<Either<u32, NormalizedScope<'a>>>,
}

impl<'a> Debug for NormalizedInstruction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledInstruction").field("from", &self.from).field("arguments", &self.arguments).finish()
    }
}

impl<'a> NormalizedInstruction<'a> {
    /// Creates a new [`CompiledInstruction`] from a (parsed) [`Instruction`].
    pub fn new(instruction: ParsedInstruction<'a>, ctx: &ScopeContext<'_>) -> Result<NormalizedInstruction<'a>, CompilerError> {
        // -- we normalize the arguments --
        let arguments_impure = instruction.arguments.iter()
        .map(|a| match a {
            Either::Left(e) => {
                if let Some(v) = e.evaluate(ctx) {
                    Ok(Either::Left(v))
                } else {
                    Err(CompilerError::AliasNotDefined(e.clone().into_owned()))
                }
            },
            Either::Right(s) => Ok(Either::Right(NormalizedScope::new(s.clone().into_owned(), ctx)?)),
        });

        // error handling my belobed
        let error = arguments_impure.clone().filter(|i| i.is_err()).next();
        if let Some(Err(error)) = error {
            return Err(error)
        }

        let arguments = arguments_impure.map(|a| a.unwrap())
            .collect();

        // -- kind --
        let instruction_ident = if let TokenType::Ident(i) = &instruction.name.0.t_type {
            i
        } else {
            panic!("Ident should be Ident")
        };
        let kind = if let Some(k) = ctx.main.find_instruction(&instruction_ident) {
            k
        } else {
            return Err(CompilerError::InstructionNotDefined(instruction.name.into_owned()))
        };

        Ok(NormalizedInstruction {
            from: instruction,
            kind,
            arguments,
        })
    }
}

/// Scope with all items normalized.
#[derive(Debug, Clone)]
pub struct NormalizedScope<'a> {
    from: ParsedScope<'a>,
    contents: Vec<Either<NormalizedInstruction<'a>, NormalizedScope<'a>>>,
}

impl<'a> NormalizedScope<'a> {
    pub fn new(scope: ParsedScope<'a>, ctx: &ScopeContext<'_>) -> Result<NormalizedScope<'a>, CompilerError> {
        // -- we normalize the arguments --
        let contents_impure = scope.contents.iter()
        .map(|a| match a {
            Either::Left(i) => {
                if let TokenType::Ident(name) = &i.name.0.t_type {
                    if name == "ALIS" {
                        todo!() // this makes me realise that not treating field conentents as scopes is a huge
                        // duplication of code problem
                    }
                } else {
                    panic!("Should not reach because Ident is Ident")
                }

                let v = NormalizedInstruction::new(i.clone(), ctx)?;
                Ok(Either::Left(v))
            },
            Either::Right(s) => {
                let nctx = ctx.sub_scope();
                let v = NormalizedScope::new(s.clone(), &nctx)?;
                Ok(Either::Right(v))
            },
        });

        // error handling my belobed
        let error = contents_impure.clone().filter(|i| i.is_err()).next();
        if let Some(Err(error)) = error {
            return Err(error)
        }

        let contents = contents_impure.map(|a| a.unwrap())
            .collect();
        
        Ok(NormalizedScope {
            contents,
            from: scope,
        })
    }
}

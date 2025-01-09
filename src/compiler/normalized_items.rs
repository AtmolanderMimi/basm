use std::{fmt::Debug, rc::Rc};

use either::Either;

use crate::{lexer::token::TokenType, parser::{Expression, Instruction as ParsedInstruction, Scope as ParsedScope, ValueRepresentation}};

use super::{instruction::{InstructionError, SendSyncInstruction}, CompilerError, MainContext, ScopeContext};

/// An instruction with all arguments normalized.
#[derive(Clone)]
pub struct NormalizedInstruction {
    pub from: ParsedInstruction,
    kind: Rc<dyn SendSyncInstruction>, // not too glad about using dynamic dyspatch
    arguments: Vec<Either<u32, NormalizedScope>>,
}

impl Debug for NormalizedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledInstruction").field("from", &self.from).field("arguments", &self.arguments).finish()
    }
}

impl NormalizedInstruction {
    /// Creates a new [`CompiledInstruction`] from a (parsed) [`Instruction`].
    pub fn new(instruction: ParsedInstruction, mut ctx: &mut ScopeContext<'_>) -> Result<NormalizedInstruction, CompilerError> {
        // -- we normalize the arguments --
        let arguments_impure = instruction.arguments.iter()
        .map(|a| match a {
            Either::Left(e) => {
                if let Some(v) = e.evaluate(ctx) {
                    Ok(Either::Left(v))
                } else {
                    Err(CompilerError::AliasNotDefined(e.clone()))
                }
            },
            Either::Right(s) => Ok(Either::Right(NormalizedScope::new(s.clone(), &mut ctx)?)),
        });

        // error handling my belobed
        // error handling my belobed
        let arguments_impure =arguments_impure.collect::<Vec<_>>();
        for c in &arguments_impure {
            if let Err(error) = c {
                return Err(error.clone())
            }
        }

        let arguments = arguments_impure.into_iter().map(|a| a.unwrap())
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
            return Err(CompilerError::InstructionNotDefined(instruction.name))
        };

        Ok(NormalizedInstruction {
            from: instruction,
            kind,
            arguments,
        })
    }

    /// Compiles the current instruction into the `buf` in string format.
    pub fn compile(&self, ctx: &MainContext, buf: &mut String) -> Result<(), CompilerError> {
        if let Err(ie) = self.kind.compile_checked(buf, ctx, &self.arguments) {
            Err(CompilerError::Instruction(ie, self.from.clone()))
        } else {
            Ok(())
        }
    }
}

/// Scope with all items normalized.
#[derive(Debug, Clone)]
pub struct NormalizedScope {
    pub from: ParsedScope,
    contents: Vec<Either<NormalizedInstruction, NormalizedScope>>,
}

impl NormalizedScope {
    pub fn new(scope: ParsedScope, ctx: &mut ScopeContext<'_>) -> Result<NormalizedScope, CompilerError> {
        // -- we normalize the arguments --
        let contents_impure = scope.contents.iter()
        .map(|a| match a {
            Either::Left(ins) => {
                if let TokenType::Ident(name) = &ins.name.0.t_type {
                    if name == "ALIS" {
                        alis(ctx, ins.clone())?;
                    }
                } else {
                    panic!("Should not reach because Ident is Ident")
                }

                let v = NormalizedInstruction::new(ins.clone(), ctx)?;
                Ok::<Either<_, _>, CompilerError>(Either::Left(v))
            },
            Either::Right(s) => {
                let mut nctx = ctx.sub_scope();
                let v = NormalizedScope::new(s.clone(), &mut nctx)?;
                Ok(Either::Right(v))
            },
        });

        // error handling my belobed
        let contents_impure = contents_impure.collect::<Vec<_>>();
        for c in &contents_impure {
            if let Err(error) = c {
                return Err(error.clone())
            }
        }

        let contents = contents_impure.into_iter().map(|a| a.unwrap())
            .collect();
        
        Ok(NormalizedScope {
            contents,
            from: scope,
        })
    }

    /// Compiles the current scope into the `buf` in string format.
    pub fn compile(&self, ctx: &MainContext, buf: &mut String) -> Result<(), CompilerError> {
        self.contents.iter().try_for_each(|c| match c {
            Either::Left(i) => i.compile(ctx, buf),
            Either::Right(s) => s.compile(ctx, buf),
        })
    }
}

/// Here's our happy little ALIS implementation.
fn alis(ctx: &mut ScopeContext<'_>, instruction: ParsedInstruction) -> Result<(), CompilerError> {
    if instruction.arguments.len() > 2 {
        let v = InstructionError::TooManyArguments { got: instruction.arguments.len(), expected: 2 };
        return Err(CompilerError::Instruction(v, instruction))
    } else if instruction.arguments.len() < 2 {
        let v = InstructionError::TooFewArguments { got: instruction.arguments.len(), expected: 2 };
        return Err(CompilerError::Instruction(v, instruction))
    }

    let alis_name = if let Either::Left(Expression {
        base: ValueRepresentation::Ident(ident),
        mods,
    }) = &instruction.arguments[0] {
        if !mods.is_empty() {
            let v = InstructionError::MalformedAlis;
            return Err(CompilerError::Instruction(v, instruction))
        }

        if let TokenType::Ident(alis_name) = &ident.0.t_type {
            alis_name
        } else {
            panic!("Ident is Ident invariant, again...")
        }
    } else {
        let v = InstructionError::MalformedAlis;
        return Err(CompilerError::Instruction(v, instruction))
    };

    let alis_value = if let Either::Left(exp) = &instruction.arguments[1] {
        if let Some(v) = exp.evaluate(ctx) {
            v
        } else {
            return Err(CompilerError::AliasNotDefined(exp.clone()))
        }
    } else {
        let v = InstructionError::MalformedAlis;
        return Err(CompilerError::Instruction(v, instruction))
    };

    ctx.add_alias(alis_name.clone(), alis_value);

    Ok(())
}

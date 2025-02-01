use std::{fmt::Debug, rc::Rc};

use either::Either;

use crate::parser::{Expression, Instruction as ParsedInstruction, Scope as ParsedScope, ValueRepresentation, Argument as ParsedArgument};

use super::{instruction::{InstructionError, SendSyncInstruction}, Argument, CompilerError, MainContext, ScopeContext};

/// An instruction with all arguments normalized.
#[derive(Clone)]
pub struct NormalizedInstruction {
    pub from: ParsedInstruction,
    kind: Rc<dyn SendSyncInstruction>, // not too glad about using dynamic dyspatch
    arguments: Vec<Argument>,
}

impl Debug for NormalizedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledInstruction").field("from", &self.from).field("arguments", &self.arguments).finish()
    }
}

impl NormalizedInstruction {
    /// Creates a new [`CompiledInstruction`] from a (parsed) [`Instruction`].
    pub fn new(instruction: ParsedInstruction, mut ctx: &mut ScopeContext<'_>) -> Result<NormalizedInstruction, CompilerError> {
        // we skip normalizing arguments to ALIS
        // since it is the whole point of ALIS to have undefined arguments.
        // (specifically it's scope identifiers that mess it up because we try to 
        // normalize it as a integer value)
        if instruction.name.value() == "ALIS" {
            return Ok(NormalizedInstruction {
                from: instruction,
                kind: ctx.main.find_instruction("ALIS").unwrap(),
                arguments: Vec::new(),
            })
        }

        // -- we normalize the arguments --
        let arguments_impure = instruction.arguments.iter()
        .map(|a| match a {
            ParsedArgument::Expression(ex) => {
                let res = ex.evaluate(ctx);
                match res {
                    Ok(v) => Ok(Argument::Operand(v)),
                    Err(err) => {
                        Err(err)
                    },
                }
            },
            ParsedArgument::Scope(s) => Ok(Argument::Scope(NormalizedScope::new(s.clone(), &mut ctx)?)),
            ParsedArgument::ScopeIdent(i) => {
                match ctx.find_scope_alias(i.ident.value()) {
                    // TODO: remove this clone
                    Some(v) => Ok(Argument::Scope(v.clone())),
                    None => Err(CompilerError::AliasNotDefined(i.ident.clone())),
                }
            }
        });

        // error handling my belobed
        // error handling my belobed
        let arguments_impure = arguments_impure.collect::<Vec<_>>();
        for c in &arguments_impure {
            if let Err(error) = c {
                return Err(error.clone())
            }
        }

        let arguments = arguments_impure.into_iter().map(|a| a.unwrap())
            .collect();

        // -- kind --
        let instruction_ident = instruction.name.value();
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
    #[allow(missing_docs)]
    pub from: ParsedScope,
    contents: Vec<Either<NormalizedInstruction, NormalizedScope>>,
}

impl NormalizedScope {
    /// Tries to normalize a [`ParsedScope`] using `ctx`.
    pub fn new(scope: ParsedScope, ctx: &mut ScopeContext<'_>) -> Result<NormalizedScope, CompilerError> {
        // -- we normalize the arguments --
        let contents_impure = scope.contents.iter()
        .map(|a| match a {
            Either::Left(ins) => {
                if ins.name.value() == "ALIS" {
                    alis(ctx, ins.clone())?;
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
    // check arguments, since this is not run as a normal instruction this is done manually
    if instruction.arguments.len() > 2 {
        let v = InstructionError::TooManyArguments { got: instruction.arguments.len(), expected: 2 };
        return Err(CompilerError::Instruction(v, instruction))
    } else if instruction.arguments.len() < 2 {
        let v = InstructionError::TooFewArguments { got: instruction.arguments.len(), expected: 2 };
        return Err(CompilerError::Instruction(v, instruction))
    }

    // gets the name..
    let alis_name = match &instruction.arguments[0] {
        ParsedArgument::Expression(Expression { base: ValueRepresentation::Ident(ident), mods }) => {
            if !mods.is_empty() {
                let v = InstructionError::MalformedAlis;
                return Err(CompilerError::Instruction(v, instruction))
            }
    
            ident.value()
        },
        _ => return Err(CompilerError::Instruction(InstructionError::MalformedAlis, instruction))
    };
    
    // .. then the value
    match &instruction.arguments[1] {
        ParsedArgument::Expression(exp) => {
            let value = exp.evaluate(ctx)?;
            ctx.add_value_alias(alis_name.to_string(), value);
        },
        ParsedArgument::Scope(scp) => {
            let val = NormalizedScope::new(scp.clone(), ctx)?;
            ctx.add_scope_alias(alis_name.to_string(), val);
        },
        ParsedArgument::ScopeIdent(scpident) => {
            let scp = if let Some(scp) = ctx.find_scope_alias(scpident.ident.value()) {
                scp.clone()
            } else {
                return Err(CompilerError::AliasNotDefined(scpident.ident.clone()))
            };
            ctx.add_scope_alias(alis_name.to_string(), scp);
        }
    };

    Ok(())
}

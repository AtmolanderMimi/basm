//! Declares many objects relative to built-in and meta-instructions.
//! Refer to `syntax-draft` for documentation about built-in instructions.

use std::{collections::HashMap, fmt::{Debug, Write}, rc::Rc};

use either::Either;

use thiserror::Error;

use crate::{lexer::token::TokenType, parser::{LanguageItem, MetaField}};

use super::{normalized_items::NormalizedScope, CompilerError, MainContext};

pub fn built_in() -> HashMap<String, Rc<dyn SendSyncInstruction>> {
    let mut map = HashMap::new();
    map.insert("ALIS", Rc::new(Alis::default()) as Rc<dyn SendSyncInstruction>);


    map.into_iter().map(|(l, b)| (l.to_string(), b)).collect()
}

/// A trait implementing the features of built-in and meta-arguments.
pub trait Instruction {
    /// The number of arguments. Use this as a constant.
    fn arguments(&self) -> &[ArgumentKind];

    /// Compiles the given instruction into string format, checks the validity of the arguments passed in.
    /// Will return an error if the number of arguments does not match the one specified by [`Instruction::arguments`].
    fn compile_checked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]) -> Result<(), InstructionError> {
        if args.len() > self.arguments().len() {
            return Err(InstructionError::TooManyArguments { got: args.len(), expected: self.arguments().len() })
        } else if args.len() < self.arguments().len() {
            return Err(InstructionError::TooFewArguments { got: args.len(), expected: self.arguments().len() })
        }

        // finds non-matching arguments
        let res = self.arguments().into_iter().enumerate().find(|(i, k)| {
            match k {
                ArgumentKind::Operand => args[*i].is_right(),
                ArgumentKind::Scope => args[*i].is_left(),
            }
        });

        if let Some((i, kind)) = res {
            return Err(InstructionError::NonMatchingArgumentKind {
                got: match kind {
                    ArgumentKind::Operand => ArgumentKind::Scope,
                    ArgumentKind::Scope => ArgumentKind::Operand,
                },
                expected: kind.clone(),
                place: i,
            })
        }

        self.compile_checked(buf, ctx, args)
    }
    
    /// Compiles the given instructions into string format.
    /// It can be assummed that the arguments passed in are valid.
    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]);
}

pub trait SendSyncInstruction: Instruction + Send + Sync {}
impl<T: Instruction + Send + Sync> SendSyncInstruction for T {}

/// Defines the expected argument kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentKind {
    /// An operand, e.g a value. (3, my_alis, sp+2)
    Operand,
    /// A scope. "[...]"
    Scope,
}

// TODO: Test the correctness of these implementations

#[derive(Debug, Clone, Default, PartialEq)]
struct Alis;
impl Instruction for Alis {
    fn arguments(&self) -> &[ArgumentKind] {
        &[]
    }

    fn compile_checked(&self, _buf: &mut String, _ctx: &MainContext, _args: &[Either<u32, NormalizedScope<'_>>]) -> Result<(), InstructionError> {
        panic!("ALIS, cannot be compiled like other built-in's, it should be catched before")
    }
    fn compile_unchecked(&self, _buf: &mut String, _ctx: &MainContext, _args: &[Either<u32, NormalizedScope<'_>>]) {
        panic!("ALIS, cannot be compiled like other built-in's, it should be catched before")
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Zero;
impl Instruction for Zero {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]) {
        move_pointer_to(buf, ctx, *args[0].as_ref().unwrap_left());
        buf.write_str("[-]");
    }
}

/// Struct defining a meta-instruction.
/// This is only a tool to make objects that implement [`Instruction`] at runtime.
#[derive(Clone)]
pub struct MetaInstruction<'a> {
    from: MetaField<'a>,
    argument_names: Vec<String>,
    arguments: Vec<ArgumentKind>,
}

impl<'a> MetaInstruction<'a> {
    /// Creates a new [`MetaInstruction`].
    pub fn new(
        meta_field: MetaField<'_>,
    ) -> Result<MetaInstruction, CompilerError> {
        let arguments_iter = meta_field.arguments.iter().map(|i| {
            if let TokenType::Ident(name) = &i.0.t_type {
                name
            } else {
                panic!("Ident is ident")
            }
        });

        // we unwrap it into two vectors so that we can reference a &[ArgumentKind] slice
        // whilst still keeping the order
        let mut argument_names = Vec::new();
        let mut arguments = Vec::new();
        for name in arguments_iter {
            argument_names.push(name.clone());
            arguments.push(ArgumentKind::Operand);
        }

        Ok(MetaInstruction {
            from: meta_field.into_owned(),
            argument_names,
            arguments,
        })
    }

    /// Returns the name of the meta-instruction.
    pub fn name(&self) -> &str {
        if let TokenType::Ident(name) = &self.from.name.0.t_type {
            name
        } else {
            panic!("Ident is ident, it is an invariant")
        }
    }
}

impl<'a> Instruction for MetaInstruction<'a> {
    fn arguments(&self) -> &[ArgumentKind] {
        &self.arguments
    }

    fn compile_checked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]) -> Result<(), InstructionError> {
        if args.len() > self.arguments().len() {
            return Err(InstructionError::TooManyArguments { got: args.len(), expected: self.arguments().len() })
        } else if args.len() < self.arguments().len() {
            return Err(InstructionError::TooFewArguments { got: args.len(), expected: self.arguments().len() })
        }

        // finds non-matching arguments
        let res = self.arguments().into_iter().enumerate().find(|(i, k)| {
            match k {
                ArgumentKind::Operand => args[*i].is_right(),
                ArgumentKind::Scope => args[*i].is_left(),
            }
        });

        if let Some((i, kind)) = res {
            return Err(InstructionError::NonMatchingArgumentKind {
                got: match kind {
                    ArgumentKind::Operand => ArgumentKind::Scope,
                    ArgumentKind::Scope => ArgumentKind::Operand,
                },
                expected: kind.clone(),
                place: i,
            })
        }

        let mut scope_ctx = ctx.build_scope();

        // we don't support scope as arguments to meta-instructions yet, so it is safe to say we can't
        // get it as an argument
        for (i, name) in self.argument_names.iter().enumerate() {
            scope_ctx.add_alias(name.clone(), args[i].clone().unwrap_left());
        }

        let mut scope_ctx = ctx.build_scope();
        let normalized = NormalizedScope::new(self.from.contents.clone(), &mut scope_ctx);

        let res = match normalized {
            Ok(n) => n.compile(ctx, buf),
            Err(e) => return Err(InstructionError::CouldNotInlineMeta(self.from.clone().into_owned(), Box::new(e)))
        };

        if let Err(e) = res {
            Err(InstructionError::CouldNotInlineMeta(self.from.clone().into_owned(), Box::new(e)))
        } else {
            Ok(())
        }
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]) {
        self.compile_checked(buf, ctx, args).unwrap();
    }
}

impl<'a> Debug for MetaInstruction<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetaInstruction").field("arguments", &self.arguments).finish()
    }
}

/// Adds the number of required '>' or '<' to set the pointer to the right position.
fn move_pointer_to(buf: &mut String, ctx: &MainContext, nposition: u32) {
    let opointer = ctx.pointer() as i64;
    let npointer = nposition as i64;
    let delta = npointer - opointer;

    let positive = delta.is_positive();

    for _ in 0..delta.abs() {
        if positive {
            buf.push('>');
        } else {
            buf.push('<');
        }
    }

    ctx.set_pointer(nposition);
}

#[derive(Debug, Clone, Error)]
pub enum InstructionError {
    #[error("too many arguments, expected {expected}, got {got}")]
    TooManyArguments {
        got: usize,
        expected: usize,
    },
    #[error("too few arguments, expected {expected}, got {got}")]
    TooFewArguments {
        got: usize,
        expected: usize,
    },
    #[error("got argument of kind \"{got:?}\", expected \"{expected:?}\"")]
    NonMatchingArgumentKind {
        got: ArgumentKind,
        expected: ArgumentKind,
        place: usize,
    },
    #[error("failed to inline meta-instruction, because {1}")]
    CouldNotInlineMeta(MetaField<'static>, Box<CompilerError>),
    #[error("the alis statement is malformed")]
    MalformedAlis,
}
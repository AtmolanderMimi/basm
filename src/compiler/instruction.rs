//! Declares many objects relative to built-in and meta-instructions.
//! Refer to `syntax-draft` for documentation about built-in instructions.

use std::{collections::HashMap, fmt::{Debug, Write}, rc::Rc};

use either::Either;
// FIXME: BEFORE DOING ANYTHING ELSE, FIGURE OUT HOW TO FIT SCOPES AS ARGUMENTS.
// FIXME: BEFORE DOING ANYTHING ELSE, FIGURE OUT HOW TO FIT SCOPES AS ARGUMENTS.
// FIXME: BEFORE DOING ANYTHING ELSE, FIGURE OUT HOW TO FIT SCOPES AS ARGUMENTS.
// FIXME: BEFORE DOING ANYTHING ELSE, FIGURE OUT HOW TO FIT SCOPES AS ARGUMENTS.
// i think we could do something like doing a pre-pass with the compiler to
// map each parsed instruction and scope into "compiled-instructions" and "compiled-scopes",
// which would simply be instructions (and list of instructions for scopes) with numeral arguments.
// (So we figure out the expressions and aliases before).
// We would be left with something like this:
// CompiledInstruction<'a> {
//      inner: Instruction<'a>
//      ident: String,
//      arguments: Vec<Either<u32, CompiledScope>,
// }
use thiserror::Error;

use crate::parser::{Instruction as ParsedInstruction, Scope as ParsedScope};

use super::{normalized_items::{NormalizedInstruction, NormalizedScope}, CompilerError, MainContext, ScopeContext};

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
#[derive(Clone, Default)]
pub struct MetaInstruction<'a> {
    argument_names: Vec<String>,
    arguments: Vec<ArgumentKind>,
    contents: Vec<Either<ParsedInstruction<'a>, ParsedScope<'a>>>,
}

impl<'a> MetaInstruction<'a> {
    pub fn new(
        arguments: Vec<(String, ArgumentKind)>,
        contents: Vec<Either<ParsedInstruction<'a>,
        ParsedScope<'a>>>
    ) -> Result<MetaInstruction<'a>, CompilerError> {
        let iter = arguments.into_iter();

        // we unwrap it into two vectors so that we can reference a &[ArgumentKind] slice
        // whilst still keeping the order
        let mut argument_names = Vec::new();
        let mut arguments = Vec::new();
        for (n, k) in iter {
            argument_names.push(n);            
            arguments.push(k);
        }

        Ok(MetaInstruction {
            argument_names,
            arguments,
            contents,
        })
    }
}

impl Instruction for MetaInstruction<'_> {
    fn arguments(&self) -> &[ArgumentKind] {
        &self.arguments
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Either<u32, NormalizedScope<'_>>]) {
        let mut scope_ctx = ctx.build_scope();

        // we don't support scope as arguments to meta-instructions yet, so it is safe to say we can't
        // get it as an argument
        for (i, name) in self.argument_names.iter().enumerate() {
            scope_ctx.add_alias(name.clone(), args[i].clone().unwrap_left());
        }

        let normalized = self.contents.iter().map(|c| match c {
            Either::Left(i) => {
                // normalization should have already been done beforehand to check if the meta-instruction is valid
                let v = NormalizedInstruction::new(i.clone(), &scope_ctx).unwrap();
                Either::Left(v)
            },
            Either::Right(s) => {
                let v = NormalizedScope::new(s.clone(), &scope_ctx).unwrap();
                Either::Right(v)
            },
        });

        normalized.for_each(|c| match c {
            Either::Left(i) => i.compile().unwrap(),
            Either::Right(s) => s.compile().unwrap(),
        });
    }
}

impl Debug for MetaInstruction<'_> {
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

#[derive(Debug, Clone, PartialEq, Error)]
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
}
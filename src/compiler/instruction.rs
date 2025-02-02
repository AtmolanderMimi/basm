//! Declares many objects relative to built-in and meta-instructions.
//! Refer to `syntax-draft` for documentation about built-in instructions.

use std::{collections::HashMap, fmt::Debug, rc::Rc};

use thiserror::Error;

use crate::parser::{MetaField, Scope, SignatureArgument};

use super::{normalized_items::NormalizedScope, Argument, CompilerError, MainContext};

pub fn built_in() -> HashMap<String, Rc<dyn SendSyncInstruction>> {
    let mut map = HashMap::new();
    map.insert("ALIS", Rc::new(Alis::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("INLN", Rc::new(Inln::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("RAW" , Rc::new(Raw ::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("BBOX", Rc::new(Bbox::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("ASUM", Rc::new(Asum::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("ZERO", Rc::new(Zero::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("INCR", Rc::new(Incr::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("DECR", Rc::new(Decr::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("COPY", Rc::new(Copy::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("ADDP", Rc::new(Addp::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("SUBP", Rc::new(Subp::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("IN"  , Rc::new(In  ::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("OUT" , Rc::new(Out ::default()) as Rc<dyn SendSyncInstruction>);
    map.insert("WHNE", Rc::new(Whne::default()) as Rc<dyn SendSyncInstruction>);

    map.into_iter().map(|(l, b)| (l.to_string(), b)).collect()
}

/// A trait implementing the features of built-in and meta-arguments.
pub trait Instruction {
    /// The number of arguments. Use this as a constant.
    fn arguments(&self) -> &[ArgumentKind];

    /// Compiles the given instruction into string format, checks the validity of the arguments passed in.
    /// Will return an error if the number of arguments does not match the one specified by [`Instruction::arguments`].
    fn compile_checked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        if args.len() > self.arguments().len() {
            return Err(InstructionError::TooManyArguments { got: args.len(), expected: self.arguments().len() })
        } else if args.len() < self.arguments().len() {
            return Err(InstructionError::TooFewArguments { got: args.len(), expected: self.arguments().len() })
        }

        // finds non-matching arguments
        let res = self.arguments().into_iter().enumerate().find(|(i, expected)| {
            match expected {
                ArgumentKind::Operand => !args[*i].is_operand(),
                ArgumentKind::Scope => !args[*i].is_scope(),
                ArgumentKind::String => !args[*i].is_string(),
            }
        });

        if let Some((i, kind)) = res {
            return Err(InstructionError::NonMatchingArgumentKind {
                got: match args[i] {
                    Argument::Operand(_) => ArgumentKind::Operand,
                    Argument::Scope(_) => ArgumentKind::Scope,
                    Argument::String(_) => ArgumentKind::String,
                },
                expected: kind.clone(),
                place: i,
            })
        }

        self.compile_unchecked(buf, ctx, args)
    }
    
    /// Compiles the given instructions into string format.
    /// It can be assummed that the arguments passed in are valid.
    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError>;
}

pub trait SendSyncInstruction: Instruction + Send + Sync {}
impl<T: Instruction + Send + Sync> SendSyncInstruction for T {}

/// Defines the expected argument kind.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentKind {
    /// An operand, e.g a value. (3, `my_alis`, `sp+2`)
    Operand,
    /// A scope. "\[ ... \]"
    Scope,
    /// A string. "..."
    String,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Alis;
impl Instruction for Alis {
    fn arguments(&self) -> &[ArgumentKind] {
        &[]
    }

    fn compile_checked(&self, _buf: &mut String, _ctx: &MainContext, _args: &[Argument]) -> Result<(), InstructionError> {
        //panic!("ALIS, cannot be compiled like other built-in's, it should be catched before")
        Ok(())
    }
    fn compile_unchecked(&self, _buf: &mut String, _ctx: &MainContext, _args: &[Argument]) -> Result<(), InstructionError> {
        //panic!("ALIS, cannot be compiled like other built-in's, it should be catched before")
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Inln;
impl Instruction for Inln {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Scope]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let scope = args[0].clone().unwrap_scope();
        let res = args[0].clone().unwrap_scope().compile(ctx, buf);
        if let Err(cpm_err) = res {
            return Err(InstructionError::CouldNotInlineScope(scope.from, Box::new(cpm_err)));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Raw;
impl Instruction for Raw {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::String]
    }

    fn compile_unchecked(&self, buf: &mut String, _ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        buf.push_str(&args[0].clone().unwrap_string());

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Bbox;
impl Instruction for Bbox {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        move_pointer_to(buf, ctx, args[0].clone().unwrap_operand());

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Asum;
impl Instruction for Asum {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, _buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        ctx.set_pointer(args[0].clone().unwrap_operand());

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Zero;
impl Instruction for Zero {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        move_pointer_to(buf, ctx, args[0].clone().unwrap_operand());
        buf.push_str("[-]");

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Incr;
impl Instruction for Incr {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos = args[0].clone().unwrap_operand();
        let incrementation = args[1].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos);
        for _ in 0..incrementation {
            buf.push('+');
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Decr;
impl Instruction for Decr {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos = args[0].clone().unwrap_operand();
        let incrementation = args[1].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos);
        for _ in 0..incrementation {
            buf.push('-');
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Copy;
impl Instruction for Copy {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand, ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let origin = args[0].clone().unwrap_operand();
        let pos1 = args[1].clone().unwrap_operand();
        let pos2 = args[2].clone().unwrap_operand();

        move_pointer_to(buf, ctx, origin);
        buf.push_str("[-");

        move_pointer_to(buf, ctx, pos1);
        buf.push('+');
        move_pointer_to(buf, ctx, pos2);
        buf.push('+');

        move_pointer_to(buf, ctx, origin);
        buf.push_str("]");

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Addp;
impl Instruction for Addp {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos1 = args[0].clone().unwrap_operand();
        let pos2 = args[1].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos2);
        buf.push_str("[-");

        move_pointer_to(buf, ctx, pos1);
        buf.push('+');

        move_pointer_to(buf, ctx, pos2);
        buf.push(']');

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Subp;
impl Instruction for Subp {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos1 = args[0].clone().unwrap_operand();
        let pos2 = args[1].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos2);
        buf.push_str("[-");

        move_pointer_to(buf, ctx, pos1);
        buf.push('-');

        move_pointer_to(buf, ctx, pos2);
        buf.push(']');

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Whne;
impl Instruction for Whne {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand, ArgumentKind::Operand, ArgumentKind::Scope]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let variable = args[0].clone().unwrap_operand();
        let compared = args[1].clone().unwrap_operand();
        let scope = args[2].clone().unwrap_scope();

        move_pointer_to(buf, ctx, variable);
        for _ in 0..compared {
            buf.push('-');
        }
        buf.push('[');
        for _ in 0..compared {
            buf.push('+');
        }

        if let Err(e) = scope.compile(ctx, buf) {
            return Err(InstructionError::ArgumentScopeError(scope.from.clone(), Box::new(e)))
        }

        move_pointer_to(buf, ctx, variable);
        for _ in 0..compared {
            buf.push('-');
        }
        buf.push(']');
        for _ in 0..compared {
            buf.push('+');
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct In;
impl Instruction for In {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos1 = args[0].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos1);
        buf.push(',');

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct Out;
impl Instruction for Out {
    fn arguments(&self) -> &[ArgumentKind] {
        &[ArgumentKind::Operand]
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        let pos1 = args[0].clone().unwrap_operand();

        move_pointer_to(buf, ctx, pos1);
        buf.push('.');

        Ok(())
    }
}

/// Struct defining a meta-instruction.
/// This is only a tool to make objects that implement [`Instruction`] at runtime.
#[derive(Clone)]
pub struct MetaInstruction {
    from: MetaField,
    argument_names: Vec<String>,
    arguments: Vec<ArgumentKind>,
}

impl MetaInstruction {
    /// Creates a new [`MetaInstruction`].
    pub fn new(
        meta_field: MetaField,
    ) -> Result<MetaInstruction, CompilerError> {
        let arguments_iter = meta_field.arguments.iter().map(|arg| {
            match arg {
                SignatureArgument::Operand(i) => {
                    let name = i.value();
                    let kind = ArgumentKind::Operand;

                    (name, kind)
                },
                SignatureArgument::Scope(s) => {
                    let name = s.ident.value();
                    let kind = ArgumentKind::Scope;

                    (name, kind)
                }
            }
        });

        // we unwrap it into two vectors so that we can reference a &[ArgumentKind] slice
        // whilst still keeping the order
        let mut argument_names = Vec::new();
        let mut arguments = Vec::new();
        for (name, kind) in arguments_iter {
            argument_names.push(name.to_string());
            arguments.push(kind);
        }

        Ok(MetaInstruction {
            from: meta_field,
            argument_names,
            arguments,
        })
    }

    /// Returns the name of the meta-instruction.
    pub fn name(&self) -> &str {
        self.from.name.value()
    }
}

impl Instruction for MetaInstruction {
    fn arguments(&self) -> &[ArgumentKind] {
        &self.arguments
    }

    fn compile_checked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        if args.len() > self.arguments().len() {
            return Err(InstructionError::TooManyArguments { got: args.len(), expected: self.arguments().len() })
        } else if args.len() < self.arguments().len() {
            return Err(InstructionError::TooFewArguments { got: args.len(), expected: self.arguments().len() })
        }

        // finds non-matching arguments
        let res = self.arguments().into_iter().enumerate().find(|(i, expected)| {
            match expected {
                ArgumentKind::Operand => !args[*i].is_operand(),
                ArgumentKind::Scope => !args[*i].is_scope(),
                ArgumentKind::String => !args[*i].is_string(),
            }
        });

        if let Some((i, kind)) = res {
            return Err(InstructionError::NonMatchingArgumentKind {
                got: match args[i] {
                    Argument::Operand(_) => ArgumentKind::Operand,
                    Argument::Scope(_) => ArgumentKind::Scope,
                    Argument::String(_) => ArgumentKind::String,
                },
                expected: kind.clone(),
                place: i,
            })
        }

        let mut scope_ctx = ctx.build_scope();

        for ((i, name), kind) in self.argument_names.iter().enumerate().zip(self.arguments()) {
            match kind {
                ArgumentKind::Operand => scope_ctx.add_value_alias(name.clone(), args[i].clone().unwrap_operand()),
                ArgumentKind::Scope => scope_ctx.add_scope_alias(name.clone(), args[i].clone().unwrap_scope()),
                kind => panic!("meta-instructions don't support {kind:?}"),
            }
        }

        let normalized = NormalizedScope::new(self.from.contents.clone(), &mut scope_ctx);

        let res = match normalized {
            Ok(n) => n.compile(ctx, buf),
            Err(e) => return Err(InstructionError::CouldNotInlineMeta(self.from.clone(), Box::new(e)))
        };

        if let Err(e) = res {
            Err(InstructionError::CouldNotInlineMeta(self.from.clone(), Box::new(e)))
        } else {
            Ok(())
        }
    }

    fn compile_unchecked(&self, buf: &mut String, ctx: &MainContext, args: &[Argument]) -> Result<(), InstructionError> {
        self.compile_checked(buf, ctx, args)
    }
}

impl Debug for MetaInstruction {
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
    CouldNotInlineMeta(MetaField, Box<CompilerError>),
    #[error("failed to inline scope, because: {1}")]
    CouldNotInlineScope(Scope, Box<CompilerError>),
    #[error("the alis statement is malformed")]
    MalformedAlis,
    #[error("error in argument scope: {1}")]
    ArgumentScopeError(Scope, Box<CompilerError>),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{interpreter::InterpreterBuilder, source::SourceFile, transpile};

    /// Test implementing custom branch instructions like IFNE and IFEQ
    /// and uses it to check wheter 9+10 == 19.
    /// This makes sure scope aliasing and INLN work well.
    #[test]
    fn instruction_scopes_practical_implementation() {
        let file = include_str!("../../test-resources/custom-conditionals.basm");
        let sf = SourceFile::from_raw_parts(PathBuf::new(), file.to_string())
            .leak();
        let bf_code = transpile(sf).unwrap();
        let mut interpreter = InterpreterBuilder::new(&bf_code).finish();
        interpreter.complete().unwrap();

        let tape = interpreter.tape().downcast_ref::<Vec<u8>>()
            .unwrap();

        // cell 2 is the cell telling us if 9+10 is equal to 19
        assert_eq!(tape[1], 1);
    }

    #[test]
    fn inline_simple_correctness() {
        let file = "
        [main] [
        ALIS do_stuff [
            INCR 1 12;
            INCR 1 33;
        ];

        // outside of inln
        INCR 0 12;
        INCR 0 33;

        // inside of inln
        INLN [do_stuff];

        // inside of inln twice
        ADDP 2 1; // moving cell 1 -> 2
        INLN [do_stuff];
        INLN [do_stuff];
        ]
        ";
        let sf = SourceFile::from_raw_parts(PathBuf::new(), file.to_string())
            .leak();
        let bf_code = transpile(sf).unwrap();
        let mut interpreter = InterpreterBuilder::new(&bf_code).finish();
        interpreter.complete().unwrap();

        let tape = interpreter.tape().downcast_ref::<Vec<u8>>()
            .unwrap();

        assert_eq!(tape[0], tape[2]); // cell 0 and 2 both are the same code
        assert_eq!(tape[0]*2, tape[1]); // cell 1 is the value of 0 times two
    }
}
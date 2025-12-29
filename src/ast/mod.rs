//! AST (Abstract Syntax Tree) module for VHP
//!
//! This module contains all the AST node definitions used by the parser
//! and interpreter.

mod expr;
mod ops;
mod stmt;

pub use expr::{Argument, ArrayElement, Expr, MatchArm, PropertyModification};
pub use ops::{AssignOp, BinaryOp, UnaryOp};
pub use stmt::{
    Attribute, AttributeArgument, EnumBackingType, EnumCase, FunctionParam, InterfaceConstant,
    InterfaceMethodSignature, Method, Program, Property, Stmt, SwitchCase, TraitResolution,
    TraitUse, Visibility,
};

//! AST (Abstract Syntax Tree) module for VHP
//!
//! This module contains all the AST node definitions used by the parser
//! and VM.

mod expr;
mod ops;
mod stmt;

pub use expr::{Argument, ArrayElement, Expr, ListElement, MatchArm, PropertyModification};
pub use ops::{AssignOp, BinaryOp, UnaryOp};
pub use stmt::{
    Attribute, AttributeArgument, CatchClause, DeclareDirective, EnumBackingType, EnumCase,
    FunctionParam, GroupUse, InterfaceConstant, InterfaceMethodSignature, Method, NamespaceBody,
    Program, Property, PropertyHook, PropertyHookBody, PropertyHookType, QualifiedName, Stmt,
    SwitchCase, TraitResolution, TraitUse, TypeHint, UseItem, UseType, Visibility,
};

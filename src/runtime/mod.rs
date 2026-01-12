//! Runtime module for VHP
//!
//! This module contains the core value types and built-in functions
//! used by the bytecode VM.

pub mod builtins;
mod value;

pub use value::{ArrayKey, Closure, ClosureBody, GeneratorInstance, ObjectInstance, Value};

/// User-defined function definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // params, body, is_abstract, is_final parsed but not yet used
pub struct UserFunction {
    pub params: Vec<crate::ast::FunctionParam>,
    #[allow(dead_code)]
    pub return_type: Option<crate::ast::TypeHint>,
    pub body: Vec<crate::ast::Stmt>,
    pub is_abstract: bool,
    pub is_final: bool,
    #[allow(dead_code)]
    pub attributes: Vec<crate::ast::Attribute>,
}

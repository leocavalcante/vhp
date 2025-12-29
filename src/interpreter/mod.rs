//! Interpreter module for VHP
//!
//! This module contains the tree-walking interpreter that executes
//! the AST produced by the parser.

mod builtins;
mod value;

// Submodules for organized implementation
mod expr_eval;
mod functions;
mod objects;
mod stmt_exec;

pub use value::{ObjectInstance, Value};

use crate::ast::FunctionParam;
use std::collections::HashMap;
use std::io::{self, Write};

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    pub body: Vec<crate::ast::Stmt>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Class definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    pub readonly: bool, // PHP 8.2+: if true, all properties are implicitly readonly
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub properties: Vec<crate::ast::Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Interface definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    #[allow(dead_code)] // Will be used for interface validation
    pub name: String,
    #[allow(dead_code)] // Will be used for interface inheritance
    pub parents: Vec<String>,
    pub methods: Vec<(String, Vec<FunctionParam>)>, // (name, params)
    #[allow(dead_code)] // Will be used for interface constants
    pub constants: HashMap<String, Value>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Trait definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct TraitDefinition {
    #[allow(dead_code)] // Will be used for trait validation
    pub name: String,
    #[allow(dead_code)] // Will be used for trait composition
    pub uses: Vec<String>,
    pub properties: Vec<crate::ast::Property>,
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Enum definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: crate::ast::EnumBackingType,
    pub cases: Vec<(String, Option<Value>)>, // (case_name, optional_value)
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

pub struct Interpreter<W: Write> {
    output: W,
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
    classes: HashMap<String, ClassDefinition>,
    interfaces: HashMap<String, InterfaceDefinition>,
    traits: HashMap<String, TraitDefinition>,
    enums: HashMap<String, EnumDefinition>,
    current_object: Option<ObjectInstance>,
    current_class: Option<String>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            current_object: None,
            current_class: None,
        }
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

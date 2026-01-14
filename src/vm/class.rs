//! Class, Interface, Trait, and Enum definitions for the VM
//!
//! This module defines the compiled representations of OOP constructs.

use crate::ast::{Attribute, Property, TypeHint, Visibility};
use crate::vm::opcode::CompiledFunction;
use std::collections::HashMap;
use std::sync::Arc;

/// Compiled class definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // name and constants fields not yet used
pub struct CompiledClass {
    pub name: String,
    pub is_abstract: bool,
    pub is_final: bool,
    pub readonly: bool,
    pub parent: Option<String>,
    pub interfaces: Vec<String>,
    pub traits: Vec<String>,
    pub properties: Vec<CompiledProperty>,
    pub methods: HashMap<String, Arc<CompiledFunction>>,
    pub static_methods: HashMap<String, Arc<CompiledFunction>>,
    pub static_properties: HashMap<String, crate::runtime::Value>,
    pub readonly_static_properties: std::collections::HashSet<String>,
    pub constants: HashMap<String, crate::runtime::Value>,
    pub method_visibility: HashMap<String, Visibility>,
    pub method_finals: HashMap<String, bool>,
    pub method_abstracts: HashMap<String, bool>,
    pub attributes: Vec<Attribute>,
}

impl CompiledClass {
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_abstract: false,
            is_final: false,
            readonly: false,
            parent: None,
            interfaces: Vec::new(),
            traits: Vec::new(),
            properties: Vec::new(),
            methods: HashMap::new(),
            static_methods: HashMap::new(),
            static_properties: HashMap::new(),
            readonly_static_properties: std::collections::HashSet::new(),
            constants: HashMap::new(),
            method_visibility: HashMap::new(),
            method_finals: HashMap::new(),
            method_abstracts: HashMap::new(),
            attributes: Vec::new(),
        }
    }

    /// Get a method by name (case-insensitive for magic methods)
    pub fn get_method(&self, name: &str) -> Option<&Arc<CompiledFunction>> {
        self.methods.get(name).or_else(|| {
            // Try case-insensitive for magic methods
            if name.starts_with("__") {
                self.methods
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map(|(_, v)| v)
            } else {
                None
            }
        })
    }
}

/// Compiled property definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // visibility and type_hint fields not yet used
pub struct CompiledProperty {
    pub name: String,
    pub visibility: Visibility,
    pub write_visibility: Option<Visibility>,
    pub default: Option<crate::runtime::Value>,
    pub readonly: bool,
    pub is_static: bool,
    pub type_hint: Option<TypeHint>,
    pub attributes: Vec<Attribute>,
    /// Compiled get hook method name (if any)
    pub get_hook: Option<String>,
    /// Compiled set hook method name (if any)
    pub set_hook: Option<String>,
}

impl CompiledProperty {
    pub fn from_ast(prop: &Property, readonly_class: bool) -> Self {
        // Try to evaluate simple default values at compile time
        let default = prop.default.as_ref().and_then(|expr| {
            use crate::ast::Expr;
            match expr {
                Expr::Integer(n) => Some(crate::runtime::Value::Integer(*n)),
                Expr::Float(n) => Some(crate::runtime::Value::Float(*n)),
                Expr::String(s) => Some(crate::runtime::Value::String(s.clone())),
                Expr::Bool(b) => Some(crate::runtime::Value::Bool(*b)),
                Expr::Null => Some(crate::runtime::Value::Null),
                Expr::Array(elements) if elements.is_empty() => {
                    Some(crate::runtime::Value::Array(Vec::new()))
                }
                _ => None, // Complex expressions need runtime evaluation
            }
        });
        Self {
            name: prop.name.clone(),
            visibility: prop.visibility,
            write_visibility: prop.write_visibility,
            default,
            readonly: prop.readonly || readonly_class,
            is_static: prop.is_static,
            type_hint: None, // Would need to be extracted from AST
            attributes: prop.attributes.clone(),
            get_hook: None, // Will be set by compiler if property has hooks
            set_hook: None,
        }
    }
}

/// Compiled interface definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // name and constants fields not yet used
pub struct CompiledInterface {
    pub name: String,
    pub parents: Vec<String>,
    pub method_signatures: Vec<(String, u8)>, // (name, param_count)
    pub constants: HashMap<String, crate::runtime::Value>,
    pub attributes: Vec<Attribute>,
}

impl CompiledInterface {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parents: Vec::new(),
            method_signatures: Vec::new(),
            constants: HashMap::new(),
            attributes: Vec::new(),
        }
    }
}

/// Compiled trait definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // name field not yet used
pub struct CompiledTrait {
    pub name: String,
    pub uses: Vec<String>,
    pub properties: Vec<CompiledProperty>,
    pub methods: HashMap<String, Arc<CompiledFunction>>,
    pub attributes: Vec<Attribute>,
}

impl CompiledTrait {
    pub fn new(name: String) -> Self {
        Self {
            name,
            uses: Vec::new(),
            properties: Vec::new(),
            methods: HashMap::new(),
            attributes: Vec::new(),
        }
    }
}

/// Compiled enum definition
#[derive(Debug, Clone)]
#[allow(dead_code)] // name and backing_type fields not yet used
pub struct CompiledEnum {
    pub name: String,
    pub backing_type: crate::ast::EnumBackingType,
    pub cases: HashMap<String, Option<crate::runtime::Value>>,
    pub case_order: Vec<String>, // Preserves insertion order for cases() method
    pub methods: HashMap<String, Arc<CompiledFunction>>,
    pub static_methods: HashMap<String, Arc<CompiledFunction>>,
    pub attributes: Vec<Attribute>,
}

impl CompiledEnum {
    pub fn new(name: String, backing_type: crate::ast::EnumBackingType) -> Self {
        Self {
            name,
            backing_type,
            cases: HashMap::new(),
            case_order: Vec::new(),
            methods: HashMap::new(),
            static_methods: HashMap::new(),
            attributes: Vec::new(),
        }
    }
}

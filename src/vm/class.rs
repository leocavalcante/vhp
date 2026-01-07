//! Class, Interface, Trait, and Enum definitions for the VM
//!
//! This module defines the compiled representations of OOP constructs.

use crate::ast::{Attribute, Property, TypeHint, Visibility};
use crate::vm::opcode::CompiledFunction;
use std::collections::HashMap;
use std::sync::Arc;

/// Compiled class definition
#[derive(Debug, Clone)]
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
    pub static_properties: HashMap<String, crate::interpreter::Value>,
    pub constants: HashMap<String, crate::interpreter::Value>,
    pub method_visibility: HashMap<String, Visibility>,
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
            constants: HashMap::new(),
            method_visibility: HashMap::new(),
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
pub struct CompiledProperty {
    pub name: String,
    pub visibility: Visibility,
    pub write_visibility: Option<Visibility>,
    pub default: Option<crate::interpreter::Value>,
    pub readonly: bool,
    pub is_static: bool,
    pub type_hint: Option<TypeHint>,
    pub attributes: Vec<Attribute>,
}

impl CompiledProperty {
    pub fn from_ast(prop: &Property, readonly_class: bool) -> Self {
        Self {
            name: prop.name.clone(),
            visibility: prop.visibility,
            write_visibility: prop.write_visibility,
            default: None, // Will be evaluated at runtime
            readonly: prop.readonly || readonly_class,
            is_static: prop.is_static,
            type_hint: None, // Would need to be extracted from AST
            attributes: prop.attributes.clone(),
        }
    }
}

/// Compiled interface definition
#[derive(Debug, Clone)]
pub struct CompiledInterface {
    pub name: String,
    pub parents: Vec<String>,
    pub method_signatures: Vec<(String, u8)>, // (name, param_count)
    pub constants: HashMap<String, crate::interpreter::Value>,
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
pub struct CompiledEnum {
    pub name: String,
    pub backing_type: crate::ast::EnumBackingType,
    pub cases: HashMap<String, Option<crate::interpreter::Value>>,
    pub methods: HashMap<String, Arc<CompiledFunction>>,
    pub attributes: Vec<Attribute>,
}

impl CompiledEnum {
    pub fn new(name: String, backing_type: crate::ast::EnumBackingType) -> Self {
        Self {
            name,
            backing_type,
            cases: HashMap::new(),
            methods: HashMap::new(),
            attributes: Vec::new(),
        }
    }
}

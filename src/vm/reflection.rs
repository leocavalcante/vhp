//! Reflection functions for the VM
//!
//! This module provides reflection functions that allow inspecting
//! attributes on classes, interfaces, traits, functions, methods, etc.

use crate::ast::Attribute;
use crate::interpreter::{ArrayKey, Value};
use crate::vm::class::{CompiledClass, CompiledInterface, CompiledTrait};
use crate::vm::opcode::CompiledFunction;
use std::sync::Arc;

/// Convert an attribute to a Value (associative array)
fn attribute_to_value(attr: &Attribute) -> Value {
    let mut entries = Vec::new();

    // Add attribute name
    entries.push((
        ArrayKey::String("name".to_string()),
        Value::String(attr.name.clone()),
    ));

    // Convert arguments
    let args: Vec<(ArrayKey, Value)> = attr
        .arguments
        .iter()
        .enumerate()
        .map(|(i, arg)| {
            let mut arg_entries = Vec::new();

            // Add argument name if it's a named argument
            if let Some(name) = &arg.name {
                arg_entries.push((
                    ArrayKey::String("name".to_string()),
                    Value::String(name.clone()),
                ));
            }

            // Convert the argument value expression to a Value
            // For now, we support simple literal expressions
            let value = expr_to_value(&arg.value);
            arg_entries.push((ArrayKey::String("value".to_string()), value));

            (ArrayKey::Integer(i as i64), Value::Array(arg_entries))
        })
        .collect();

    entries.push((
        ArrayKey::String("arguments".to_string()),
        Value::Array(args),
    ));

    Value::Array(entries)
}

/// Convert a simple expression to a Value
/// This only handles literal expressions that can be in attributes
fn expr_to_value(expr: &crate::ast::Expr) -> Value {
    use crate::ast::Expr;
    match expr {
        Expr::Integer(n) => Value::Integer(*n),
        Expr::Float(f) => Value::Float(*f),
        Expr::String(s) => Value::String(s.clone()),
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Null => Value::Null,
        Expr::Array(elements) => {
            let values: Vec<(ArrayKey, Value)> = elements
                .iter()
                .enumerate()
                .map(|(i, elem)| (ArrayKey::Integer(i as i64), expr_to_value(&elem.value)))
                .collect();
            Value::Array(values)
        }
        _ => Value::Null, // Unsupported expressions default to null
    }
}

/// Get attributes for a class
pub fn get_class_attributes(
    class_name: &str,
    classes: &std::collections::HashMap<String, Arc<CompiledClass>>,
) -> Result<Value, String> {
    let class = classes
        .get(class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?;

    let attrs: Vec<(ArrayKey, Value)> = class
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a property
pub fn get_property_attributes(
    class_name: &str,
    property_name: &str,
    classes: &std::collections::HashMap<String, Arc<CompiledClass>>,
) -> Result<Value, String> {
    let class = classes
        .get(class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?;

    let prop = class
        .properties
        .iter()
        .find(|p| p.name == property_name)
        .ok_or_else(|| {
            format!(
                "Property '{}' not found in class '{}'",
                property_name, class_name
            )
        })?;

    let attrs: Vec<(ArrayKey, Value)> = prop
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a method
pub fn get_method_attributes(
    class_name: &str,
    method_name: &str,
    classes: &std::collections::HashMap<String, Arc<CompiledClass>>,
) -> Result<Value, String> {
    let class = classes
        .get(class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?;

    let method = class
        .methods
        .get(method_name)
        .or_else(|| class.static_methods.get(method_name))
        .ok_or_else(|| {
            format!(
                "Method '{}' not found in class '{}'",
                method_name, class_name
            )
        })?;

    let attrs: Vec<(ArrayKey, Value)> = method
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a method parameter
pub fn get_method_parameter_attributes(
    class_name: &str,
    method_name: &str,
    parameter_name: &str,
    classes: &std::collections::HashMap<String, Arc<CompiledClass>>,
) -> Result<Value, String> {
    let class = classes
        .get(class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?;

    let method = class
        .methods
        .get(method_name)
        .or_else(|| class.static_methods.get(method_name))
        .ok_or_else(|| {
            format!(
                "Method '{}' not found in class '{}'",
                method_name, class_name
            )
        })?;

    let param = method
        .parameters
        .iter()
        .find(|p| p.name == parameter_name)
        .ok_or_else(|| {
            format!(
                "Parameter '{}' not found in method '{}'",
                parameter_name, method_name
            )
        })?;

    let attrs: Vec<(ArrayKey, Value)> = param
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a function
pub fn get_function_attributes(
    function_name: &str,
    functions: &std::collections::HashMap<String, Arc<CompiledFunction>>,
) -> Result<Value, String> {
    let func = functions
        .get(function_name)
        .ok_or_else(|| format!("Function '{}' not found", function_name))?;

    let attrs: Vec<(ArrayKey, Value)> = func
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a function parameter
pub fn get_parameter_attributes(
    function_name: &str,
    parameter_name: &str,
    functions: &std::collections::HashMap<String, Arc<CompiledFunction>>,
) -> Result<Value, String> {
    let func = functions
        .get(function_name)
        .ok_or_else(|| format!("Function '{}' not found", function_name))?;

    let param = func
        .parameters
        .iter()
        .find(|p| p.name == parameter_name)
        .ok_or_else(|| {
            format!(
                "Parameter '{}' not found in function '{}'",
                parameter_name, function_name
            )
        })?;

    let attrs: Vec<(ArrayKey, Value)> = param
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for an interface
pub fn get_interface_attributes(
    interface_name: &str,
    interfaces: &std::collections::HashMap<String, Arc<CompiledInterface>>,
) -> Result<Value, String> {
    let interface = interfaces
        .get(interface_name)
        .ok_or_else(|| format!("Interface '{}' not found", interface_name))?;

    let attrs: Vec<(ArrayKey, Value)> = interface
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

/// Get attributes for a trait
pub fn get_trait_attributes(
    trait_name: &str,
    traits: &std::collections::HashMap<String, Arc<CompiledTrait>>,
) -> Result<Value, String> {
    let trait_def = traits
        .get(trait_name)
        .ok_or_else(|| format!("Trait '{}' not found", trait_name))?;

    let attrs: Vec<(ArrayKey, Value)> = trait_def
        .attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| (ArrayKey::Integer(i as i64), attribute_to_value(attr)))
        .collect();
    Ok(Value::Array(attrs))
}

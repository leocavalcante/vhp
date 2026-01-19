//! Additional type and class checking functions

use crate::runtime::{ArrayKey, Value};

/// get_class - Returns the name of the class of an object
pub fn get_class(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("get_class() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Object(obj) => Ok(Value::String(obj.class_name.clone())),
        _ => Ok(Value::String("".to_string())),
    }
}

/// get_parent_class - Returns the name of the parent class of an object or class
pub fn get_parent_class(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("get_parent_class() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Object(obj) => {
            if let Some(parent) = &obj.parent_class {
                Ok(Value::String(parent.clone()))
            } else {
                Ok(Value::String("".to_string()))
            }
        }
        _ => Ok(Value::String("".to_string())),
    }
}

/// get_class_methods - Returns an array of class method names
pub fn get_class_methods(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("get_class_methods() expects at least 1 parameter".to_string());
    }
    let _class_name = match &args[0] {
        Value::String(s) => s.clone(),
        Value::Object(obj) => obj.class_name.clone(),
        _ => return Err("get_class_methods() expects class name or object".to_string()),
    };
    // For now, return empty array - full implementation needs class registry access
    Ok(Value::Array(Vec::new()))
}

/// get_class_vars - Returns an array of class properties
pub fn get_class_vars(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("get_class_vars() expects at least 1 parameter".to_string());
    }
    // For now, return empty array - full implementation needs class registry access
    Ok(Value::Array(Vec::new()))
}

/// get_object_vars - Returns an array of object properties
pub fn get_object_vars(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("get_object_vars() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Object(obj) => {
            let props: Vec<(ArrayKey, Value)> = obj
                .properties
                .iter()
                .map(|(k, v)| (ArrayKey::String(k.clone()), v.clone()))
                .collect();
            Ok(Value::Array(props))
        }
        _ => Ok(Value::Array(Vec::new())),
    }
}

/// method_exists - Checks if a method exists
pub fn method_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("method_exists() expects exactly 2 parameters".to_string());
    }
    // For now, return false - full implementation needs class registry
    Ok(Value::Bool(false))
}

/// property_exists - Checks if a property exists
pub fn property_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("property_exists() expects exactly 2 parameters".to_string());
    }
    match &args[1] {
        Value::Object(obj) => {
            let prop_name = args[1].to_string_val();
            let exists = obj.properties.contains_key(&prop_name);
            Ok(Value::Bool(exists))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// class_exists - Checks if a class has been defined
pub fn class_exists(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("class_exists() expects at least 1 parameter".to_string());
    }
    // For now, return false - full implementation needs class registry
    Ok(Value::Bool(false))
}

/// interface_exists - Checks if an interface has been defined
/// NOTE: Handled by VM in call_reflection_or_builtin, this stub is unused
#[allow(dead_code)]
pub fn interface_exists(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("interface_exists() expects at least 1 parameter".to_string());
    }
    // For now, return false - full implementation needs registry
    Ok(Value::Bool(false))
}

/// trait_exists - Checks if a trait exists
/// NOTE: Handled by VM in call_reflection_or_builtin, this stub is unused
#[allow(dead_code)]
pub fn trait_exists(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("trait_exists() expects at least 1 parameter".to_string());
    }
    // For now, return false - full implementation needs registry
    Ok(Value::Bool(false))
}

/// is_a - Checks if the object is of this class
pub fn is_a(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("is_a() expects at least 2 parameters".to_string());
    }
    match &args[0] {
        Value::Object(obj) => {
            let class_name = args[1].to_string_val();
            let is_same = obj.class_name == class_name;
            let is_subclass = obj
                .parent_class
                .as_ref()
                .map(|p| p == &class_name)
                .unwrap_or(false);
            Ok(Value::Bool(is_same || is_subclass))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// is_subclass_of - Checks if the object has this class as one of its parents
pub fn is_subclass_of(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("is_subclass_of() expects at least 2 parameters".to_string());
    }
    match &args[0] {
        Value::Object(obj) => {
            let class_name = args[1].to_string_val();
            let is_subclass = obj
                .parent_class
                .as_ref()
                .map(|p| p == &class_name)
                .unwrap_or(false);
            Ok(Value::Bool(is_subclass))
        }
        _ => Ok(Value::Bool(false)),
    }
}

/// get_declared_classes - Returns an array of all declared classes
pub fn get_declared_classes(_args: &[Value]) -> Result<Value, String> {
    // For now, return empty array
    Ok(Value::Array(Vec::new()))
}

/// get_declared_interfaces - Returns an array of all declared interfaces
pub fn get_declared_interfaces(_args: &[Value]) -> Result<Value, String> {
    // For now, return empty array
    Ok(Value::Array(Vec::new()))
}

/// get_declared_traits - Returns an array of all declared traits
pub fn get_declared_traits(_args: &[Value]) -> Result<Value, String> {
    // For now, return empty array
    Ok(Value::Array(Vec::new()))
}

/// class_alias - Creates an alias for a class
pub fn class_alias(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("class_alias() expects at least 2 parameters".to_string());
    }
    // For now, return true - full implementation needs registry
    Ok(Value::Bool(true))
}

/// get_defined_functions - Returns an array of all defined functions
pub fn get_defined_functions(_args: &[Value]) -> Result<Value, String> {
    // For now, return empty array
    Ok(Value::Array(Vec::new()))
}

/// func_num_args - Returns the number of arguments passed to the function
pub fn func_num_args(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Integer(0))
}

/// func_get_arg - Returns an argument by index
pub fn func_get_arg(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("func_get_arg() expects at least 1 parameter".to_string());
    }
    Ok(Value::Null)
}

/// func_get_args - Returns an array of arguments
pub fn func_get_args(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Array(Vec::new()))
}

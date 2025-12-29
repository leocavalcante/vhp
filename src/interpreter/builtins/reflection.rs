//! Reflection built-in functions for attributes

use crate::ast::{Attribute, AttributeArgument, Expr};
use crate::interpreter::value::{ArrayKey, Value};

/// Convert an Attribute AST node to a runtime Value (associative array)
/// Format: ["name" => "AttributeName", "arguments" => [...]]
pub fn attribute_to_value(attr: &Attribute) -> Value {
    let result = vec![
        (
            ArrayKey::String("name".to_string()),
            Value::String(attr.name.clone()),
        ),
        (
            ArrayKey::String("arguments".to_string()),
            Value::Array(
                attr.arguments
                    .iter()
                    .enumerate()
                    .map(|(idx, arg)| (ArrayKey::Integer(idx as i64), argument_to_value(arg)))
                    .collect(),
            ),
        ),
    ];

    Value::Array(result)
}

/// Convert an AttributeArgument to a runtime Value (associative array)
/// Format: ["name" => "param_name" or null, "value" => <evaluated_value>]
fn argument_to_value(arg: &AttributeArgument) -> Value {
    let result = vec![
        (
            ArrayKey::String("name".to_string()),
            match &arg.name {
                Some(name) => Value::String(name.clone()),
                None => Value::Null,
            },
        ),
        (
            ArrayKey::String("value".to_string()),
            expr_to_simple_value(&arg.value),
        ),
    ];

    Value::Array(result)
}

/// Convert a simple expression (literals) to a Value
/// For more complex expressions, we'd need full expression evaluation
fn expr_to_simple_value(expr: &Expr) -> Value {
    match expr {
        Expr::Null => Value::Null,
        Expr::Bool(b) => Value::Bool(*b),
        Expr::Integer(n) => Value::Integer(*n),
        Expr::Float(f) => Value::Float(*f),
        Expr::String(s) => Value::String(s.clone()),
        Expr::Array(elements) => {
            // Convert array elements
            let arr = elements
                .iter()
                .enumerate()
                .map(|(idx, elem)| {
                    let key = if let Some(key_expr) = &elem.key {
                        // Has explicit key
                        ArrayKey::from_value(&expr_to_simple_value(key_expr))
                    } else {
                        // Auto-incrementing numeric key
                        ArrayKey::Integer(idx as i64)
                    };
                    let value = expr_to_simple_value(&elem.value);
                    (key, value)
                })
                .collect();
            Value::Array(arr)
        }
        // For other expressions, return a placeholder string
        _ => Value::String("<expression>".to_string()),
    }
}

/// get_class_attributes - Get attributes for a class
/// Usage: get_class_attributes(string $class_name): array
pub fn get_class_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_class_attributes() expects exactly 1 parameter".to_string());
    }

    let class_name = args[0].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let attrs = class_def
            .attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
            .collect();
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_method_attributes - Get attributes for a class method
/// Usage: get_method_attributes(string $class_name, string $method_name): array
pub fn get_method_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_method_attributes() expects exactly 2 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let method_name = args[1].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let method_name_lower = method_name.to_lowercase();
        if let Some(method) = class_def.methods.get(&method_name_lower) {
            let attrs = method
                .attributes
                .iter()
                .enumerate()
                .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
                .collect();
            Ok(Value::Array(attrs))
        } else {
            Err(format!(
                "Method '{}::{}' not found",
                class_name, method_name
            ))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_property_attributes - Get attributes for a class property
/// Usage: get_property_attributes(string $class_name, string $property_name): array
pub fn get_property_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_property_attributes() expects exactly 2 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let property_name = args[1].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        // Find property by name (properties are stored without '$')
        let prop_name = property_name
            .strip_prefix('$')
            .unwrap_or(&property_name)
            .to_string();

        if let Some(prop) = class_def.properties.iter().find(|p| p.name == prop_name) {
            let attrs = prop
                .attributes
                .iter()
                .enumerate()
                .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
                .collect();
            Ok(Value::Array(attrs))
        } else {
            Err(format!(
                "Property '{}::{}' not found",
                class_name, property_name
            ))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_function_attributes - Get attributes for a function
/// Usage: get_function_attributes(string $function_name): array
pub fn get_function_attributes(
    args: &[Value],
    functions: &std::collections::HashMap<String, crate::interpreter::UserFunction>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_function_attributes() expects exactly 1 parameter".to_string());
    }

    let function_name = args[0].to_string_val();
    let function_name_lower = function_name.to_lowercase();

    // Find function by case-insensitive name lookup
    if let Some(func) = functions
        .iter()
        .find(|(k, _)| k.to_lowercase() == function_name_lower)
        .map(|(_, v)| v)
    {
        let attrs = func
            .attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
            .collect();
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Function '{}' not found", function_name))
    }
}

/// get_parameter_attributes - Get attributes for a function parameter
/// Usage: get_parameter_attributes(string $function_name, string $parameter_name): array
pub fn get_parameter_attributes(
    args: &[Value],
    functions: &std::collections::HashMap<String, crate::interpreter::UserFunction>,
) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("get_parameter_attributes() expects exactly 2 parameters".to_string());
    }

    let function_name = args[0].to_string_val();
    let parameter_name = args[1].to_string_val();
    let function_name_lower = function_name.to_lowercase();

    // Find function by case-insensitive name lookup
    if let Some(func) = functions
        .iter()
        .find(|(k, _)| k.to_lowercase() == function_name_lower)
        .map(|(_, v)| v)
    {
        // Find parameter by name (parameters are stored without '$')
        let param_name = parameter_name
            .strip_prefix('$')
            .unwrap_or(&parameter_name)
            .to_string();

        if let Some(param) = func.params.iter().find(|p| p.name == param_name) {
            let attrs = param
                .attributes
                .iter()
                .enumerate()
                .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
                .collect();
            Ok(Value::Array(attrs))
        } else {
            Err(format!(
                "Parameter '{}' in function '{}' not found",
                parameter_name, function_name
            ))
        }
    } else {
        Err(format!("Function '{}' not found", function_name))
    }
}

/// get_method_parameter_attributes - Get attributes for a method parameter
/// Usage: get_method_parameter_attributes(string $class_name, string $method_name, string $parameter_name): array
pub fn get_method_parameter_attributes(
    args: &[Value],
    classes: &std::collections::HashMap<String, crate::interpreter::ClassDefinition>,
) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("get_method_parameter_attributes() expects exactly 3 parameters".to_string());
    }

    let class_name = args[0].to_string_val();
    let method_name = args[1].to_string_val();
    let parameter_name = args[2].to_string_val();
    let class_name_lower = class_name.to_lowercase();

    if let Some(class_def) = classes.get(&class_name_lower) {
        let method_name_lower = method_name.to_lowercase();
        if let Some(method) = class_def.methods.get(&method_name_lower) {
            // Find parameter by name (parameters are stored without '$')
            let param_name = parameter_name
                .strip_prefix('$')
                .unwrap_or(&parameter_name)
                .to_string();

            if let Some(param) = method.params.iter().find(|p| p.name == param_name) {
                let attrs = param
                    .attributes
                    .iter()
                    .enumerate()
                    .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
                    .collect();
                Ok(Value::Array(attrs))
            } else {
                Err(format!(
                    "Parameter '{}' in method '{}::{}' not found",
                    parameter_name, class_name, method_name
                ))
            }
        } else {
            Err(format!(
                "Method '{}::{}' not found",
                class_name, method_name
            ))
        }
    } else {
        Err(format!("Class '{}' not found", class_name))
    }
}

/// get_interface_attributes - Get attributes for an interface
/// Usage: get_interface_attributes(string $interface_name): array
pub fn get_interface_attributes(
    args: &[Value],
    interfaces: &std::collections::HashMap<String, crate::interpreter::InterfaceDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_interface_attributes() expects exactly 1 parameter".to_string());
    }

    let interface_name = args[0].to_string_val();
    let interface_name_lower = interface_name.to_lowercase();

    if let Some(interface_def) = interfaces.get(&interface_name_lower) {
        let attrs = interface_def
            .attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
            .collect();
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Interface '{}' not found", interface_name))
    }
}

/// get_trait_attributes - Get attributes for a trait
/// Usage: get_trait_attributes(string $trait_name): array
pub fn get_trait_attributes(
    args: &[Value],
    traits: &std::collections::HashMap<String, crate::interpreter::TraitDefinition>,
) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("get_trait_attributes() expects exactly 1 parameter".to_string());
    }

    let trait_name = args[0].to_string_val();
    let trait_name_lower = trait_name.to_lowercase();

    if let Some(trait_def) = traits.get(&trait_name_lower) {
        let attrs = trait_def
            .attributes
            .iter()
            .enumerate()
            .map(|(idx, attr)| (ArrayKey::Integer(idx as i64), attribute_to_value(attr)))
            .collect();
        Ok(Value::Array(attrs))
    } else {
        Err(format!("Trait '{}' not found", trait_name))
    }
}

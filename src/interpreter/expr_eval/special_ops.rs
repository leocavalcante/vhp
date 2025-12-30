//! Special/advanced expression operations
//!
//! Handles:
//! - Match expressions (PHP 8.0)
//! - Enum case evaluation
//! - Clone expressions
//! - Clone with property modifications (PHP 8.1)
//! - Pipe operator (PHP 8.5)

use crate::ast::{Expr, MatchArm, PropertyModification};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

/// Evaluate match expression (PHP 8.0)
pub(crate) fn eval_match<W: Write>(
    interpreter: &mut Interpreter<W>,
    expr: &Expr,
    arms: &[MatchArm],
    default: &Option<Box<Expr>>,
) -> Result<Value, String> {
    let match_value = interpreter.eval_expr(expr)?;

    // Try each arm - match uses strict (===) comparison
    for arm in arms {
        // Check if any condition matches (using strict identity comparison)
        for condition in &arm.conditions {
            let cond_value = interpreter.eval_expr(condition)?;
            if match_value.type_equals(&cond_value) {
                return interpreter.eval_expr(&arm.result);
            }
        }
    }

    // No match found, try default
    if let Some(default_expr) = default {
        return interpreter.eval_expr(default_expr);
    }

    // PHP 8 throws UnhandledMatchError if no match and no default
    Err(format!(
        "Unhandled match value: {}",
        match_value.to_output_string()
    ))
}

/// Evaluate enum case
pub(crate) fn eval_enum_case<W: Write>(
    interpreter: &Interpreter<W>,
    enum_name: &str,
    case_name: &str,
) -> Result<Value, String> {
    let enum_name_lower = enum_name.to_lowercase();

    // Look up enum definition
    let enum_def = interpreter
        .enums
        .get(&enum_name_lower)
        .ok_or_else(|| format!("Undefined enum '{}'", enum_name))?;

    // Find the case
    for (name, value) in &enum_def.cases {
        if name == case_name {
            return Ok(Value::EnumCase {
                enum_name: enum_def.name.clone(),
                case_name: name.clone(),
                backing_value: value.as_ref().map(|v| Box::new(v.clone())),
            });
        }
    }

    Err(format!(
        "Undefined case '{}' for enum '{}'",
        case_name, enum_name
    ))
}

/// Evaluate clone expression
pub(crate) fn eval_clone<W: Write>(
    interpreter: &mut Interpreter<W>,
    object_expr: &Expr,
) -> Result<Value, String> {
    let object_value = interpreter.eval_expr(object_expr)?;

    match object_value {
        Value::Object(instance) => {
            // Create a deep clone of the object
            let cloned_instance = crate::interpreter::ObjectInstance {
                class_name: instance.class_name.clone(),
                properties: instance.properties.clone(),
                readonly_properties: instance.readonly_properties.clone(),
                initialized_readonly: std::collections::HashSet::new(), // Reset initialization tracking
                parent_class: instance.parent_class.clone(),
                interfaces: instance.interfaces.clone(),
            };

            // For a cloned object, readonly properties can be re-initialized
            // This is PHP's behavior: clone creates a new object context
            Ok(Value::Object(cloned_instance))
        }
        _ => Err(format!(
            "__clone method called on non-object ({})",
            object_value.get_type()
        )),
    }
}

/// Evaluate clone with property modifications (PHP 8.1)
pub(crate) fn eval_clone_with<W: Write>(
    interpreter: &mut Interpreter<W>,
    object_expr: &Expr,
    modifications: &[PropertyModification],
) -> Result<Value, String> {
    let object_value = interpreter.eval_expr(object_expr)?;

    match object_value {
        Value::Object(instance) => {
            // Create a deep clone of the object
            let mut cloned_instance = crate::interpreter::ObjectInstance {
                class_name: instance.class_name.clone(),
                properties: instance.properties.clone(),
                readonly_properties: instance.readonly_properties.clone(),
                initialized_readonly: std::collections::HashSet::new(), // Reset for clone
                parent_class: instance.parent_class.clone(),
                interfaces: instance.interfaces.clone(),
            };

            // Apply modifications
            for modification in modifications {
                let property_name = &modification.property;

                // Check if property exists in the original object
                if !cloned_instance.properties.contains_key(property_name) {
                    return Err(format!(
                        "Property '{}' does not exist on class '{}'",
                        property_name, cloned_instance.class_name
                    ));
                }

                // Evaluate the new value
                let new_value = interpreter.eval_expr(&modification.value)?;

                // Set the property value
                cloned_instance
                    .properties
                    .insert(property_name.clone(), new_value);

                // Mark readonly property as initialized if it's readonly
                if cloned_instance.readonly_properties.contains(property_name) {
                    cloned_instance
                        .initialized_readonly
                        .insert(property_name.clone());
                }
            }

            Ok(Value::Object(cloned_instance))
        }
        _ => Err(format!(
            "Clone with called on non-object ({})",
            object_value.get_type()
        )),
    }
}

/// Evaluate pipe operator (PHP 8.5)
pub(crate) fn eval_pipe<W: Write>(
    interpreter: &mut Interpreter<W>,
    left: &Expr,
    right: &Expr,
) -> Result<Value, String> {
    // Evaluate the left side to get the value to pipe
    let piped_value = interpreter.eval_expr(left)?;

    // The right side must be a function call or method call
    match right {
        Expr::CallableFromFunction(name) => {
            // First-class callable: $value |> func(...)
            // Call the function with the piped value as the only argument
            interpreter.call_function_with_values(name, &[piped_value])
        }

        Expr::FunctionCall { name, args } => {
            // Find placeholder position
            let placeholder_pos = args
                .iter()
                .position(|arg| matches!(&*arg.value, Expr::Placeholder));

            let mut arg_values = Vec::new();

            if let Some(pos) = placeholder_pos {
                // Placeholder found: insert piped value at that position
                for (i, arg) in args.iter().enumerate() {
                    if i == pos {
                        arg_values.push(piped_value.clone());
                    } else {
                        arg_values.push(interpreter.eval_expr(&arg.value)?);
                    }
                }
            } else {
                // No placeholder: insert piped value as first argument
                arg_values.push(piped_value);
                for arg in args {
                    arg_values.push(interpreter.eval_expr(&arg.value)?);
                }
            }

            // Call the function with the modified argument list
            interpreter.call_function_with_values(name, &arg_values)
        }

        Expr::MethodCall {
            object,
            method,
            args,
        } => {
            let object_value = interpreter.eval_expr(object)?;

            // Evaluate arguments with piped value as first
            let mut arg_values = vec![piped_value];
            for arg in args {
                arg_values.push(interpreter.eval_expr(&arg.value)?);
            }

            match object_value {
                Value::Object(instance) => {
                    let method_lower = method.to_lowercase();

                    // Look up the method in the class definition
                    if let Some(class_def) = interpreter
                        .classes
                        .get(&instance.class_name.to_lowercase())
                        .cloned()
                    {
                        if let Some(method_func) = class_def.methods.get(&method_lower) {
                            // Set current object context
                            let saved_object = interpreter.current_object.clone();
                            let saved_class = interpreter.current_class.clone();
                            interpreter.current_object = Some(instance.clone());
                            interpreter.current_class = Some(class_def.name.clone());

                            // Call the method with piped value as first argument
                            let result = interpreter.call_user_function(method_func, &arg_values);

                            // Restore context
                            interpreter.current_object = saved_object;
                            interpreter.current_class = saved_class;

                            return result;
                        }

                        return Err(format!(
                            "Method '{}' not found on class '{}'",
                            method, class_def.name
                        ));
                    }

                    Err(format!("Class '{}' not found", instance.class_name))
                }
                _ => Err(format!(
                    "Attempting to call method on non-object ({})",
                    object_value.get_type()
                )),
            }
        }

        _ => Err(format!(
            "Pipe operator right-hand side must be a function call or method call, got {:?}",
            right
        )),
    }
}

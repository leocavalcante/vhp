//! Callback helper functions for VM
//!
//! This module provides utilities for calling callbacks from built-in functions.
//! Supports: closures, string function names, and callable arrays [ClassName, method]

use crate::runtime::Value;
use crate::vm::VM;
use std::io::Write;

/// Check if a value is a valid callable callback
pub fn is_callable(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Array(arr) if arr.len() == 2 => {
            let first = &arr[0].1;
            let second = &arr[1].1;
            matches!((first, second), (Value::String(_), Value::String(_)))
        }
        Value::Closure(_) => true,
        _ => false,
    }
}

/// Call a callback with given arguments
///
/// Supports:
/// - String function names: "my_function"
/// - Callable arrays: [ClassName, method]
/// - Closures: fn($x) => $x * 2
///
/// Returns the result of the callback or an error
pub fn call_callback<W: Write>(
    vm: &mut VM<W>,
    callback: &Value,
    args: &[Value],
) -> Result<Value, String> {
    match callback {
        Value::String(func_name) => {
            let normalized = func_name.trim_start_matches('\\').to_string();
            vm.call_function(&normalized, args)
        }
        Value::Array(arr) if arr.len() == 2 => {
            let class = &arr[0].1;
            let method = &arr[1].1;
            match (class, method) {
                (Value::String(class_name), Value::String(method_name)) => {
                    let qualified_name =
                        format!("{}::{}", class_name.trim_start_matches('\\'), method_name);
                    vm.call_function(&qualified_name, args)
                }
                _ => Err("Callable array must contain two strings".to_string()),
            }
        }
        Value::Closure(closure) => vm.call_closure(closure, args),
        _ => Err("Invalid callback type".to_string()),
    }
}

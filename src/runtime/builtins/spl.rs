//! SPL (Standard PHP Library) autoload functions
//!
//! This module provides autoloading support for PHP classes.
//! When a class is not found, registered autoloader functions are called
//! to attempt to load the class definition.

#![allow(dead_code)]

use crate::runtime::{ArrayKey, Closure, Value};
use crate::vm::VM;
use std::io::Write;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref AUTOLOADERS: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    pub static ref INCLUDE_PATH: Mutex<Vec<String>> = Mutex::new(vec![".".to_string()]);
    pub static ref PSR4_REGISTRY: Arc<Mutex<Vec<(String, String)>>> =
        Arc::new(Mutex::new(Vec::new()));
}

/// Normalize class name (remove leading backslash)
pub fn normalize_class_name(name: &str) -> String {
    if let Some(stripped) = name.strip_prefix('\\') {
        stripped.to_string()
    } else {
        name.to_string()
    }
}

/// spl_autoload_register - Register a function as __autoload() implementation
///
/// Register a given function as implementation of __autoload(). This function
/// will be called when PHP tries to use a class that hasn't been defined yet.
pub fn spl_autoload_register(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("spl_autoload_register() expects at least 1 parameter".to_string());
    }

    let callback = &args[0];

    if !is_callable(callback) {
        return Err("spl_autoload_register() expects a valid callback".to_string());
    }

    let mut autoloaders = AUTOLOADERS.lock().unwrap();
    autoloaders.push(callback.clone());
    drop(autoloaders);

    Ok(Value::Bool(true))
}

/// spl_autoload_unregister - Unregister an autoloader function
///
/// Unregister a function that was registered with spl_autoload_register().
pub fn spl_autoload_unregister(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("spl_autoload_unregister() expects exactly 1 parameter".to_string());
    }

    let callback = &args[0];

    if !is_callable(callback) {
        return Err("spl_autoload_unregister() expects a valid callback".to_string());
    }

    let mut autoloaders = AUTOLOADERS.lock().unwrap();
    let pos = autoloaders
        .iter()
        .position(|c| callbacks_equal(c, callback));

    if let Some(index) = pos {
        autoloaders.remove(index);
        drop(autoloaders);
        Ok(Value::Bool(true))
    } else {
        drop(autoloaders);
        Ok(Value::Bool(false))
    }
}

/// spl_autoload_functions - Return all registered autoload functions
///
/// Returns an array of all registered autoload functions.
pub fn spl_autoload_functions(args: &[Value]) -> Result<Value, String> {
    let _args = args;
    let autoloaders = AUTOLOADERS.lock().unwrap();
    let result: Vec<Value> = autoloaders.iter().cloned().collect();
    drop(autoloaders);
    Ok(Value::Array(
        result
            .into_iter()
            .enumerate()
            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
            .collect(),
    ))
}

/// spl_autoload_call - Try to load a class by calling all registered autoloaders
///
/// This function is called internally when a class is not found.
/// It attempts to load the class by calling each registered autoloader.
pub fn spl_autoload_call<W: Write>(vm: &mut VM<W>, class_name: &str) -> bool {
    let autoloaders = AUTOLOADERS.lock().unwrap();
    let autoloaders_copy: Vec<Value> = autoloaders.iter().cloned().collect();
    drop(autoloaders);

    let normalized_name = normalize_class_name(class_name);

    for autoloader in autoloaders_copy.iter() {
        if call_autoloader(vm, autoloader, &normalized_name) {
            return true;
        }
    }

    false
}

/// Call an autoloader callback with the class name
fn call_autoloader<W: Write>(vm: &mut VM<W>, callback: &Value, class_name: &str) -> bool {
    match callback {
        Value::String(func_name) => {
            let name = class_name.to_string();
            let result = call_named_function(vm, func_name, &[Value::String(name)]);
            result.is_ok()
        }
        Value::Array(arr) if arr.len() == 2 => {
            let first = &arr[0].1;
            let second = &arr[1].1;
            if let (Value::String(class_or_obj), Value::String(method)) = (first, second) {
                let name = class_name.to_string();
                let result = call_static_method(vm, class_or_obj, method, &[Value::String(name)]);
                result.is_ok()
            } else {
                false
            }
        }
        Value::Closure(closure) => {
            let name = class_name.to_string();
            let result = call_closure(vm, closure, &[Value::String(name)]);
            result.is_ok()
        }
        _ => false,
    }
}

/// Check if a value is callable
fn is_callable(value: &Value) -> bool {
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

/// Compare two callbacks for equality
fn callbacks_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Array(arr1), Value::Array(arr2)) => {
            arr1.len() == 2 && arr2.len() == 2 && arr1[0].1 == arr2[0].1 && arr1[1].1 == arr2[1].1
        }
        _ => false,
    }
}

/// Call a named function with the given arguments
fn call_named_function<W: Write>(
    vm: &mut VM<W>,
    name: &str,
    args: &[Value],
) -> Result<Value, String> {
    vm.call_function(name, args)
}

/// Call a static method with the given arguments
fn call_static_method<W: Write>(
    vm: &mut VM<W>,
    class: &str,
    method: &str,
    args: &[Value],
) -> Result<Value, String> {
    let method_name = format!("{}::{}", class.trim_start_matches('\\'), method);
    vm.call_function(&method_name, args)
}

/// Call a closure with the given arguments
fn call_closure<W: Write>(
    vm: &mut VM<W>,
    closure: &Closure,
    args: &[Value],
) -> Result<Value, String> {
    vm.call_closure(closure, args)
}

/// set_include_path - Set the include_path configuration option
pub fn set_include_path(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("set_include_path() expects exactly 1 parameter".to_string());
    }

    let path = args[0].to_string_val();
    let mut include_path_guard = INCLUDE_PATH.lock().unwrap();
    let old_path = include_path_guard.join(":");
    *include_path_guard = if path.is_empty() {
        Vec::new()
    } else {
        path.split(':').map(|s| s.to_string()).collect()
    };
    drop(include_path_guard);
    Ok(Value::String(old_path))
}

/// get_include_path - Get the current include_path
pub fn get_include_path(args: &[Value]) -> Result<Value, String> {
    let _args = args;
    let include_path_guard = INCLUDE_PATH.lock().unwrap();
    let paths = include_path_guard.clone();
    drop(include_path_guard);
    Ok(Value::String(paths.join(":")))
}

/// Check if autoloaders are registered
pub fn has_autoloaders() -> bool {
    let autoloaders = AUTOLOADERS.lock().unwrap();
    !autoloaders.is_empty()
}

/// Get the include path as a vector
pub fn get_include_path_vec() -> Vec<String> {
    let include_path_guard = INCLUDE_PATH.lock().unwrap();
    let paths = include_path_guard.clone();
    drop(include_path_guard);
    if paths.is_empty() {
        vec![".".to_string()]
    } else {
        paths
    }
}

/// Clear all autoloaders (useful for testing)
pub fn clear_autoloaders() {
    let mut autoloaders = AUTOLOADERS.lock().unwrap();
    autoloaders.clear();
}

/// PSR-4 Autoloading Functions
/// PSR-4 defines a standard for autoloading classes based on namespace prefixes.
///
/// Register a PSR-4 namespace prefix to directory mapping
///
/// Registers a mapping from a namespace prefix to a base directory.
/// When a class is being resolved, the autoloader will check if the class
/// matches any registered prefix and attempt to load it from the corresponding
/// directory.
///
/// # Arguments
/// * `prefix` - The namespace prefix (e.g., "MyApp\\" or "MyApp\\Models\\")
/// * `base_dir` - The base directory for this namespace prefix
///
/// # Example
/// ```php
/// // Register PSR-4 autoloader
/// spl_autoload_register_psr4('MyApp\\', __DIR__ . '/src/');
///
/// // Now MyApp\Models\User maps to /src/Models/User.php
/// $user = new MyApp\Models\User();
/// ```
pub fn spl_autoload_register_psr4(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("spl_autoload_register_psr4() expects at least 2 parameters".to_string());
    }

    let prefix = args[0].to_string_val();
    let base_dir = args[1].to_string_val();

    if prefix.is_empty() {
        return Err("spl_autoload_register_psr4(): namespace prefix cannot be empty".to_string());
    }

    // Ensure prefix ends with backslash for consistent matching
    let normalized_prefix = if !prefix.ends_with('\\') {
        format!("{}\\", prefix)
    } else {
        prefix
    };

    let mut registry = PSR4_REGISTRY.lock().unwrap();
    registry.push((normalized_prefix, base_dir));
    drop(registry);

    Ok(Value::Bool(true))
}

/// Find the PSR-4 mapping for a given class name
///
/// Searches through registered PSR-4 mappings and returns the matching
/// prefix and base directory for the given class name.
///
/// # Arguments
/// * `class_name` - The fully qualified class name (e.g., "MyApp\\Models\\User")
///
/// # Returns
/// Some((prefix, base_dir)) if a matching prefix is found, None otherwise
pub fn find_psr4_mapping(class_name: &str) -> Option<(String, String)> {
    let registry = PSR4_REGISTRY.lock().unwrap();
    let mut best_match: Option<(String, String)> = None;
    let mut best_prefix_len = 0;

    for (prefix, base_dir) in registry.iter() {
        if class_name.starts_with(prefix) {
            let prefix_len = prefix.len();
            if prefix_len > best_prefix_len {
                best_prefix_len = prefix_len;
                best_match = Some((prefix.clone(), base_dir.clone()));
            }
        }
    }

    best_match
}

/// Convert a class name to a file path using PSR-4 mapping
///
/// Given a class name, finds the longest matching PSR-4 prefix and converts
/// the class name to a file path relative to the base directory.
///
/// # Arguments
/// * `class_name` - The fully qualified class name
/// * `base_dir` - The base directory for the matching prefix
/// * `prefix` - The matched namespace prefix
///
/// # Returns
/// The file path where the class file should be located
pub fn namespace_to_path(class_name: &str, base_dir: &str, prefix: &str) -> String {
    // Remove the namespace prefix to get the relative class name
    let relative_class = if let Some(stripped) = class_name.strip_prefix(prefix) {
        stripped
    } else {
        class_name
    };

    // Replace namespace separators with directory separators
    let path = relative_class.replace('\\', "/");

    // Build the full path
    format!("{}/{}.php", base_dir.trim_end_matches('/'), path)
}

/// Get all registered PSR-4 mappings
///
/// Returns an array of all registered PSR-4 namespace prefix mappings.
pub fn spl_autoload_registered_psr4(args: &[Value]) -> Result<Value, String> {
    let _args = args;
    let registry = PSR4_REGISTRY.lock().unwrap();
    let result: Vec<Value> = registry
        .iter()
        .map(|(prefix, base_dir)| {
            Value::Array(vec![
                (
                    ArrayKey::String("prefix".to_string()),
                    Value::String(prefix.clone()),
                ),
                (
                    ArrayKey::String("path".to_string()),
                    Value::String(base_dir.clone()),
                ),
            ])
        })
        .collect();
    drop(registry);

    Ok(Value::Array(
        result
            .into_iter()
            .enumerate()
            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
            .collect(),
    ))
}

/// Clear all PSR-4 registrations (useful for testing)
pub fn clear_psr4_registry() {
    let mut registry = PSR4_REGISTRY.lock().unwrap();
    registry.clear();
}

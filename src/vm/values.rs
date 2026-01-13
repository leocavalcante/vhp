//! Value operations and type checking for VM
//!
//! This module contains functions for:
//! - Arithmetic operations on values
//! - Value comparison operations
//! - Type checking and coercion (for type hints)
//! - Type name formatting for error messages
//! - Numeric string validation

use crate::ast::TypeHint;
use crate::runtime::Value;
use crate::vm::VM;
use std::io::Write;

impl<W: Write> VM<W> {
    /// Add two values (for Add opcode)
    pub fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if let Some(result) = a.checked_add(*b) {
                    Ok(Value::Integer(result))
                } else {
                    Err("Integer overflow in addition".to_string())
                }
            }
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + (*b as f64))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), Value::Integer(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::String(a), Value::Float(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Array(_), Value::Integer(_)) => {
                let count = right.to_int();
                if let Value::Integer(n) = left {
                    let mut new_arr = left.clone();
                    if let Value::Array(mut arr) = new_arr {
                        for _ in 0..count {
                            arr.push((
                                crate::runtime::ArrayKey::Integer(arr.len() as i64),
                                n.clone(),
                            ));
                        }
                        Ok(Value::Array(arr))
                    } else {
                        Ok(Value::Array(vec![]))
                    }
                } else {
                    Ok(Value::Array(vec![]))
                }
            }
            (Value::Integer(_), Value::String(_)) => Ok(Value::Array(vec![])),
            (Value::Float(_), Value::String(_)) => Ok(Value::Array(vec![])),
            _ => Err(format!(
                "Invalid operand types for addition: {:?} + {:?}",
                left, right
            )),
        }
    }

    /// Compare two values (for comparison opcodes)
    pub fn compare_values(&self, left: &Value, right: &Value) -> Result<i64, String> {
        use crate::runtime::Value;
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                if a < (*b as f64) {
                    Ok(-1)
                } else if a > (*b as f64) {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::String(a), Value::String(b)) => {
                // String comparison with PHP's strcmp semantics
                let result = a.cmp(b);
                Ok(result as i64)
            }
            (Value::String(a), Value::Integer(b)) => {
                let result = a.cmp(&b.to_string());
                Ok(result as i64)
            }
            (Value::String(a), Value::Float(b)) => {
                let result = a.cmp(&b.to_string());
                Ok(result as i64)
            }
            (Value::Bool(a), Value::Bool(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Null, _) => {
                if right.is_null() {
                    Ok(0)
                } else {
                    Ok(1)
                }
            }
            _ => Err(format!("Cannot compare values: {:?} and {:?}", left, right)),
        }
    }

    /// Check if a value matches a type hint (strict mode - no coercion)
    /// Used for return type validation which is always strict in PHP
    pub fn value_matches_type_strict(&self, value: &Value, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type_strict(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type_strict(value, inner)
            }
            TypeHint::Union(types) => types
                .iter()
                .any(|t| self.value_matches_type_strict(value, t)),
            TypeHint::Intersection(types) => types
                .iter()
                .all(|t| self.value_matches_type_strict(value, t)),
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                intersections.iter().any(|group| {
                    // All types in group must match
                    group
                        .iter()
                        .all(|t| self.value_matches_type_strict(value, t))
                })
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,       // void is for return types only
            TypeHint::Never => false,      // never is for return types only
            TypeHint::Static => false,     // Requires class context
            TypeHint::SelfType => false,   // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }

    /// Helper for strict type matching (no coercion) - used for return types
    pub fn value_matches_simple_type_strict(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            ("string", Value::String(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            ("bool", Value::Bool(_)) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", crate::runtime::Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("null", Value::Null) => true,
            ("mixed", _) => true, // mixed accepts anything
            _ => {
                // Check if it's a class/interface/enum type
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, type_name)
                } else if let crate::runtime::Value::EnumCase { enum_name, .. } = value {
                    enum_name == type_name
                } else {
                    false
                }
            }
        }
    }

    /// Check if a value matches a type hint (includes coercive mode for scalars)
    pub fn value_matches_type(&self, value: &Value, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type(value, inner)
            }
            TypeHint::Union(types) => types.iter().any(|t| self.value_matches_type(value, t)),
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.value_matches_type(value, t))
            }
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                intersections.iter().any(|group| {
                    // All types in group must match
                    group.iter().all(|t| self.value_matches_type(value, t))
                })
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,       // void is for return types only
            TypeHint::Never => false,      // never is for return types only
            TypeHint::Static => false,     // Requires class context
            TypeHint::SelfType => false,   // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }

    /// Helper to check simple type matches (includes coercive mode for scalars)
    pub fn value_matches_simple_type(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            // Coercive mode: float can be coerced to int
            ("int", Value::Float(_)) => true,
            // Coercive mode: only numeric strings can be coerced to int
            ("int", Value::String(s)) => self.is_numeric_string(s),
            // Coercive mode: bool can be coerced to int
            ("int", Value::Bool(_)) => true,
            ("string", Value::String(_)) => true,
            // Coercive mode: scalars can be coerced to string
            ("string", Value::Integer(_)) => true,
            ("string", Value::Float(_)) => true,
            ("string", Value::Bool(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            // Coercive mode: only numeric strings can be coerced to float
            ("float", Value::String(s)) => self.is_numeric_string(s),
            ("bool", Value::Bool(_)) => true,
            // Coercive mode: any scalar can be coerced to bool
            ("bool", Value::Integer(_)) => true,
            ("bool", Value::Float(_)) => true,
            ("bool", Value::String(_)) => true,
            ("bool", Value::Null) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", crate::runtime::Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("mixed", _) => true,
            ("null", Value::Null) => true,
            ("false", Value::Bool(false)) => true,
            ("true", Value::Bool(true)) => true,
            _ => false,
        }
    }

    /// Check if a string is numeric (can be coerced to int/float)
    pub fn is_numeric_string(&self, s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        // Try to parse as a number - must be a valid numeric string
        // PHP considers strings like "123", "123.45", "1e5", "-42" as numeric
        if trimmed.parse::<i64>().is_ok() {
            return true;
        }
        if trimmed.parse::<f64>().is_ok() {
            return true;
        }
        false
    }

    /// Check if a type hint requires strict type checking (class/interface types)
    /// Simple scalar types (int, string, etc.) use PHP's coercive mode by default
    pub fn requires_strict_type_check(&self, type_hint: &TypeHint) -> bool {
        match type_hint {
            // Class types always require strict checking
            TypeHint::Class(_) => true,
            // Simple types - only check for class names (not scalar types)
            TypeHint::Simple(name) => {
                // These are scalar types that can be coerced
                !matches!(
                    name.as_str(),
                    "int"
                        | "string"
                        | "float"
                        | "bool"
                        | "array"
                        | "mixed"
                        | "null"
                        | "callable"
                        | "iterable"
                        | "object"
                )
            }
            // Check inner type for nullable
            TypeHint::Nullable(inner) => self.requires_strict_type_check(inner),
            // For union/intersection/DNF, check if any part requires strict checking
            TypeHint::Union(types) => types.iter().any(|t| self.requires_strict_type_check(t)),
            TypeHint::Intersection(types) => {
                types.iter().any(|t| self.requires_strict_type_check(t))
            }
            TypeHint::DNF(groups) => groups
                .iter()
                .any(|g| g.iter().any(|t| self.requires_strict_type_check(t))),
            // These special types are for return types
            TypeHint::Void
            | TypeHint::Never
            | TypeHint::Static
            | TypeHint::SelfType
            | TypeHint::ParentType => false,
        }
    }

    /// Get the type name for error messages
    pub fn get_value_type_name(&self, value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Fiber(_) => "Fiber",
            Value::Closure(_) => "Closure",
            crate::runtime::Value::Generator(_) => "Generator",
            crate::runtime::Value::EnumCase { .. } => "enum",
            crate::runtime::Value::Exception(_) => "Exception",
        }
    }

    /// Coerce a value to match a type hint (for coercive mode)
    pub fn coerce_value_to_type(&self, value: Value, type_hint: &TypeHint) -> Value {
        match type_hint {
            TypeHint::Simple(name) => {
                match name.as_str() {
                    "int" => {
                        // Convert to int using PHP rules
                        match &value {
                            Value::Integer(_) => value,
                            Value::Float(f) => Value::Integer(*f as i64),
                            Value::Bool(b) => Value::Integer(if *b { 1 } else { 0 }),
                            Value::String(s) => {
                                // Parse leading digits from string
                                let trimmed = s.trim_start();
                                if trimmed.is_empty() {
                                    return Value::Integer(0);
                                }
                                // Try to parse as integer or float, taking only leading valid part
                                let mut end_pos = 0;
                                let chars: Vec<char> = trimmed.chars().collect();
                                // Handle optional sign
                                if !chars.is_empty() && (chars[0] == '+' || chars[0] == '-') {
                                    end_pos = 1;
                                }
                                // Collect digits
                                while end_pos < chars.len() && chars[end_pos].is_ascii_digit() {
                                    end_pos += 1;
                                }
                                if end_pos == 0
                                    || (end_pos == 1 && (chars[0] == '+' || chars[0] == '-'))
                                {
                                    // No digits found - coerce to 0
                                    return Value::Integer(0);
                                }
                                // Parse numeric part
                                let numeric_part: String = chars[..end_pos].iter().collect();
                                Value::Integer(numeric_part.parse().unwrap_or(0))
                            }
                            _ => Value::Integer(value.to_int()),
                        }
                    }
                    "float" => Value::Float(value.to_float()),
                    "string" => Value::String(value.to_string_val()),
                    "bool" => Value::Bool(value.to_bool()),
                    _ => value, // For other types, don't coerce
                }
            }
            TypeHint::Nullable(inner) => {
                if matches!(value, Value::Null) {
                    value
                } else {
                    self.coerce_value_to_type(value, inner)
                }
            }
            _ => value, // For complex types, don't coerce
        }
    }

    /// Format a type hint for error messages
    pub fn format_type_hint(&self, type_hint: &TypeHint) -> String {
        match type_hint {
            TypeHint::Simple(name) => name.clone(),
            TypeHint::Nullable(inner) => format!("?{}", self.format_type_hint(inner)),
            TypeHint::Union(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Intersection(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("&"),
            TypeHint::DNF(groups) => groups
                .iter()
                .map(|group| {
                    let inner = group
                        .iter()
                        .map(|t| self.format_type_hint(t))
                        .collect::<Vec<_>>()
                        .join("&");
                    if group.len() > 1 {
                        format!("({})", inner)
                    } else {
                        inner
                    }
                })
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Class(name) => name.clone(),
            TypeHint::Void => "void".to_string(),
            TypeHint::Never => "never".to_string(),
            TypeHint::Static => "static".to_string(),
            TypeHint::SelfType => "self".to_string(),
            TypeHint::ParentType => "parent".to_string(),
        }
    }

    /// Convert a value to string, calling __toString for objects if available
    pub fn value_to_string(&mut self, value: Value) -> Result<String, String> {
        match value {
            crate::runtime::Value::Object(ref instance) => {
                let class_name = instance.class_name.clone();
                // Look for __toString method
                if let Some(to_string_method) = self.find_method_in_chain(&class_name, "__toString")
                {
                    let result = self.call_method_sync(instance.clone(), to_string_method)?;
                    match result {
                        crate::runtime::Value::String(s) => Ok(s),
                        _ => Err("__toString must return a string".to_string()),
                    }
                } else {
                    // No __toString method - this is an error in PHP
                    Err(format!(
                        "Object of class {} could not be converted to string",
                        class_name
                    ))
                }
            }
            _ => Ok(value.to_string_val()),
        }
    }
}

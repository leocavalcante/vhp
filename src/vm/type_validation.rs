//! Type checking, validation, and coercion for VM values
//!
//! This module provides functions for:
//! - Type hint matching (strict and coercive modes)
//! - Type coercion for parameter validation
//! - Type hint formatting for error messages
//! - Type name utilities

use crate::ast::TypeHint;
use crate::runtime::Value;
use crate::vm::VM;
use std::io::Write;

impl<W: Write> VM<W> {
    /// Check if a value matches a type hint (strict mode - no coercion)
    /// Used for return type validation which is always strict in PHP
    pub(crate) fn value_matches_type_strict(&self, value: &Value, type_hint: &TypeHint) -> bool {
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
            TypeHint::DNF(intersections) => intersections.iter().any(|group| {
                group
                    .iter()
                    .all(|t| self.value_matches_type_strict(value, t))
            }),
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,
            TypeHint::Never => false,
            TypeHint::Static => false,
            TypeHint::SelfType => false,
            TypeHint::ParentType => false,
        }
    }

    /// Helper for strict type matching (no coercion) - used for return types
    pub(crate) fn value_matches_simple_type_strict(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            ("string", Value::String(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true,
            ("bool", Value::Bool(_)) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true,
            ("iterable", Value::Array(_)) => true,
            ("null", Value::Null) => true,
            ("mixed", _) => true,
            _ => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, type_name)
                } else if let Value::EnumCase { enum_name, .. } = value {
                    enum_name == type_name
                } else {
                    false
                }
            }
        }
    }

    /// Check if a value matches a type hint (includes coercive mode for scalars)
    pub(crate) fn value_matches_type(&self, value: &Value, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type(value, inner)
            }
            TypeHint::Union(types) => types.iter().any(|t| self.value_matches_type(value, t)),
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.value_matches_type(value, t))
            }
            TypeHint::DNF(intersections) => intersections
                .iter()
                .any(|group| group.iter().all(|t| self.value_matches_type(value, t))),
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,
            TypeHint::Never => false,
            TypeHint::Static => false,
            TypeHint::SelfType => false,
            TypeHint::ParentType => false,
        }
    }

    /// Helper to check simple type matches (includes coercive mode for scalars)
    pub(crate) fn value_matches_simple_type(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            ("int", Value::Float(_)) => true,
            ("int", Value::String(s)) => self.is_numeric_string(s),
            ("int", Value::Bool(_)) => true,
            ("string", Value::String(_)) => true,
            ("string", Value::Integer(_)) => true,
            ("string", Value::Float(_)) => true,
            ("string", Value::Bool(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true,
            ("float", Value::String(s)) => self.is_numeric_string(s),
            ("bool", Value::Bool(_)) => true,
            ("bool", Value::Integer(_)) => true,
            ("bool", Value::Float(_)) => true,
            ("bool", Value::String(_)) => true,
            ("bool", Value::Null) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true,
            ("iterable", Value::Array(_)) => true,
            ("mixed", _) => true,
            ("null", Value::Null) => true,
            ("false", Value::Bool(false)) => true,
            ("true", Value::Bool(true)) => true,
            _ => false,
        }
    }

    /// Check if a string is numeric (can be coerced to int/float)
    pub(crate) fn is_numeric_string(&self, s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
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
    pub(crate) fn requires_strict_type_check(&self, type_hint: &TypeHint) -> bool {
        match type_hint {
            TypeHint::Class(_) => true,
            TypeHint::Simple(name) => !matches!(
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
            ),
            TypeHint::Nullable(inner) => self.requires_strict_type_check(inner),
            TypeHint::Union(types) => types.iter().any(|t| self.requires_strict_type_check(t)),
            TypeHint::Intersection(types) => {
                types.iter().any(|t| self.requires_strict_type_check(t))
            }
            TypeHint::DNF(groups) => groups
                .iter()
                .any(|g| g.iter().any(|t| self.requires_strict_type_check(t))),
            TypeHint::Void
            | TypeHint::Never
            | TypeHint::Static
            | TypeHint::SelfType
            | TypeHint::ParentType => false,
        }
    }

    /// Get the type name for error messages
    pub(crate) fn get_value_type_name(&self, value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Closure(_) => "Closure",
            Value::Fiber(_) => "Fiber",
            Value::Generator(_) => "Generator",
            Value::EnumCase { .. } => "enum",
            Value::Exception(_) => "Exception",
        }
    }

    /// Coerce a value to match a type hint (for coercive mode)
    pub(crate) fn coerce_value_to_type(&self, value: Value, type_hint: &TypeHint) -> Value {
        match type_hint {
            TypeHint::Simple(name) => match name.as_str() {
                "int" => match &value {
                    Value::Integer(_) => value,
                    Value::Float(f) => Value::Integer(*f as i64),
                    Value::Bool(b) => Value::Integer(if *b { 1 } else { 0 }),
                    Value::String(s) => {
                        let trimmed = s.trim_start();
                        if trimmed.is_empty() {
                            return Value::Integer(0);
                        }
                        let mut end_pos = 0;
                        let chars: Vec<char> = trimmed.chars().collect();
                        if !chars.is_empty() && (chars[0] == '+' || chars[0] == '-') {
                            end_pos = 1;
                        }
                        while end_pos < chars.len() && chars[end_pos].is_ascii_digit() {
                            end_pos += 1;
                        }
                        if end_pos == 0 || (end_pos == 1 && (chars[0] == '+' || chars[0] == '-')) {
                            return Value::Integer(0);
                        }
                        let numeric_part: String = chars[..end_pos].iter().collect();
                        Value::Integer(numeric_part.parse().unwrap_or(0))
                    }
                    _ => Value::Integer(value.to_int()),
                },
                "float" => Value::Float(value.to_float()),
                "string" => Value::String(value.to_string_val()),
                "bool" => Value::Bool(value.to_bool()),
                _ => value,
            },
            TypeHint::Nullable(inner) => {
                if matches!(value, Value::Null) {
                    value
                } else {
                    self.coerce_value_to_type(value, inner)
                }
            }
            _ => value,
        }
    }

    /// Format a type hint for error messages
    pub(crate) fn format_type_hint(&self, type_hint: &TypeHint) -> String {
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
}

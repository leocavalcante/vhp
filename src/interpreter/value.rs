//! Runtime value representation for VHP

use std::fmt;
use std::hash::{Hash, Hasher};

/// Array key type - PHP arrays support both integer and string keys
#[derive(Debug, Clone)]
pub enum ArrayKey {
    Integer(i64),
    String(String),
}

impl PartialEq for ArrayKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ArrayKey::Integer(a), ArrayKey::Integer(b)) => a == b,
            (ArrayKey::String(a), ArrayKey::String(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ArrayKey {}

impl Hash for ArrayKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ArrayKey::Integer(n) => {
                0u8.hash(state);
                n.hash(state);
            }
            ArrayKey::String(s) => {
                1u8.hash(state);
                s.hash(state);
            }
        }
    }
}

impl fmt::Display for ArrayKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrayKey::Integer(n) => write!(f, "{}", n),
            ArrayKey::String(s) => write!(f, "{}", s),
        }
    }
}

impl ArrayKey {
    /// Convert a Value to an ArrayKey following PHP's key coercion rules
    pub fn from_value(value: &Value) -> ArrayKey {
        match value {
            Value::Integer(n) => ArrayKey::Integer(*n),
            Value::Float(n) => ArrayKey::Integer(*n as i64),
            Value::Bool(b) => ArrayKey::Integer(if *b { 1 } else { 0 }),
            Value::Null => ArrayKey::String(String::new()),
            Value::String(s) => {
                // PHP converts numeric strings to integer keys
                if let Ok(n) = s.parse::<i64>() {
                    ArrayKey::Integer(n)
                } else {
                    ArrayKey::String(s.clone())
                }
            }
            Value::Array(_) => ArrayKey::String("Array".to_string()),
        }
    }

    /// Convert ArrayKey to a Value
    pub fn to_value(&self) -> Value {
        match self {
            ArrayKey::Integer(n) => Value::Integer(*n),
            ArrayKey::String(s) => Value::String(s.clone()),
        }
    }
}

/// Runtime value representation
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<(ArrayKey, Value)>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.type_equals(other)
    }
}

impl Value {
    /// Convert value to string for output
    pub fn to_output_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    String::new()
                }
            }
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(_) => "Array".to_string(),
        }
    }

    /// Convert to boolean (PHP truthiness)
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty() && s != "0",
            Value::Array(arr) => !arr.is_empty(),
        }
    }

    /// Convert to integer
    pub fn to_int(&self) -> i64 {
        match self {
            Value::Null => 0,
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Value::Integer(n) => *n,
            Value::Float(n) => *n as i64,
            Value::String(s) => s.parse().unwrap_or(0),
            Value::Array(arr) => {
                if arr.is_empty() {
                    0
                } else {
                    1
                }
            }
        }
    }

    /// Convert to float
    pub fn to_float(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Integer(n) => *n as f64,
            Value::Float(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0.0),
            Value::Array(arr) => {
                if arr.is_empty() {
                    0.0
                } else {
                    1.0
                }
            }
        }
    }

    /// Convert to string
    pub fn to_string_val(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    String::new()
                }
            }
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Array(_) => "Array".to_string(),
        }
    }

    /// Check if value is numeric (used by is_numeric built-in function)
    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    /// Check if value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Check type equality for === and !==
    pub fn type_equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for ((k1, v1), (k2, v2)) in a.iter().zip(b.iter()) {
                    if k1 != k2 || !v1.type_equals(v2) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Loose equality for == and !=
    pub fn loose_equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Null, Value::Bool(b)) | (Value::Bool(b), Value::Null) => !b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
                (*a as f64) == *b
            }
            (Value::String(a), Value::String(b)) => a == b,
            // Numeric string comparisons
            (Value::Integer(n), Value::String(s)) | (Value::String(s), Value::Integer(n)) => {
                if let Ok(sn) = s.parse::<i64>() {
                    *n == sn
                } else if let Ok(sf) = s.parse::<f64>() {
                    (*n as f64) == sf
                } else {
                    false
                }
            }
            (Value::Float(n), Value::String(s)) | (Value::String(s), Value::Float(n)) => {
                if let Ok(sf) = s.parse::<f64>() {
                    *n == sf
                } else {
                    false
                }
            }
            // Array comparisons
            (Value::Array(a), Value::Array(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for ((k1, v1), (k2, v2)) in a.iter().zip(b.iter()) {
                    if k1 != k2 || !v1.loose_equals(v2) {
                        return false;
                    }
                }
                true
            }
            _ => self.to_bool() == other.to_bool(),
        }
    }

    /// Get the PHP type name
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Null => "NULL",
            Value::Bool(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "double",
            Value::String(_) => "string",
            Value::Array(_) => "array",
        }
    }
}

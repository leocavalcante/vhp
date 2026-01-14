impl super::Value {
    pub fn to_bool(&self) -> bool {
        match self {
            super::Value::Null => false,
            super::Value::Bool(b) => *b,
            super::Value::Integer(n) => *n != 0,
            super::Value::Float(n) => *n != 0.0,
            super::Value::String(s) => !s.is_empty() && s != "0",
            super::Value::Array(arr) => !arr.is_empty(),
            super::Value::Object(_) => true,
            super::Value::Fiber(_) => true,
            super::Value::Closure(_) => true,
            super::Value::Generator(_) => true,
            super::Value::EnumCase { .. } => true,
            super::Value::Exception(_) => true,
        }
    }

    pub fn to_int(&self) -> i64 {
        match self {
            super::Value::Null => 0,
            super::Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            super::Value::Integer(n) => *n,
            super::Value::Float(n) => *n as i64,
            super::Value::String(s) => s.parse().unwrap_or(0),
            super::Value::Array(arr) => {
                if arr.is_empty() {
                    0
                } else {
                    1
                }
            }
            super::Value::Object(_) => 1,
            super::Value::Fiber(_) => 0,
            super::Value::Closure(_) => 1,
            super::Value::Generator(_) => 0,
            super::Value::EnumCase { .. } => 1,
            super::Value::Exception(_) => 1,
        }
    }

    pub fn to_float(&self) -> f64 {
        match self {
            super::Value::Null => 0.0,
            super::Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            super::Value::Integer(n) => *n as f64,
            super::Value::Float(n) => *n,
            super::Value::String(s) => s.parse().unwrap_or(0.0),
            super::Value::Array(arr) => {
                if arr.is_empty() {
                    0.0
                } else {
                    1.0
                }
            }
            super::Value::Object(_) => 1.0,
            super::Value::Fiber(_) => 0.0,
            super::Value::Closure(_) => 1.0,
            super::Value::Generator(_) => 0.0,
            super::Value::EnumCase { .. } => 1.0,
            super::Value::Exception(_) => 1.0,
        }
    }

    pub fn to_string_val(&self) -> String {
        match self {
            super::Value::Null => String::new(),
            super::Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    String::new()
                }
            }
            super::Value::Integer(n) => n.to_string(),
            super::Value::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            super::Value::String(s) => s.clone(),
            super::Value::Array(_) => "Array".to_string(),
            super::Value::Object(obj) => format!("Object({})", obj.class_name),
            super::Value::Fiber(_) => "Object(Fiber)".to_string(),
            super::Value::Closure(_) => "Object(Closure)".to_string(),
            super::Value::Generator(_) => "Object(Generator)".to_string(),
            super::Value::EnumCase {
                enum_name,
                case_name,
                ..
            } => format!("{}::{}", enum_name, case_name),
            super::Value::Exception(exc) => format!("Object({})", exc.class_name),
        }
    }

    pub fn type_equals(&self, other: &super::Value) -> bool {
        match (self, other) {
            (super::Value::Null, super::Value::Null) => true,
            (super::Value::Bool(a), super::Value::Bool(b)) => a == b,
            (super::Value::Integer(a), super::Value::Integer(b)) => a == b,
            (super::Value::Float(a), super::Value::Float(b)) => a == b,
            (super::Value::String(a), super::Value::String(b)) => a == b,
            (super::Value::Array(a), super::Value::Array(b)) => {
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
            (super::Value::Object(a), super::Value::Object(b)) => {
                a.class_name == b.class_name && a.properties == b.properties
            }
            (super::Value::Fiber(a), super::Value::Fiber(b)) => a.id == b.id,
            (super::Value::Closure(_), super::Value::Closure(_)) => false,
            (super::Value::Generator(a), super::Value::Generator(b)) => a.id == b.id,
            (
                super::Value::EnumCase {
                    enum_name: en1,
                    case_name: cn1,
                    ..
                },
                super::Value::EnumCase {
                    enum_name: en2,
                    case_name: cn2,
                    ..
                },
            ) => en1 == en2 && cn1 == cn2,
            (super::Value::Exception(a), super::Value::Exception(b)) => {
                a.class_name == b.class_name && a.message == b.message
            }
            _ => false,
        }
    }

    pub fn loose_equals(&self, other: &super::Value) -> bool {
        match (self, other) {
            (super::Value::Null, super::Value::Null) => true,
            (super::Value::Null, super::Value::Bool(b))
            | (super::Value::Bool(b), super::Value::Null) => !b,
            (super::Value::Bool(a), super::Value::Bool(b)) => a == b,
            (super::Value::Integer(a), super::Value::Integer(b)) => a == b,
            (super::Value::Float(a), super::Value::Float(b)) => a == b,
            (super::Value::Integer(a), super::Value::Float(b))
            | (super::Value::Float(b), super::Value::Integer(a)) => (*a as f64) == *b,
            (super::Value::String(a), super::Value::String(b)) => a == b,
            (super::Value::Integer(n), super::Value::String(s))
            | (super::Value::String(s), super::Value::Integer(n)) => {
                if let Ok(sn) = s.parse::<i64>() {
                    *n == sn
                } else if let Ok(sf) = s.parse::<f64>() {
                    (*n as f64) == sf
                } else {
                    false
                }
            }
            (super::Value::Float(n), super::Value::String(s))
            | (super::Value::String(s), super::Value::Float(n)) => {
                if let Ok(sf) = s.parse::<f64>() {
                    *n == sf
                } else {
                    false
                }
            }
            (super::Value::Array(a), super::Value::Array(b)) => {
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
            (super::Value::Object(a), super::Value::Object(b)) => {
                a.class_name == b.class_name && a.properties == b.properties
            }
            (super::Value::Fiber(a), super::Value::Fiber(b)) => a.id == b.id,
            (super::Value::Closure(_), super::Value::Closure(_)) => false,
            (super::Value::Generator(a), super::Value::Generator(b)) => a.id == b.id,
            (
                super::Value::EnumCase {
                    enum_name: en1,
                    case_name: cn1,
                    ..
                },
                super::Value::EnumCase {
                    enum_name: en2,
                    case_name: cn2,
                    ..
                },
            ) => en1 == en2 && cn1 == cn2,
            (super::Value::Exception(a), super::Value::Exception(b)) => {
                a.class_name == b.class_name && a.message == b.message
            }
            _ => self.to_bool() == other.to_bool(),
        }
    }

    pub fn get_type(&self) -> &'static str {
        match self {
            super::Value::Null => "NULL",
            super::Value::Bool(_) => "boolean",
            super::Value::Integer(_) => "integer",
            super::Value::Float(_) => "double",
            super::Value::String(_) => "string",
            super::Value::Array(_) => "array",
            super::Value::Object(_) => "object",
            super::Value::Fiber(_) => "object",
            super::Value::Closure(_) => "object",
            super::Value::Generator(_) => "object",
            super::Value::EnumCase { .. } => "object",
            super::Value::Exception(_) => "object",
        }
    }

    #[allow(dead_code)]
    pub fn type_name(&self) -> &'static str {
        match self {
            super::Value::Null => "null",
            super::Value::Bool(_) => "bool",
            super::Value::Integer(_) => "int",
            super::Value::Float(_) => "float",
            super::Value::String(_) => "string",
            super::Value::Array(_) => "array",
            super::Value::Object(obj) => Box::leak(obj.class_name.clone().into_boxed_str()),
            super::Value::Fiber(_) => "Fiber",
            super::Value::Closure(_) => "Closure",
            super::Value::Generator(_) => "Generator",
            super::Value::EnumCase { enum_name, .. } => {
                Box::leak(enum_name.clone().into_boxed_str())
            }
            super::Value::Exception(exc) => Box::leak(exc.class_name.clone().into_boxed_str()),
        }
    }

    #[allow(dead_code)]
    pub fn matches_type_strict(&self, type_hint: &crate::ast::TypeHint) -> bool {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => match (name.as_str(), self) {
                ("int", super::Value::Integer(_)) => true,
                ("float", super::Value::Float(_)) => true,
                ("float", super::Value::Integer(_)) => true,
                ("string", super::Value::String(_)) => true,
                ("bool", super::Value::Bool(_)) => true,
                ("array", super::Value::Array(_)) => true,
                ("object", super::Value::Object(_)) => true,
                ("object", super::Value::Fiber(_)) => true,
                ("object", super::Value::Closure(_)) => true,
                ("object", super::Value::EnumCase { .. }) => true,
                ("callable", super::Value::Closure(_)) => true,
                ("callable", super::Value::String(_)) => true,
                ("iterable", super::Value::Array(_)) => true,
                ("mixed", _) => true,
                ("null", super::Value::Null) => true,
                ("false", super::Value::Bool(false)) => true,
                ("true", super::Value::Bool(true)) => true,
                _ => false,
            },
            TypeHint::Nullable(inner) => {
                matches!(self, super::Value::Null) || self.matches_type_strict(inner)
            }
            TypeHint::Union(types) => types.iter().any(|t| self.matches_type_strict(t)),
            TypeHint::Intersection(types) => types.iter().all(|t| self.matches_type_strict(t)),
            TypeHint::DNF(intersections) => intersections
                .iter()
                .any(|group| group.iter().all(|t| self.matches_type_strict(t))),
            TypeHint::Class(class_name) => {
                if let super::Value::Object(obj) = self {
                    obj.is_instance_of(class_name)
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
}

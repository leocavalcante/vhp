use std::fmt;
use std::hash::{Hash, Hasher};

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
    pub fn from_value(value: &super::Value) -> ArrayKey {
        match value {
            super::Value::Integer(n) => ArrayKey::Integer(*n),
            super::Value::Float(n) => ArrayKey::Integer(*n as i64),
            super::Value::Bool(b) => ArrayKey::Integer(if *b { 1 } else { 0 }),
            super::Value::Null => ArrayKey::String(String::new()),
            super::Value::String(s) => {
                if let Ok(n) = s.parse::<i64>() {
                    ArrayKey::Integer(n)
                } else {
                    ArrayKey::String(s.clone())
                }
            }
            super::Value::Array(_) => ArrayKey::String("Array".to_string()),
            super::Value::Object(obj) => ArrayKey::String(format!("Object({})", obj.class_name)),
            super::Value::Fiber(fiber) => {
                ArrayKey::String(format!("Object(Fiber#{:06})", fiber.id))
            }
            super::Value::Closure(_) => ArrayKey::String("Object(Closure)".to_string()),
            super::Value::Generator(gen) => {
                ArrayKey::String(format!("Object(Generator#{:06})", gen.id))
            }
            super::Value::EnumCase {
                enum_name,
                case_name,
                ..
            } => ArrayKey::String(format!("{}::{}", enum_name, case_name)),
            super::Value::Exception(exc) => ArrayKey::String(format!("Object({})", exc.class_name)),
        }
    }

    pub fn to_value(&self) -> super::Value {
        match self {
            ArrayKey::Integer(n) => super::Value::Integer(*n),
            ArrayKey::String(s) => super::Value::String(s.clone()),
        }
    }
}

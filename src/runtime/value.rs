//! Runtime value representation for VHP

use std::collections::HashMap;

pub mod array_key;
pub mod object_instance;
pub mod value_helpers;

pub use array_key::ArrayKey;
pub use object_instance::{ExceptionValue, ObjectInstance};

/// Closure (arrow function or anonymous function)
#[derive(Debug, Clone)]
#[allow(dead_code)] // params parsed but not yet used
pub struct Closure {
    pub params: Vec<crate::ast::FunctionParam>,
    pub body: ClosureBody,
    pub captured_vars: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ClosureBody {
    Expression(Box<crate::ast::Expr>),
    FunctionRef(String),
    MethodRef {
        class_name: String,
        method_name: String,
    },
    StaticMethodRef {
        class_name: String,
        method_name: String,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FiberInstance {
    pub id: usize,
    pub state: FiberState,
    pub callback: Option<crate::runtime::UserFunction>,
    pub call_stack: Vec<CallFrame>,
    pub variables: HashMap<String, Value>,
    pub suspended_value: Option<Box<Value>>,
    pub return_value: Option<Box<Value>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FiberState {
    NotStarted,
    Running,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CallFrame {
    pub function_name: String,
    pub variables: HashMap<String, Value>,
    pub statements: Vec<crate::ast::Stmt>,
    pub current_statement: usize,
}

#[derive(Debug, Clone)]
pub struct GeneratorInstance {
    pub id: usize,
    pub position: usize,
    pub values: Vec<Value>,
    pub statements: Vec<crate::ast::Stmt>,
    pub current_statement: usize,
    pub variables: HashMap<String, Value>,
    pub finished: bool,
}

/// Runtime value representation
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<(ArrayKey, Value)>),
    Object(ObjectInstance),
    Fiber(Box<FiberInstance>),
    Closure(Box<Closure>),
    Generator(Box<GeneratorInstance>),
    EnumCase {
        enum_name: String,
        case_name: String,
        backing_value: Option<Box<Value>>,
    },
    Exception(ExceptionValue),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.type_equals(other)
    }
}

impl Value {
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
            Value::Object(obj) => format!("Object({})", obj.class_name),
            Value::Fiber(fiber) => format!("Object(Fiber#{:06})", fiber.id),
            Value::Closure(_) => "Object(Closure)".to_string(),
            Value::Generator(gen) => format!("Object(Generator#{:06})", gen.id),
            Value::EnumCase {
                enum_name,
                case_name,
                ..
            } => format!("{}::{}", enum_name, case_name),
            Value::Exception(exc) => format!("Object({})", exc.class_name),
        }
    }

    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    #[allow(dead_code)]
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }
}

impl ObjectInstance {}

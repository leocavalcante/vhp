use super::ops::{AssignOp, BinaryOp, UnaryOp};

/// Array element with optional key
#[derive(Debug, Clone)]
pub struct ArrayElement {
    pub key: Option<Box<Expr>>,
    pub value: Box<Expr>,
}

/// Match arm for match expressions (PHP 8.0)
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub conditions: Vec<Expr>, // Multiple conditions separated by comma
    pub result: Box<Expr>,
}

/// Function call argument (PHP 8.0 named arguments)
#[derive(Debug, Clone)]
pub enum FunctionArg {
    Positional(Expr),
    Named { name: String, value: Expr },
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    // Literals
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,

    // Variable
    Variable(String),

    // Array literal
    Array(Vec<ArrayElement>),

    // Array access
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },

    // Operations
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Assign {
        var: String,
        op: AssignOp,
        value: Box<Expr>,
    },
    // Array element assignment: $arr[key] = value
    ArrayAssign {
        array: Box<Expr>,
        index: Option<Box<Expr>>, // None for $arr[] = value (append)
        op: AssignOp,
        value: Box<Expr>,
    },

    // Grouping
    Grouped(Box<Expr>),

    // Ternary
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },

    // Function call
    FunctionCall {
        name: String,
        args: Vec<FunctionArg>,
    },

    // Object instantiation: new ClassName(args)
    New {
        class_name: String,
        args: Vec<FunctionArg>,
    },

    // Property access: $obj->property
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },

    // Method call: $obj->method(args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<FunctionArg>,
    },

    // Property assignment: $obj->property = value
    PropertyAssign {
        object: Box<Expr>,
        property: String,
        value: Box<Expr>,
    },

    // $this reference
    This,

    // Static method call: ClassName::method(args)
    StaticMethodCall {
        class_name: String,
        method: String,
        args: Vec<FunctionArg>,
    },

    // Match expression (PHP 8.0)
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        default: Option<Box<Expr>>,
    },
}

use super::ops::{AssignOp, BinaryOp, UnaryOp};

/// Array element with optional key
#[derive(Debug, Clone)]
pub struct ArrayElement {
    pub key: Option<Box<Expr>>,
    pub value: Box<Expr>,
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
        args: Vec<Expr>,
    },
}

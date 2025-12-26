/// AST nodes for VHP

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Mod,        // %
    Pow,        // **

    // String
    Concat,     // .

    // Comparison
    Equal,      // ==
    Identical,  // ===
    NotEqual,   // !=
    NotIdentical, // !==
    LessThan,   // <
    GreaterThan, // >
    LessEqual,  // <=
    GreaterEqual, // >=
    Spaceship,  // <=>

    // Logical
    And,        // && or 'and'
    Or,         // || or 'or'
    Xor,        // xor

    // Null coalescing
    NullCoalesce, // ??
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,        // -
    Not,        // !
    PreInc,     // ++$x
    PreDec,     // --$x
    PostInc,    // $x++
    PostDec,    // $x--
}

/// Assignment operators
#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,     // =
    AddAssign,  // +=
    SubAssign,  // -=
    MulAssign,  // *=
    DivAssign,  // /=
    ModAssign,  // %=
    ConcatAssign, // .=
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

/// Statements
#[derive(Debug, Clone)]
pub enum Stmt {
    Echo(Vec<Expr>),
    Expression(Expr),
    Html(String),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elseif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    For {
        init: Option<Expr>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    Foreach {
        array: Expr,
        key: Option<String>,
        value: String,
        body: Vec<Stmt>,
    },
    Switch {
        expr: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Stmt>>,
    },
    Break,
    Continue,
    Function {
        name: String,
        params: Vec<FunctionParam>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
}

/// Switch case
#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub value: Expr,
    pub body: Vec<Stmt>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub default: Option<Expr>,
    pub by_ref: bool,
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

use super::expr::Expr;

/// Visibility modifier for class members
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
}

/// Class method definition
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
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
    /// Foreach loop (fields will be used when array support is implemented)
    #[allow(dead_code)]
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
    Class {
        name: String,
        parent: Option<String>,
        properties: Vec<Property>,
        methods: Vec<Method>,
    },
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
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Visibility for constructor property promotion (PHP 8.0)
    /// If Some(visibility), this parameter is promoted to a property
    pub promoted: Option<Visibility>,
}

/// Program root
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

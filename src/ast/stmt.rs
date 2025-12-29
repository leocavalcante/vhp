use super::expr::Expr;

/// Visibility modifier for class members
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

/// Attribute argument (can be positional or named)
#[derive(Debug, Clone)]
pub struct AttributeArgument {
    #[allow(dead_code)] // Will be used for reflection
    pub name: Option<String>, // None for positional, Some("name") for named
    #[allow(dead_code)] // Will be used for reflection
    pub value: Expr,
}

/// Attribute metadata (PHP 8.0)
#[derive(Debug, Clone)]
pub struct Attribute {
    #[allow(dead_code)] // Will be used for reflection
    pub name: String,
    #[allow(dead_code)] // Will be used for reflection
    pub arguments: Vec<AttributeArgument>,
}

/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub default: Option<Expr>,
    pub readonly: bool,             // PHP 8.1+
    pub attributes: Vec<Attribute>, // PHP 8.0+
}

/// Class method definition
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
    pub attributes: Vec<Attribute>, // PHP 8.0+
}

/// Interface method signature (no body)
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<Attribute>, // PHP 8.0+
}

/// Interface constant
#[derive(Debug, Clone)]
pub struct InterfaceConstant {
    pub name: String,
    pub value: Expr,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<Attribute>, // PHP 8.0+
}

/// Enum case definition
#[derive(Debug, Clone)]
pub struct EnumCase {
    pub name: String,
    pub value: Option<Expr>, // Some(expr) for backed enums, None for pure enums
}

/// Enum backing type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnumBackingType {
    None,   // Pure enum
    Int,    // Backed by integers
    String, // Backed by strings
}

/// Trait usage in class
#[derive(Debug, Clone)]
pub struct TraitUse {
    pub traits: Vec<String>,
    #[allow(dead_code)] // Will be used for trait conflict resolution
    pub resolutions: Vec<TraitResolution>,
}

/// Conflict resolution for traits
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used for trait conflict resolution
pub enum TraitResolution {
    InsteadOf {
        trait_name: String,
        method: String,
        excluded_traits: Vec<String>,
    },
    Alias {
        trait_name: Option<String>,
        method: String,
        alias: String,
        visibility: Option<Visibility>,
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
        attributes: Vec<Attribute>, // PHP 8.0+
    },
    Return(Option<Expr>),
    Interface {
        name: String,
        parents: Vec<String>,
        methods: Vec<InterfaceMethodSignature>,
        constants: Vec<InterfaceConstant>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
    Trait {
        name: String,
        uses: Vec<String>,
        properties: Vec<Property>,
        methods: Vec<Method>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
    Class {
        name: String,
        readonly: bool, // PHP 8.2+: all properties are implicitly readonly
        parent: Option<String>,
        interfaces: Vec<String>,
        trait_uses: Vec<TraitUse>,
        properties: Vec<Property>,
        methods: Vec<Method>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
    Enum {
        name: String,
        backing_type: EnumBackingType,
        cases: Vec<EnumCase>,
        methods: Vec<Method>,       // Enums can have methods
        attributes: Vec<Attribute>, // PHP 8.0+
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
    pub visibility: Option<Visibility>,
    /// Readonly modifier for constructor property promotion (PHP 8.1)
    pub readonly: bool,
    /// Attributes for parameters (PHP 8.0)
    pub attributes: Vec<Attribute>,
}

/// Program root
#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

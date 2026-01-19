use super::ops::{AssignOp, BinaryOp, UnaryOp};

/// Array element with optional key
#[derive(Debug, Clone)]
pub struct ArrayElement {
    pub key: Option<Box<Expr>>,
    pub value: Box<Expr>,
}

/// List element for destructuring with optional key
#[derive(Debug, Clone)]
pub struct ListElement {
    pub key: Option<Box<Expr>>, // Optional key: "key" => $var
    pub value: Box<Expr>,       // Variable or nested list
}

/// Property modification for clone with syntax (PHP 8.4)
#[derive(Debug, Clone)]
pub struct PropertyModification {
    pub property: String,
    pub value: Box<Expr>,
}

/// Function/method call argument with optional name (PHP 8.0 named arguments)
#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Option<String>, // None for positional, Some("name") for named
    pub value: Box<Expr>,
}

/// Match arm for match expressions (PHP 8.0)
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub conditions: Vec<Expr>, // Multiple conditions separated by comma
    pub result: Box<Expr>,
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

    // Heredoc string with variable interpolation
    /// Contains the content with variable placeholders
    Heredoc(String),

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
        args: Vec<Argument>,
    },

    // Variable/callable function call: $func(), $obj->method() result(), etc.
    CallableCall {
        callable: Box<Expr>,
        args: Vec<Argument>,
    },

    // Object instantiation: new ClassName(args)
    New {
        class_name: String,
        args: Vec<Argument>,
    },

    // Anonymous class instantiation (PHP 7.0): new class(...) extends X implements Y { ... }
    #[allow(dead_code)] // interfaces and traits parsed but not yet used
    NewAnonymousClass {
        constructor_args: Vec<Argument>,
        parent: Option<String>,
        interfaces: Vec<String>,
        traits: Vec<crate::ast::TraitUse>,
        properties: Vec<crate::ast::Property>,
        methods: Vec<crate::ast::Method>,
    },

    // Fiber instantiation: new Fiber(callback) - Special case
    NewFiber {
        callback: Box<Expr>, // Function name or closure
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
        args: Vec<Argument>,
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
        args: Vec<Argument>,
    },

    // Static property access: ClassName::$property or static::$property
    StaticPropertyAccess {
        class: String, // Can be "self", "parent", or "static" for LSB
        property: String,
    },

    // Static property assignment: ClassName::$property = value
    StaticPropertyAssign {
        class: String,
        property: String,
        value: Box<Expr>,
    },

    // Fiber static calls - Special cases for suspend/getCurrent
    FiberSuspend {
        value: Option<Box<Expr>>, // Optional value to suspend with
    },

    FiberGetCurrent,

    // Match expression (PHP 8.0)
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        default: Option<Box<Expr>>,
    },

    // Enum case access: EnumName::CASE
    EnumCase {
        enum_name: String,
        case_name: String,
    },

    // Clone expression: clone $obj
    Clone {
        object: Box<Expr>,
    },

    // Clone with expression: clone $obj with { prop: value, ... }
    CloneWith {
        object: Box<Expr>,
        modifications: Vec<PropertyModification>,
    },

    // Placeholder for pipe operator: ... (PHP 8.5)
    Placeholder,

    // Spread/unpack expression: ...$array
    Spread(Box<Expr>),

    // Arrow function (PHP 7.4): fn($params) => expr
    ArrowFunction {
        params: Vec<crate::ast::FunctionParam>,
        body: Box<Expr>, // Single expression (not statement block)
    },

    // First-class callable (PHP 8.1): functionName(...)
    CallableFromFunction(String),

    // First-class callable from method (PHP 8.1): $obj->method(...)
    CallableFromMethod {
        object: Box<Expr>,
        method: String,
    },

    // First-class callable from static method (PHP 8.1): Class::method(...)
    CallableFromStaticMethod {
        class: String,
        method: String,
    },

    // Throw expression (PHP 8.0+)
    /// Used in: $x ?? throw new Exception("..."), fn() => throw new Exception()
    Throw(Box<Expr>),

    // Yield expression (PHP 5.5+)
    /// yield, yield $value, or yield $key => $value
    Yield {
        key: Option<Box<Expr>>,
        value: Option<Box<Expr>>,
    },

    // Yield from expression (PHP 7.0+)
    /// yield from $iterable
    YieldFrom(Box<Expr>),

    // List destructuring (PHP 7.1+): list($a, $b) = $array
    /// Contains the list elements with optional keys and the source array
    ListDestructure {
        elements: Vec<ListElement>,
        array: Box<Expr>,
    },

    // Magic constants (compile-time resolved)
    /// __FILE__ - Full path of the file being executed
    MagicFile,
    /// __LINE__ - Current line number (1-based)
    MagicLine(usize),
    /// __DIR__ - Directory of the file being executed
    MagicDir,
    /// __FUNCTION__ - Current function name (or empty at top level)
    MagicFunction,
    /// __CLASS__ - Current class name (or empty at top level)
    MagicClass,
    /// __METHOD__ - Current method name with class (or empty at top level)
    MagicMethod,
    /// __NAMESPACE__ - Current namespace (or empty if no namespace)
    MagicNamespace,
    /// __TRAIT__ - Current trait name (or empty)
    MagicTrait,
}

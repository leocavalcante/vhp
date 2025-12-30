use super::expr::Expr;

/// Declare directive type
#[derive(Debug, Clone)]
pub enum DeclareDirective {
    /// strict_types=0 or strict_types=1
    StrictTypes(bool),
    /// encoding='UTF-8' (mostly ignored in modern PHP)
    #[allow(dead_code)]
    Encoding(String),
    /// ticks=N (for register_tick_function, advanced feature)
    #[allow(dead_code)]
    Ticks(i64),
}

/// Qualified name for namespace and class references (e.g., MyApp\Database\Connection)
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedName {
    /// Path parts (e.g., ["MyApp", "Database", "Connection"])
    pub parts: Vec<String>,
    /// Whether it starts with \ (fully qualified)
    pub is_fully_qualified: bool,
}

impl QualifiedName {
    pub fn new(parts: Vec<String>, is_fully_qualified: bool) -> Self {
        Self {
            parts,
            is_fully_qualified,
        }
    }

    /// Get just the final name (class name, function name, etc.)
    pub fn last(&self) -> Option<&String> {
        self.parts.last()
    }
}

/// Use statement type
#[derive(Debug, Clone, PartialEq)]
pub enum UseType {
    Class,    // use Foo\Bar;
    Function, // use function Foo\helper;
    Constant, // use const Foo\VALUE;
}

/// Single use import
#[derive(Debug, Clone)]
pub struct UseItem {
    pub name: QualifiedName,
    pub alias: Option<String>, // `as` alias
    pub use_type: UseType,
}

/// Group use statement: use Foo\{Bar, Baz};
#[derive(Debug, Clone)]
pub struct GroupUse {
    pub prefix: QualifiedName,
    pub items: Vec<UseItem>,
}

/// Namespace body style
#[derive(Debug, Clone)]
pub enum NamespaceBody {
    /// Braced: namespace Foo { ... }
    Braced(Vec<Stmt>),
    /// Unbraced: namespace Foo; (rest of file)
    Unbraced,
}

/// Type hint for parameters and return values
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum TypeHint {
    /// Simple type: int, string, float, bool, array, object, callable, mixed
    Simple(String),
    /// Nullable type: ?int, ?string, etc.
    Nullable(Box<TypeHint>),
    /// Union type (PHP 8.0+): int|string, int|null
    Union(Vec<TypeHint>),
    /// Intersection type (PHP 8.1+): Iterator&Countable
    Intersection(Vec<TypeHint>),
    /// DNF type (PHP 8.2+): (A&B)|C, (A&B)|(C&D)
    /// Each inner Vec represents an intersection, the outer Vec is the union
    /// Example: (A&B)|C is represented as [[A, B], [C]]
    DNF(Vec<Vec<TypeHint>>),
    /// Class/interface type
    Class(String),
    /// void (only for return types)
    Void,
    /// never (PHP 8.1+, only for return types)
    Never,
    /// static (PHP 8.0+, only for return types)
    Static,
    /// self/parent (in class context)
    SelfType,
    ParentType,
}

impl TypeHint {
    /// Check if this type hint allows null values
    #[allow(dead_code)] // Will be used for type validation
    pub fn is_nullable(&self) -> bool {
        match self {
            TypeHint::Nullable(_) => true,
            TypeHint::Union(types) => types
                .iter()
                .any(|t| matches!(t, TypeHint::Simple(s) if s == "null")),
            TypeHint::Simple(s) => s == "mixed" || s == "null",
            TypeHint::DNF(intersections) => {
                // DNF is nullable if any intersection group contains only null
                intersections.iter().any(|group| {
                    group.len() == 1 && matches!(&group[0], TypeHint::Simple(s) if s == "null")
                })
            }
            _ => false,
        }
    }
}

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

/// Property hook type (PHP 8.4)
#[derive(Debug, Clone)]
pub enum PropertyHookType {
    Get,
    Set,
}

/// Property hook body can be expression or statements
#[derive(Debug, Clone)]
pub enum PropertyHookBody {
    /// Short syntax: get => expr
    Expression(Box<Expr>),
    /// Block syntax: get { statements }
    Block(Vec<Stmt>),
}

/// Property hook definition (PHP 8.4)
#[derive(Debug, Clone)]
pub struct PropertyHook {
    pub hook_type: PropertyHookType,
    pub body: PropertyHookBody,
}

/// Class property definition
#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub visibility: Visibility,
    pub write_visibility: Option<Visibility>, // PHP 8.4+ asymmetric visibility, None means same as read
    pub default: Option<Expr>,
    pub readonly: bool,             // PHP 8.1+
    pub is_static: bool,            // PHP 5.0+
    pub attributes: Vec<Attribute>, // PHP 8.0+
    pub hooks: Vec<PropertyHook>,   // PHP 8.4+
}

/// Class method definition
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeHint>,
    pub body: Vec<Stmt>,
    pub attributes: Vec<Attribute>, // PHP 8.0+
}

/// Interface method signature (no body)
#[derive(Debug, Clone)]
pub struct InterfaceMethodSignature {
    pub name: String,
    pub params: Vec<FunctionParam>,
    #[allow(dead_code)] // Will be used for type validation
    pub return_type: Option<TypeHint>,
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

/// Catch clause for try statement
#[derive(Debug, Clone)]
pub struct CatchClause {
    /// Exception types to catch (supports multi-catch with |)
    pub exception_types: Vec<String>,
    /// Variable name to bind exception (e.g., $e)
    pub variable: String,
    /// Body of catch block
    pub body: Vec<Stmt>,
}

/// Statements
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
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
        return_type: Option<TypeHint>,
        body: Vec<Stmt>,
        attributes: Vec<Attribute>, // PHP 8.0+
    },
    Return(Option<Expr>),
    Interface {
        name: String,
        parents: Vec<QualifiedName>,
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
        is_abstract: bool, // abstract class modifier
        is_final: bool,    // final class modifier
        readonly: bool,    // PHP 8.2+: all properties are implicitly readonly
        parent: Option<QualifiedName>,
        interfaces: Vec<QualifiedName>,
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
    /// Try/Catch/Finally statement
    TryCatch {
        try_body: Vec<Stmt>,
        catch_clauses: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
    },
    /// Throw statement
    Throw(Expr),
    /// Namespace declaration
    Namespace {
        name: Option<QualifiedName>, // None for global namespace
        body: NamespaceBody,
    },
    /// Use statement
    Use(Vec<UseItem>),
    /// Group use statement (PHP 7.0+)
    GroupUse(GroupUse),
    /// Declare directive (PHP 7.0+)
    /// declare(directive) or declare(directive) { ... }
    Declare {
        directives: Vec<DeclareDirective>,
        body: Option<Vec<Stmt>>, // None for file-scope
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
    #[allow(dead_code)] // Will be used for type validation
    pub type_hint: Option<TypeHint>,
    pub default: Option<Expr>,
    /// By-reference parameter (will be used when reference semantics are implemented)
    #[allow(dead_code)]
    pub by_ref: bool,
    /// Variadic parameter (...$param) collects remaining arguments
    pub is_variadic: bool,
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

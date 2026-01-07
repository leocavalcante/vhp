//! Interpreter module for VHP
//!
//! This module contains the tree-walking interpreter that executes
//! the AST produced by the parser.

pub mod builtins;
mod value;

// Submodules for organized implementation
mod expr_eval;
mod functions; // Function call handling (dispatcher, user functions)
mod objects;
mod stmt_exec;

pub use value::{ArrayKey, ExceptionValue, ObjectInstance, Value};

use crate::ast::{Expr, FunctionParam};
use std::collections::HashMap;
use std::io::{self, Write};

/// Namespace context for name resolution
#[derive(Debug, Clone)]
pub struct NamespaceContext {
    /// Current namespace (empty for global)
    pub current: Vec<String>,
    /// Use imports: alias -> fully qualified name
    pub class_imports: HashMap<String, crate::ast::QualifiedName>,
    pub function_imports: HashMap<String, crate::ast::QualifiedName>,
    pub constant_imports: HashMap<String, crate::ast::QualifiedName>,
}

impl NamespaceContext {
    pub fn new() -> Self {
        Self {
            current: vec![],
            class_imports: HashMap::new(),
            function_imports: HashMap::new(),
            constant_imports: HashMap::new(),
        }
    }

    /// Resolve a class name to fully qualified
    pub fn resolve_class(&self, name: &crate::ast::QualifiedName) -> String {
        // Already fully qualified
        if name.is_fully_qualified {
            return name.parts.join("\\");
        }

        // Single name - check imports first
        if name.parts.len() == 1 {
            let simple_name = &name.parts[0];
            if let Some(imported) = self.class_imports.get(simple_name) {
                return imported.parts.join("\\");
            }
        }

        // Relative name - prepend current namespace
        let mut full = self.current.clone();
        full.extend(name.parts.clone());
        full.join("\\")
    }

    // Note: Function resolution would work similar to class resolution
    // but with fallback to global namespace for built-in functions.
    // This is left for future implementation when namespaced functions are needed.

    /// Add a use import
    pub fn add_import(&mut self, item: &crate::ast::UseItem) {
        let alias = item
            .alias
            .clone()
            .unwrap_or_else(|| item.name.last().cloned().unwrap_or_default());

        match item.use_type {
            crate::ast::UseType::Class => {
                self.class_imports.insert(alias, item.name.clone());
            }
            crate::ast::UseType::Function => {
                self.function_imports.insert(alias, item.name.clone());
            }
            crate::ast::UseType::Constant => {
                self.constant_imports.insert(alias, item.name.clone());
            }
        }
    }
}

/// Control flow signals for break/continue/return/exception
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
    Exception(ExceptionValue),
}

/// User-defined function
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    #[allow(dead_code)] // Will be used for type validation
    pub return_type: Option<crate::ast::TypeHint>,
    pub body: Vec<crate::ast::Stmt>,
    pub is_abstract: bool, // for abstract methods
    pub is_final: bool,    // for final methods
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Class definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    pub is_abstract: bool, // abstract class modifier
    pub is_final: bool,    // final class modifier
    pub readonly: bool,    // PHP 8.2+: if true, all properties are implicitly readonly
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub interfaces: Vec<String>, // Implemented interfaces
    pub properties: Vec<crate::ast::Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

impl ClassDefinition {
    /// Find a magic method by name (case-insensitive)
    pub fn get_magic_method(&self, name: &str) -> Option<&UserFunction> {
        self.methods
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }

    /// Check if class has a magic method
    #[allow(dead_code)]
    pub fn has_magic_method(&self, name: &str) -> bool {
        self.get_magic_method(name).is_some()
    }
}

/// Interface definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    #[allow(dead_code)] // Will be used for interface validation
    pub name: String,
    #[allow(dead_code)] // Will be used for interface inheritance
    pub parents: Vec<String>,
    pub methods: Vec<(String, Vec<FunctionParam>)>, // (name, params)
    #[allow(dead_code)] // Will be used for interface constants
    pub constants: HashMap<String, Value>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Trait definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct TraitDefinition {
    #[allow(dead_code)] // Will be used for trait validation
    pub name: String,
    #[allow(dead_code)] // Will be used for trait composition
    pub uses: Vec<String>,
    pub properties: Vec<crate::ast::Property>,
    pub methods: HashMap<String, UserFunction>,
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Enum definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub backing_type: crate::ast::EnumBackingType,
    pub cases: Vec<(String, Option<Value>)>, // (case_name, optional_value)
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

pub struct Interpreter<W: Write> {
    output: W,
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
    classes: HashMap<String, ClassDefinition>,
    interfaces: HashMap<String, InterfaceDefinition>,
    traits: HashMap<String, TraitDefinition>,
    enums: HashMap<String, EnumDefinition>,
    current_object: Option<ObjectInstance>,
    current_class: Option<String>,

    // Static properties: class_name_lowercase -> property_name -> value
    // Initialized when class is defined, shared across all instances
    static_properties: HashMap<String, HashMap<String, Value>>,

    // Readonly static properties: class_name_lowercase -> property_name set
    static_readonly_properties: HashMap<String, std::collections::HashSet<String>>,

    // Called class context (for 'static' - late static binding)
    called_class: Option<String>,

    // Namespace support
    namespace_context: NamespaceContext,

    // Fiber support
    fibers: HashMap<usize, value::FiberInstance>, // All fibers by ID
    current_fiber: Option<usize>,                 // Currently executing fiber ID
    fiber_counter: usize,                         // For generating unique IDs

    // Anonymous class support
    anonymous_class_counter: usize, // For generating unique class names

    // Strict types mode (PHP 7.0+)
    strict_types: bool,              // Current strict_types mode
    strict_types_stack: Vec<bool>,   // Stack for block-scoped declare
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        let mut interp = Self {
            output,
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            current_object: None,
            current_class: None,
            static_properties: HashMap::new(),
            static_readonly_properties: HashMap::new(),
            called_class: None,
            namespace_context: NamespaceContext::new(),
            fibers: HashMap::new(),
            current_fiber: None,
            fiber_counter: 0,
            anonymous_class_counter: 0,
            strict_types: false,
            strict_types_stack: vec![],
        };
        // Register built-in Exception class
        interp.register_exception_class();
        interp
    }

    /// Register built-in Exception class
    fn register_exception_class(&mut self) {
        use crate::ast::{FunctionParam, Property, Stmt, Visibility};

        // Create constructor that sets message and code
        let constructor = UserFunction {
            params: vec![
                FunctionParam {
                    name: "message".to_string(),
                    type_hint: None,
                    default: Some(Expr::String(String::new())),
                    by_ref: false,
                    is_variadic: false,
                    visibility: None,
                    readonly: false,
                    attributes: vec![],
                },
                FunctionParam {
                    name: "code".to_string(),
                    type_hint: None,
                    default: Some(Expr::Integer(0)),
                    by_ref: false,
                    is_variadic: false,
                    visibility: None,
                    readonly: false,
                    attributes: vec![],
                },
            ],
            return_type: None,
            body: vec![
                // $this->message = $message;
                Stmt::Expression(Expr::PropertyAssign {
                    object: Box::new(Expr::This),
                    property: "message".to_string(),
                    value: Box::new(Expr::Variable("message".to_string())),
                }),
                // $this->code = $code;
                Stmt::Expression(Expr::PropertyAssign {
                    object: Box::new(Expr::This),
                    property: "code".to_string(),
                    value: Box::new(Expr::Variable("code".to_string())),
                }),
            ],
            is_abstract: false,
            is_final: false,
            attributes: vec![],
        };

        let mut methods = HashMap::new();
        methods.insert("__construct".to_string(), constructor);

        // Add getMessage() method
        let get_message = UserFunction {
            params: vec![],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::PropertyAccess {
                object: Box::new(Expr::This),
                property: "message".to_string(),
            }))],
            is_abstract: false,
            is_final: false,
            attributes: vec![],
        };
        methods.insert("getmessage".to_string(), get_message);

        // Add getCode() method
        let get_code = UserFunction {
            params: vec![],
            return_type: None,
            body: vec![Stmt::Return(Some(Expr::PropertyAccess {
                object: Box::new(Expr::This),
                property: "code".to_string(),
            }))],
            is_abstract: false,
            is_final: false,
            attributes: vec![],
        };
        methods.insert("getcode".to_string(), get_code);

        let mut method_visibility = HashMap::new();
        method_visibility.insert("__construct".to_string(), Visibility::Public);
        method_visibility.insert("getmessage".to_string(), Visibility::Public);
        method_visibility.insert("getcode".to_string(), Visibility::Public);

        let exception_class = ClassDefinition {
            name: "Exception".to_string(),
            is_abstract: false,
            is_final: false,
            readonly: false,
            parent: None,
            interfaces: Vec::new(),
            properties: vec![
                Property {
                    name: "message".to_string(),
                    visibility: Visibility::Protected,
                    write_visibility: None,
                    default: Some(Expr::String(String::new())),
                    readonly: false,
                    is_static: false,
                    attributes: vec![],
                    hooks: vec![],
                },
                Property {
                    name: "code".to_string(),
                    visibility: Visibility::Protected,
                    write_visibility: None,
                    default: Some(Expr::Integer(0)),
                    readonly: false,
                    is_static: false,
                    attributes: vec![],
                    hooks: vec![],
                },
            ],
            methods,
            method_visibility,
            attributes: vec![],
        };

        self.classes
            .insert("exception".to_string(), exception_class);
    }

    // Fiber management methods

    /// Create a new Fiber instance
    fn eval_new_fiber(&mut self, callback_expr: &Expr) -> Result<Value, String> {
        // Evaluate callback expression to get function
        let callback_function = match callback_expr {
            Expr::Variable(name) => {
                // Look up function by name (strip $ prefix if present)
                let func_name = if let Some(stripped) = name.strip_prefix('$') {
                    stripped
                } else {
                    name
                };
                self.functions
                    .get(func_name)
                    .cloned()
                    .ok_or_else(|| format!("Function '{}' not found", func_name))?
            }
            Expr::String(name) => {
                // Direct function name as string literal
                self.functions
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("Function '{}' not found", name))?
            }
            _ => return Err("Fiber callback must be a function name".to_string()),
        };

        // Generate unique fiber ID
        self.fiber_counter += 1;
        let fiber_id = self.fiber_counter;

        // Create fiber instance
        let fiber = value::FiberInstance {
            id: fiber_id,
            state: value::FiberState::NotStarted,
            callback: Some(callback_function),
            call_stack: Vec::new(),
            variables: HashMap::new(),
            suspended_value: None,
            return_value: None,
            error: None,
        };

        // Store fiber
        self.fibers.insert(fiber_id, fiber.clone());

        Ok(Value::Fiber(Box::new(fiber)))
    }

    /// Suspend current fiber with optional value
    fn eval_fiber_suspend(&mut self, value_expr: Option<&Expr>) -> Result<Value, String> {
        // Get current fiber ID
        let fiber_id = self
            .current_fiber
            .ok_or("Fiber::suspend() called outside of fiber")?;

        // Evaluate suspend value
        let suspend_value = if let Some(expr) = value_expr {
            self.eval_expr(expr)?
        } else {
            Value::Null
        };

        // Update fiber state
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.state = value::FiberState::Suspended;
            fiber.suspended_value = Some(Box::new(suspend_value.clone()));
        }

        // Return the suspend value (this is what start()/resume() will return)
        Ok(suspend_value)
    }

    /// Get currently executing fiber
    fn eval_fiber_get_current(&self) -> Value {
        match self.current_fiber {
            Some(fiber_id) => {
                if let Some(fiber) = self.fibers.get(&fiber_id) {
                    Value::Fiber(Box::new(fiber.clone()))
                } else {
                    Value::Null
                }
            }
            None => Value::Null,
        }
    }

    /// Start fiber execution
    pub fn fiber_start(&mut self, fiber_id: usize, args: Vec<Value>) -> Result<Value, String> {
        // Get fiber and validate state
        let fiber = self.fibers.get(&fiber_id).ok_or("Invalid fiber ID")?;

        if fiber.state != value::FiberState::NotStarted {
            return Err("Fiber has already been started".to_string());
        }

        // Set current fiber context
        let previous_fiber = self.current_fiber;
        self.current_fiber = Some(fiber_id);

        // Execute fiber function
        let result = self.execute_fiber_function(fiber_id, args);

        // Restore previous fiber context
        self.current_fiber = previous_fiber;

        result
    }

    /// Resume fiber execution
    pub fn fiber_resume(&mut self, fiber_id: usize, value: Value) -> Result<Value, String> {
        // Get fiber and validate state
        let fiber = self.fibers.get(&fiber_id).ok_or("Invalid fiber ID")?;

        if fiber.state != value::FiberState::Suspended {
            return Err("Fiber is not suspended".to_string());
        }

        // Set current fiber context
        let previous_fiber = self.current_fiber;
        self.current_fiber = Some(fiber_id);

        // Resume from suspension point with provided value
        let result = self.resume_fiber_from_suspension(fiber_id, value);

        // Restore previous fiber context
        self.current_fiber = previous_fiber;

        result
    }

    /// Execute fiber function from beginning
    fn execute_fiber_function(
        &mut self,
        fiber_id: usize,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        let callback = {
            let fiber = self.fibers.get(&fiber_id).unwrap();
            fiber.callback.as_ref().unwrap().clone()
        };

        // Update fiber state to running
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.state = value::FiberState::Running;
        }

        // Execute function body with fiber context
        self.execute_function_in_fiber(fiber_id, &callback, args)
    }

    /// Resume fiber from suspension
    fn resume_fiber_from_suspension(
        &mut self,
        fiber_id: usize,
        resume_value: Value,
    ) -> Result<Value, String> {
        // Update fiber state to running
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.state = value::FiberState::Running;
        }

        // Continue execution where it left off
        // This is simplified - in a real implementation, we'd need to save/restore call stack
        self.continue_fiber_execution(fiber_id, resume_value)
    }

    /// Execute function within fiber context
    fn execute_function_in_fiber(
        &mut self,
        fiber_id: usize,
        function: &UserFunction,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        // Save current variables
        let saved_vars = self.variables.clone();

        // Set up function parameters
        for (i, param) in function.params.iter().enumerate() {
            let value = args.get(i).cloned().unwrap_or(Value::Null);
            self.variables.insert(param.name.clone(), value);
        }

        // Execute function body
        let mut return_value = Value::Null;
        for stmt in &function.body {
            match self.execute_stmt(stmt).map_err(|e| e.to_string())? {
                ControlFlow::Return(val) => {
                    return_value = val;
                    break;
                }
                ControlFlow::Break | ControlFlow::Continue => {
                    return Err("break/continue outside of loop in fiber".to_string());
                }
                ControlFlow::Exception(e) => {
                    return Err(format!("__EXCEPTION__:{}:{}", e.class_name, e.message));
                }
                ControlFlow::None => {}
            }
        }

        // Mark fiber as terminated
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.state = value::FiberState::Terminated;
            fiber.return_value = Some(Box::new(return_value.clone()));
        }

        // Restore variables
        self.variables = saved_vars;

        Ok(return_value)
    }

    /// Continue fiber execution after suspension
    fn continue_fiber_execution(
        &mut self,
        _fiber_id: usize,
        resume_value: Value,
    ) -> Result<Value, String> {
        // Simplified implementation - return the resume value
        // In a full implementation, we'd restore the exact execution context
        Ok(resume_value)
    }

    /// Check if an object is an instance of a given class or interface (with full hierarchy checking)
    /// This method properly traverses the full inheritance chain including:
    /// - Direct class match
    /// - Parent class chain (recursively)
    /// - Implemented interfaces (recursively)
    /// - Interface inheritance (recursively)
    pub fn is_instance_of(&self, object_class: &str, target_class: &str) -> bool {
        let object_class_lower = object_class.to_lowercase();
        let target_class_lower = target_class.to_lowercase();

        // Check exact match (case-insensitive)
        if object_class_lower == target_class_lower {
            return true;
        }

        // Check if object_class is a class
        if let Some(class_def) = self.classes.get(&object_class_lower) {
            // Check parent chain recursively
            if let Some(parent) = &class_def.parent {
                if self.is_instance_of(parent, target_class) {
                    return true;
                }
            }

            // Check implemented interfaces recursively
            for interface in &class_def.interfaces {
                if self.is_instance_of(interface, target_class) {
                    return true;
                }
            }
        }

        // Check if object_class is an interface extending other interfaces
        if let Some(interface_def) = self.interfaces.get(&object_class_lower) {
            for parent_interface in &interface_def.parents {
                if self.is_instance_of(parent_interface, target_class) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a value matches a type hint (with full interpreter access for hierarchy checking)
    /// This is the proper version that handles inheritance correctly.
    pub fn value_matches_type(&self, value: &Value, type_hint: &crate::ast::TypeHint) -> bool {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type(value, inner)
            }
            TypeHint::Union(types) => types.iter().any(|t| self.value_matches_type(value, t)),
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.value_matches_type(value, t))
            }
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                intersections.iter().any(|group| {
                    // All types in the group must match
                    group.iter().all(|t| self.value_matches_type(value, t))
                })
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,       // void is for return types only
            TypeHint::Never => false,      // never is for return types only
            TypeHint::Static => false,     // Requires class context
            TypeHint::SelfType => false,   // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }

    /// Helper to check simple type matches
    fn value_matches_simple_type(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            ("string", Value::String(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            ("bool", Value::Bool(_)) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("mixed", _) => true,
            ("null", Value::Null) => true,
            ("false", Value::Bool(false)) => true,
            ("true", Value::Bool(true)) => true,
            _ => false,
        }
    }

    /// Convert value to string, calling __toString if available
    pub(super) fn value_to_string(&mut self, value: &Value) -> Result<String, String> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => {
                if f.fract() == 0.0 && f.abs() < 1e15 {
                    Ok(format!("{:.0}", f))
                } else {
                    Ok(f.to_string())
                }
            }
            Value::Bool(true) => Ok("1".to_string()),
            Value::Bool(false) => Ok(String::new()),
            Value::Null => Ok(String::new()),
            Value::Array(_) => Err("Array to string conversion".to_string()),
            Value::Object(obj) => {
                // Check for __toString magic method
                let class = self.classes.get(&obj.class_name.to_lowercase()).cloned();
                if let Some(class) = class {
                    if let Some(method) = class.get_magic_method("__toString") {
                        let class_name = obj.class_name.clone();
                        let mut obj_mut = obj.clone();
                        let result =
                            self.call_method_on_object(&mut obj_mut, method, &[], class_name)?;
                        if let Value::String(s) = result {
                            Ok(s)
                        } else {
                            Err(format!(
                                "{}::__toString() must return a string value",
                                obj.class_name
                            ))
                        }
                    } else {
                        Err(format!(
                            "Object of class {} could not be converted to string",
                            obj.class_name
                        ))
                    }
                } else {
                    Err(format!("Class {} not found", obj.class_name))
                }
            }
            Value::Fiber(fiber) => Ok(format!("Object(Fiber#{:06})", fiber.id)),
            Value::Closure(_) => Ok("Object(Closure)".to_string()),
            Value::EnumCase {
                enum_name,
                case_name,
                ..
            } => Ok(format!("{}::{}", enum_name, case_name)),
            Value::Exception(exc) => Ok(format!("Object({})", exc.class_name)),
        }
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

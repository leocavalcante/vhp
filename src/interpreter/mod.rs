//! Interpreter module for VHP
//!
//! This module contains the tree-walking interpreter that executes
//! the AST produced by the parser.

mod builtins;
mod value;

// Submodules for organized implementation
mod expr_eval;
mod functions; // Function call handling (dispatcher, user functions)
mod objects;
mod stmt_exec;

pub use value::{ObjectInstance, Value};

use crate::ast::{FunctionParam, Expr};
use std::collections::HashMap;
use std::io::{self, Write};

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    pub body: Vec<crate::ast::Stmt>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
}

/// Class definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    pub readonly: bool, // PHP 8.2+: if true, all properties are implicitly readonly
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub properties: Vec<crate::ast::Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, crate::ast::Visibility>,
    #[allow(dead_code)] // Will be used for reflection
    pub attributes: Vec<crate::ast::Attribute>,
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
    
    // Fiber support
    fibers: HashMap<usize, value::FiberInstance>, // All fibers by ID
    current_fiber: Option<usize>,                 // Currently executing fiber ID
    fiber_counter: usize,                         // For generating unique IDs
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            current_object: None,
            current_class: None,
            fibers: HashMap::new(),
            current_fiber: None,
            fiber_counter: 0,
        }
    }

    // Fiber management methods
    
    /// Create a new Fiber instance
    fn eval_new_fiber(&mut self, callback_expr: &Expr) -> Result<Value, String> {
        // Evaluate callback expression to get function
        let callback_function = match callback_expr {
            Expr::Variable(name) => {
                // Look up function by name (strip $ prefix if present)
                let func_name = if name.starts_with('$') {
                    &name[1..]
                } else {
                    name
                };
                self.functions.get(func_name)
                    .cloned()
                    .ok_or_else(|| format!("Function '{}' not found", func_name))?
            }
            Expr::String(name) => {
                // Direct function name as string literal
                self.functions.get(name)
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
        let fiber_id = self.current_fiber.ok_or("Fiber::suspend() called outside of fiber")?;
        
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
        let fiber = self.fibers.get(&fiber_id)
            .ok_or("Invalid fiber ID")?;
        
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
        let fiber = self.fibers.get(&fiber_id)
            .ok_or("Invalid fiber ID")?;
        
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
    fn execute_fiber_function(&mut self, fiber_id: usize, args: Vec<Value>) -> Result<Value, String> {
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
    fn resume_fiber_from_suspension(&mut self, fiber_id: usize, resume_value: Value) -> Result<Value, String> {
        // Update fiber state to running
        if let Some(fiber) = self.fibers.get_mut(&fiber_id) {
            fiber.state = value::FiberState::Running;
        }
        
        // Continue execution where it left off
        // This is simplified - in a real implementation, we'd need to save/restore call stack
        self.continue_fiber_execution(fiber_id, resume_value)
    }
    
    /// Execute function within fiber context
    fn execute_function_in_fiber(&mut self, fiber_id: usize, function: &UserFunction, args: Vec<Value>) -> Result<Value, String> {
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
    fn continue_fiber_execution(&mut self, _fiber_id: usize, resume_value: Value) -> Result<Value, String> {
        // Simplified implementation - return the resume value
        // In a full implementation, we'd restore the exact execution context
        Ok(resume_value)
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

//! Function call handling for the interpreter
//!
//! This module handles:
//! - Built-in function dispatch
//! - User-defined function calls
//! - Named argument support (PHP 8.0)

mod dispatch;

use crate::ast::Argument;
use crate::interpreter::value::{ArrayKey, Value};
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Call a function with Argument nodes (handles both built-in and user-defined)
    pub(super) fn call_function(&mut self, name: &str, args: &[Argument]) -> Result<Value, String> {
        // Evaluate arguments, handling spread expressions
        let mut arg_values = Vec::new();
        for arg in args {
            if let crate::ast::Expr::Spread(inner) = arg.value.as_ref() {
                // Spread: unpack array into multiple arguments
                let value = self.eval_expr(inner)?;
                match value {
                    Value::Array(arr) => {
                        // Flatten array values into arguments
                        for (_, v) in arr {
                            arg_values.push(v);
                        }
                    }
                    _ => return Err("Cannot unpack non-array value".to_string()),
                }
            } else {
                arg_values.push(self.eval_expr(&arg.value)?);
            }
        }

        // Try to dispatch built-in function first (case-insensitive)
        let lower_name = name.to_lowercase();
        match self.dispatch_builtin(&lower_name, &arg_values) {
            Ok(val) => return Ok(val),
            Err(e) => {
                // If it's an "unknown built-in" error, try user-defined functions
                if e.contains("Unknown built-in function") {
                    // Fall through to user function lookup
                } else {
                    // Other errors should be propagated (e.g., wrong argument count)
                    return Err(e);
                }
            }
        }

        // Look up in user-defined functions (case-insensitive)
        let func = self
            .functions
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower_name)
            .map(|(_, v)| v.clone());

        if let Some(func) = func {
            self.call_user_function_with_arguments(&func, args)
        } else {
            Err(format!("Call to undefined function {}()", name))
        }
    }

    /// Call user-defined function with pre-evaluated values
    #[allow(dead_code)]
    pub(super) fn call_user_function(
        &mut self,
        func: &crate::interpreter::UserFunction,
        args: &[Value],
    ) -> Result<Value, String> {
        // Save current variables (for scoping)
        let saved_variables = self.variables.clone();
        // Clear current class context for global functions
        let saved_current_class = self.current_class.take();

        // Bind parameters
        for (arg_idx, param) in func.params.iter().enumerate() {
            if param.is_variadic {
                // Variadic parameter: collect all remaining arguments into an array
                let remaining: Vec<Value> = args[arg_idx..].to_vec();
                let arr: Vec<(ArrayKey, Value)> = remaining
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                    .collect();
                self.variables.insert(param.name.clone(), Value::Array(arr));
                break; // Variadic must be last
            }

            let value = if arg_idx < args.len() {
                args[arg_idx].clone()
            } else if let Some(default) = &param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!(
                    "Missing argument {} for parameter ${}",
                    arg_idx + 1,
                    param.name
                ));
            };
            self.variables.insert(param.name.clone(), value);
        }

        // Execute function body
        let mut return_value = Value::Null;
        for stmt in &func.body.clone() {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            if let crate::interpreter::ControlFlow::Return(val) = cf {
                return_value = val;
                break;
            }
        }

        // Restore variables and class context
        self.variables = saved_variables;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Call user-defined function with support for named arguments (PHP 8.0) and variadic params
    pub(super) fn call_user_function_with_arguments(
        &mut self,
        func: &crate::interpreter::UserFunction,
        args: &[Argument],
    ) -> Result<Value, String> {
        // Save current variables (for scoping)
        let saved_variables = self.variables.clone();
        // Clear current class context for global functions
        let saved_current_class = self.current_class.take();

        // First, evaluate all argument values, handling spread expressions
        let mut arg_values = Vec::new();
        let mut named_args: HashMap<String, Value> = HashMap::new();
        
        for arg in args {
            if let Some(ref name) = arg.name {
                // Named argument
                let value = self.eval_expr(&arg.value)?;
                named_args.insert(name.clone(), value);
            } else if let crate::ast::Expr::Spread(inner) = arg.value.as_ref() {
                // Spread: unpack array into positional arguments
                let value = self.eval_expr(inner)?;
                match value {
                    Value::Array(arr) => {
                        for (_, v) in arr {
                            arg_values.push(v);
                        }
                    }
                    _ => return Err("Cannot unpack non-array value".to_string()),
                }
            } else {
                // Positional argument
                arg_values.push(self.eval_expr(&arg.value)?);
            }
        }

        // Bind parameters
        let mut positional_arg_idx = 0;
        for param in &func.params {
            if param.is_variadic {
                // Variadic parameter: collect all remaining positional arguments into an array
                let remaining: Vec<Value> = arg_values[positional_arg_idx..].to_vec();
                let arr: Vec<(ArrayKey, Value)> = remaining
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                    .collect();
                self.variables.insert(param.name.clone(), Value::Array(arr));
                break; // Variadic must be last
            }

            let value = if let Some(named_value) = named_args.get(&param.name) {
                // Named argument matched
                named_value.clone()
            } else if positional_arg_idx < arg_values.len() {
                // Use positional argument
                let v = arg_values[positional_arg_idx].clone();
                positional_arg_idx += 1;
                v
            } else if let Some(default) = &param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!(
                    "Missing required argument for parameter ${}",
                    param.name
                ));
            };

            self.variables.insert(param.name.clone(), value);
        }

        // Check for unknown named arguments
        for name in named_args.keys() {
            if !func.params.iter().any(|p| p.name == *name) {
                return Err(format!("Unknown named parameter ${}", name));
            }
        }

        // Execute function body
        let mut return_value = Value::Null;
        for stmt in &func.body.clone() {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            if let crate::interpreter::ControlFlow::Return(val) = cf {
                return_value = val;
                break;
            }
        }

        // Restore variables and class context
        self.variables = saved_variables;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Helper to call a function with pre-evaluated argument values
    pub(super) fn call_function_with_values(
        &mut self,
        name: &str,
        arg_values: &[Value],
    ) -> Result<Value, String> {
        // Try to dispatch built-in function first (case-insensitive)
        let lower_name = name.to_lowercase();
        match self.dispatch_builtin(&lower_name, arg_values) {
            Ok(val) => return Ok(val),
            Err(e) => {
                // If it's an "unknown built-in" error, try user-defined functions
                if e.contains("Unknown built-in function") {
                    // Fall through to user function lookup
                } else {
                    // Other errors should be propagated
                    return Err(e);
                }
            }
        }

        // Check for user-defined functions (case-insensitive)
        let func = self
            .functions
            .iter()
            .find(|(k, _)| k.to_lowercase() == lower_name)
            .map(|(_, v)| v.clone());

        if let Some(func) = func {
            self.call_user_function(&func, arg_values)
        } else {
            Err(format!("Undefined function: {}", name))
        }
    }

    /// Call a closure (arrow function or anonymous function)
    pub(super) fn call_closure(
        &mut self,
        closure: &crate::interpreter::value::Closure,
        args: &[Argument],
    ) -> Result<Value, String> {
        use crate::interpreter::value::ClosureBody;

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            if let crate::ast::Expr::Spread(inner) = arg.value.as_ref() {
                // Spread: unpack array into multiple arguments
                let value = self.eval_expr(inner)?;
                match value {
                    Value::Array(arr) => {
                        for (_, v) in arr {
                            arg_values.push(v);
                        }
                    }
                    _ => return Err("Cannot unpack non-array value".to_string()),
                }
            } else {
                arg_values.push(self.eval_expr(&arg.value)?);
            }
        }

        // Save current variables
        let saved_vars = self.variables.clone();

        // Set up new scope with captured variables
        self.variables = closure.captured_vars.clone();

        // Bind parameters
        for (i, param) in closure.params.iter().enumerate() {
            let value = if i < arg_values.len() {
                arg_values[i].clone()
            } else if let Some(ref default) = param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!("Missing argument for parameter ${}", param.name));
            };
            self.variables.insert(param.name.clone(), value);
        }

        // Execute closure body
        let result = match &closure.body {
            ClosureBody::Expression(expr) => self.eval_expr(expr)?,
            ClosureBody::FunctionRef(name) => {
                // Call the referenced function with pre-evaluated values
                self.call_function_with_values(name, &arg_values)?
            }
            ClosureBody::MethodRef { .. } => {
                return Err("Method callables not yet fully implemented".to_string());
            }
            ClosureBody::StaticMethodRef { .. } => {
                return Err("Static method callables not yet fully implemented".to_string());
            }
        };

        // Restore variables
        self.variables = saved_vars;

        Ok(result)
    }
}

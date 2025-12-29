//! Function call handling for the interpreter
//!
//! This module handles:
//! - Built-in function dispatch
//! - User-defined function calls
//! - Named argument support (PHP 8.0)

mod dispatch;

use crate::ast::Argument;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Call a function with Argument nodes (handles both built-in and user-defined)
    pub(super) fn call_function(&mut self, name: &str, args: &[Argument]) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
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
        for (i, param) in func.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default) = &param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!(
                    "Missing argument {} for parameter ${}",
                    i + 1,
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

    /// Call user-defined function with support for named arguments (PHP 8.0)
    pub(super) fn call_user_function_with_arguments(
        &mut self,
        func: &crate::interpreter::UserFunction,
        args: &[Argument],
    ) -> Result<Value, String> {
        // Save current variables (for scoping)
        let saved_variables = self.variables.clone();
        // Clear current class context for global functions
        let saved_current_class = self.current_class.take();

        // First, evaluate all argument values
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
        }

        // Build a map of named arguments for quick lookup
        let mut named_args: HashMap<String, Value> = HashMap::new();
        let mut positional_idx = 0;

        for (i, arg) in args.iter().enumerate() {
            if let Some(ref name) = arg.name {
                // Named argument: validate that we haven't used positional args after named
                if positional_idx < i {
                    // We have positional args before named args - this is allowed
                }
                named_args.insert(name.clone(), arg_values[i].clone());
            } else {
                // Positional argument
                positional_idx = i + 1;
            }
        }

        // Bind parameters
        let mut positional_arg_idx = 0;
        for param in &func.params {
            let value = if let Some(named_value) = named_args.get(&param.name) {
                // Named argument matched
                named_value.clone()
            } else if positional_arg_idx < positional_idx {
                // Use positional argument
                arg_values[positional_arg_idx].clone()
            } else if let Some(default) = &param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!(
                    "Missing required argument for parameter ${}",
                    param.name
                ));
            };

            if positional_arg_idx < positional_idx {
                positional_arg_idx += 1;
            }

            self.variables.insert(param.name.clone(), value);
        }

        // Check for unknown named arguments
        for arg in args {
            if let Some(ref name) = arg.name {
                if !func.params.iter().any(|p| p.name == *name) {
                    return Err(format!("Unknown named parameter ${}", name));
                }
            }
        }

        // Check for duplicate arguments (both positional and named for same param)
        for arg in args {
            if let Some(ref name) = arg.name {
                // Check if this parameter was already provided positionally
                if positional_arg_idx > 0 {
                    if let Some(param) = func.params.get(positional_arg_idx - 1) {
                        if param.name == *name {
                            return Err(format!(
                                "Cannot use positional argument after named argument for parameter ${}",
                                name
                            ));
                        }
                    }
                }
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
}

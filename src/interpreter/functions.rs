//! Function call handling for the interpreter
//!
//! This module handles:
//! - Built-in function dispatch
//! - User-defined function calls
//! - Named argument support (PHP 8.0)
//! - Variable assignment and compound assignments

use crate::ast::Argument;
use crate::interpreter::builtins;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    pub(super) fn call_function(&mut self, name: &str, args: &[Argument]) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
        }

        // Check for built-in functions first (case-insensitive)
        let lower_name = name.to_lowercase();
        match lower_name.as_str() {
            // String functions
            "strlen" => builtins::string::strlen(&arg_values),
            "substr" => builtins::string::substr(&arg_values),
            "strtoupper" => builtins::string::strtoupper(&arg_values),
            "strtolower" => builtins::string::strtolower(&arg_values),
            "trim" => builtins::string::trim(&arg_values),
            "ltrim" => builtins::string::ltrim(&arg_values),
            "rtrim" => builtins::string::rtrim(&arg_values),
            "str_repeat" => builtins::string::str_repeat(&arg_values),
            "str_replace" => builtins::string::str_replace(&arg_values),
            "strpos" => builtins::string::strpos(&arg_values),
            "str_contains" => builtins::string::str_contains(&arg_values),
            "str_starts_with" => builtins::string::str_starts_with(&arg_values),
            "str_ends_with" => builtins::string::str_ends_with(&arg_values),
            "ucfirst" => builtins::string::ucfirst(&arg_values),
            "lcfirst" => builtins::string::lcfirst(&arg_values),
            "ucwords" => builtins::string::ucwords(&arg_values),
            "strrev" => builtins::string::strrev(&arg_values),
            "str_pad" => builtins::string::str_pad(&arg_values),
            "explode" => builtins::string::explode(&arg_values),
            "implode" | "join" => builtins::string::implode(&arg_values),
            "sprintf" => builtins::string::sprintf(&arg_values),
            "printf" => builtins::output::printf(&mut self.output, &arg_values),
            "chr" => builtins::string::chr(&arg_values),
            "ord" => builtins::string::ord(&arg_values),

            // Math functions
            "abs" => builtins::math::abs(&arg_values),
            "ceil" => builtins::math::ceil(&arg_values),
            "floor" => builtins::math::floor(&arg_values),
            "round" => builtins::math::round(&arg_values),
            "max" => builtins::math::max(&arg_values),
            "min" => builtins::math::min(&arg_values),
            "pow" => builtins::math::pow(&arg_values),
            "sqrt" => builtins::math::sqrt(&arg_values),
            "rand" | "mt_rand" => builtins::math::rand(&arg_values),

            // Type functions
            "intval" => builtins::types::intval(&arg_values),
            "floatval" | "doubleval" => builtins::types::floatval(&arg_values),
            "strval" => builtins::types::strval(&arg_values),
            "boolval" => builtins::types::boolval(&arg_values),
            "gettype" => builtins::types::gettype(&arg_values),
            "is_null" => builtins::types::is_null(&arg_values),
            "is_bool" => builtins::types::is_bool(&arg_values),
            "is_int" | "is_integer" | "is_long" => builtins::types::is_int(&arg_values),
            "is_float" | "is_double" | "is_real" => builtins::types::is_float(&arg_values),
            "is_string" => builtins::types::is_string(&arg_values),
            "is_array" => builtins::types::is_array(&arg_values),
            "is_numeric" => builtins::types::is_numeric(&arg_values),
            "isset" => builtins::types::isset(&arg_values),
            "empty" => builtins::types::empty(&arg_values),

            // Output functions
            "print" => builtins::output::print(&mut self.output, &arg_values),
            "var_dump" => builtins::output::var_dump(&mut self.output, &arg_values),
            "print_r" => builtins::output::print_r(&mut self.output, &arg_values),

            // Array functions
            "count" | "sizeof" => builtins::array::count(&arg_values),
            "array_push" => builtins::array::array_push(&arg_values),
            "array_pop" => builtins::array::array_pop(&arg_values),
            "array_shift" => builtins::array::array_shift(&arg_values),
            "array_unshift" => builtins::array::array_unshift(&arg_values),
            "array_keys" => builtins::array::array_keys(&arg_values),
            "array_values" => builtins::array::array_values(&arg_values),
            "in_array" => builtins::array::in_array(&arg_values),
            "array_search" => builtins::array::array_search(&arg_values),
            "array_reverse" => builtins::array::array_reverse(&arg_values),
            "array_merge" => builtins::array::array_merge(&arg_values),
            "array_key_exists" => builtins::array::array_key_exists(&arg_values),
            "range" => builtins::array::range(&arg_values),

            // Reflection functions (PHP 8.0 attributes)
            "get_class_attributes" => {
                builtins::reflection::get_class_attributes(&arg_values, &self.classes)
            }
            "get_method_attributes" => {
                builtins::reflection::get_method_attributes(&arg_values, &self.classes)
            }
            "get_property_attributes" => {
                builtins::reflection::get_property_attributes(&arg_values, &self.classes)
            }
            "get_function_attributes" => {
                builtins::reflection::get_function_attributes(&arg_values, &self.functions)
            }
            "get_parameter_attributes" => {
                builtins::reflection::get_parameter_attributes(&arg_values, &self.functions)
            }
            "get_method_parameter_attributes" => {
                builtins::reflection::get_method_parameter_attributes(&arg_values, &self.classes)
            }
            "get_interface_attributes" => {
                builtins::reflection::get_interface_attributes(&arg_values, &self.interfaces)
            }
            "get_trait_attributes" => {
                builtins::reflection::get_trait_attributes(&arg_values, &self.traits)
            }

            // User-defined function
            _ => {
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
        }
    }

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
        // Check for built-in functions first (case-insensitive)
        let lower_name = name.to_lowercase();
        match lower_name.as_str() {
            // String functions
            "strlen" => builtins::string::strlen(arg_values),
            "substr" => builtins::string::substr(arg_values),
            "strtoupper" => builtins::string::strtoupper(arg_values),
            "strtolower" => builtins::string::strtolower(arg_values),
            "trim" => builtins::string::trim(arg_values),
            "ltrim" => builtins::string::ltrim(arg_values),
            "rtrim" => builtins::string::rtrim(arg_values),
            "str_repeat" => builtins::string::str_repeat(arg_values),
            "str_replace" => builtins::string::str_replace(arg_values),
            "strpos" => builtins::string::strpos(arg_values),
            "str_contains" => builtins::string::str_contains(arg_values),
            "str_starts_with" => builtins::string::str_starts_with(arg_values),
            "str_ends_with" => builtins::string::str_ends_with(arg_values),
            "ucfirst" => builtins::string::ucfirst(arg_values),
            "lcfirst" => builtins::string::lcfirst(arg_values),
            "ucwords" => builtins::string::ucwords(arg_values),
            "strrev" => builtins::string::strrev(arg_values),
            "str_pad" => builtins::string::str_pad(arg_values),
            "explode" => builtins::string::explode(arg_values),
            "implode" | "join" => builtins::string::implode(arg_values),
            "sprintf" => builtins::string::sprintf(arg_values),
            "printf" => builtins::output::printf(&mut self.output, arg_values),
            "chr" => builtins::string::chr(arg_values),
            "ord" => builtins::string::ord(arg_values),

            // Math functions
            "abs" => builtins::math::abs(arg_values),
            "ceil" => builtins::math::ceil(arg_values),
            "floor" => builtins::math::floor(arg_values),
            "round" => builtins::math::round(arg_values),
            "max" => builtins::math::max(arg_values),
            "min" => builtins::math::min(arg_values),
            "pow" => builtins::math::pow(arg_values),
            "sqrt" => builtins::math::sqrt(arg_values),
            "rand" | "mt_rand" => builtins::math::rand(arg_values),

            // Type functions
            "intval" => builtins::types::intval(arg_values),
            "floatval" | "doubleval" => builtins::types::floatval(arg_values),
            "strval" => builtins::types::strval(arg_values),
            "boolval" => builtins::types::boolval(arg_values),
            "gettype" => builtins::types::gettype(arg_values),
            "is_null" => builtins::types::is_null(arg_values),
            "is_bool" => builtins::types::is_bool(arg_values),
            "is_int" | "is_integer" | "is_long" => builtins::types::is_int(arg_values),
            "is_float" | "is_double" | "is_real" => builtins::types::is_float(arg_values),
            "is_string" => builtins::types::is_string(arg_values),
            "is_array" => builtins::types::is_array(arg_values),
            "is_numeric" => builtins::types::is_numeric(arg_values),
            "isset" => builtins::types::isset(arg_values),
            "empty" => builtins::types::empty(arg_values),

            // Output functions
            "print" => builtins::output::print(&mut self.output, arg_values),
            "var_dump" => builtins::output::var_dump(&mut self.output, arg_values),
            "print_r" => builtins::output::print_r(&mut self.output, arg_values),

            // Array functions
            "count" | "sizeof" => builtins::array::count(arg_values),
            "array_push" => builtins::array::array_push(arg_values),
            "array_pop" => builtins::array::array_pop(arg_values),
            "array_shift" => builtins::array::array_shift(arg_values),
            "array_unshift" => builtins::array::array_unshift(arg_values),
            "array_keys" => builtins::array::array_keys(arg_values),
            "array_values" => builtins::array::array_values(arg_values),
            "in_array" => builtins::array::in_array(arg_values),
            "array_search" => builtins::array::array_search(arg_values),
            "array_reverse" => builtins::array::array_reverse(arg_values),
            "array_merge" => builtins::array::array_merge(arg_values),
            "array_key_exists" => builtins::array::array_key_exists(arg_values),
            "range" => builtins::array::range(arg_values),

            _ => {
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
    }
}

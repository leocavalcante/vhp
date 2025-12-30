//! Method dispatch and invocation
//!
//! Handles:
//! - Instance method calls ($obj->method())
//! - Static method calls (ClassName::method())
//! - Method resolution in class hierarchy
//! - Named arguments (PHP 8.0)
//! - Enum built-in methods (cases, from, tryFrom)
//! - Setting up $this context for instance methods
//! - Setting up current_class context for static methods

use crate::ast::Argument;
use crate::interpreter::value::ObjectInstance;
use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Evaluate instance method call ($obj->method(...))
    ///
    /// Looks up the method in the class hierarchy and invokes it
    /// with the object as $this context. Updates the object after
    /// the call if it was modified.
    pub(crate) fn eval_method_call(
        &mut self,
        object: &crate::ast::Expr,
        method: &str,
        args: &[Argument],
    ) -> Result<Value, String> {
        // Get the variable name if object is a variable, so we can update it after the method call
        let var_name = match object {
            crate::ast::Expr::Variable(name) => Some(name.clone()),
            _ => None,
        };

        let obj_value = self.eval_expr(object)?;

        match obj_value {
            Value::Fiber(fiber) => {
                // Handle Fiber method calls
                self.call_fiber_method(fiber.id, method, args)
            }
            Value::Exception(ref exc) => {
                // Handle Exception method calls
                // Exceptions have built-in methods: getMessage(), getCode(), etc.
                match method.to_lowercase().as_str() {
                    "getmessage" => {
                        if !args.is_empty() {
                            return Err("getMessage() expects exactly 0 parameters".to_string());
                        }
                        Ok(Value::String(exc.message.clone()))
                    }
                    "getcode" => {
                        if !args.is_empty() {
                            return Err("getCode() expects exactly 0 parameters".to_string());
                        }
                        Ok(Value::Integer(exc.code))
                    }
                    _ => Err(format!(
                        "Call to undefined method {}::{}()",
                        exc.class_name, method
                    )),
                }
            }
            Value::Object(mut instance) => {
                let class_name = instance.class_name.clone();

                // Look up method in hierarchy
                let (method_func, declaring_class) =
                    self.find_method(&class_name, method).ok_or_else(|| {
                        format!("Call to undefined method {}::{}()", class_name, method)
                    })?;

                // Call method with $this bound and named argument support
                let result = self.call_method_on_object_with_arguments(
                    &mut instance,
                    &method_func,
                    args,
                    declaring_class,
                )?;

                // Write back the modified instance to the variable if applicable
                if let Some(name) = var_name {
                    self.variables.insert(name, Value::Object(instance));
                }

                Ok(result)
            }
            _ => Err(format!(
                "Cannot call method on non-object ({})",
                obj_value.get_type()
            )),
        }
    }

    /// Evaluate static method call (ClassName::method(...))
    ///
    /// Supports special class names: 'self', 'parent', and user-defined classes.
    /// Handles both user-defined methods and built-in enum methods (cases, from, tryFrom).
    pub(crate) fn eval_static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Argument],
    ) -> Result<Value, String> {
        let class_name_lower = class_name.to_lowercase();

        let target_class = if class_name_lower == "parent" {
            if let Some(current_class_name) = &self.current_class {
                let current_class_def = self
                    .classes
                    .get(&current_class_name.to_lowercase())
                    .unwrap();
                if let Some(parent) = &current_class_def.parent {
                    parent.clone()
                } else {
                    return Err(format!("Class '{}' has no parent", current_class_name));
                }
            } else {
                return Err("Cannot use 'parent' outside of class context".to_string());
            }
        } else if class_name_lower == "self" {
            if let Some(current_class_name) = &self.current_class {
                current_class_name.clone()
            } else {
                return Err("Cannot use 'self' outside of class context".to_string());
            }
        } else {
            class_name.to_string()
        };

        // Check if this is an enum (handle built-in enum methods)
        if let Some(enum_def) = self.enums.get(&target_class.to_lowercase()).cloned() {
            let method_lower = method.to_lowercase();

            return match method_lower.as_str() {
                "cases" => {
                    // Return array of all enum cases
                    if !args.is_empty() {
                        return Err("cases() takes no arguments".to_string());
                    }

                    use crate::interpreter::value::ArrayKey;
                    let cases: Vec<(ArrayKey, Value)> = enum_def
                        .cases
                        .iter()
                        .enumerate()
                        .map(|(i, (name, value))| {
                            (
                                ArrayKey::Integer(i as i64),
                                Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: value.as_ref().map(|v| Box::new(v.clone())),
                                },
                            )
                        })
                        .collect();

                    Ok(Value::Array(cases))
                }
                "from" => {
                    // Get case by backing value (throws on invalid)
                    if args.len() != 1 {
                        return Err("from() expects exactly 1 argument".to_string());
                    }

                    if enum_def.backing_type == crate::ast::EnumBackingType::None {
                        return Err(format!(
                            "Pure enum '{}' cannot use from() method",
                            enum_def.name
                        ));
                    }

                    let search_value = self.eval_expr(&args[0].value)?;

                    for (name, value) in &enum_def.cases {
                        if let Some(val) = value {
                            if self.values_identical(val, &search_value) {
                                return Ok(Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: Some(Box::new(val.clone())),
                                });
                            }
                        }
                    }

                    Err(format!(
                        "Value '{}' is not a valid backing value for enum '{}'",
                        search_value.to_string_val(),
                        enum_def.name
                    ))
                }
                "tryfrom" => {
                    // Get case by backing value (returns null on invalid)
                    if args.len() != 1 {
                        return Err("tryFrom() expects exactly 1 argument".to_string());
                    }

                    if enum_def.backing_type == crate::ast::EnumBackingType::None {
                        return Err(format!(
                            "Pure enum '{}' cannot use tryFrom() method",
                            enum_def.name
                        ));
                    }

                    let search_value = self.eval_expr(&args[0].value)?;

                    for (name, value) in &enum_def.cases {
                        if let Some(val) = value {
                            if self.values_identical(val, &search_value) {
                                return Ok(Value::EnumCase {
                                    enum_name: enum_def.name.clone(),
                                    case_name: name.clone(),
                                    backing_value: Some(Box::new(val.clone())),
                                });
                            }
                        }
                    }

                    Ok(Value::Null)
                }
                _ => {
                    // Check for user-defined method
                    if let Some(func) = enum_def.methods.get(&method_lower) {
                        // Call enum method (enums don't have instance state)
                        self.call_user_function_with_arguments(func, args)
                    } else {
                        Err(format!(
                            "Call to undefined method {}::{}()",
                            enum_def.name, method
                        ))
                    }
                }
            };
        }

        // Look up method in hierarchy
        let (method_func, declaring_class) = self
            .find_method(&target_class, method)
            .ok_or_else(|| format!("Call to undefined method {}::{}()", target_class, method))?;

        // Evaluate all arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
        }

        // Build a map of named arguments for quick lookup
        let mut named_args: std::collections::HashMap<String, Value> =
            std::collections::HashMap::new();
        let mut positional_idx = 0;

        for (i, arg) in args.iter().enumerate() {
            if let Some(ref name) = arg.name {
                named_args.insert(name.clone(), arg_values[i].clone());
            } else {
                positional_idx = i + 1;
            }
        }

        // Call method without $this (static call), but set current_class
        // Save current state
        let saved_variables = self.variables.clone();
        let saved_current_class = self.current_class.take();

        // Set current class to where the method is defined
        self.current_class = Some(declaring_class);

        // Clear variables
        self.variables.clear();

        // Bind arguments to parameters
        let mut positional_arg_idx = 0;
        for param in &method_func.params {
            let value = if let Some(named_value) = named_args.get(&param.name) {
                named_value.clone()
            } else if positional_arg_idx < positional_idx {
                arg_values[positional_arg_idx].clone()
            } else if let Some(ref default_expr) = param.default {
                self.eval_expr(default_expr)?
            } else {
                return Err(format!("Missing argument for parameter ${}", param.name));
            };

            if positional_arg_idx < positional_idx {
                positional_arg_idx += 1;
            }

            self.variables.insert(param.name.clone(), value);
        }

        // Check for unknown named arguments
        for arg in args {
            if let Some(ref name) = arg.name {
                if !method_func.params.iter().any(|p| p.name == *name) {
                    return Err(format!("Unknown named parameter ${}", name));
                }
            }
        }

        // Execute method body
        let mut return_value = Value::Null;
        for stmt in &method_func.body {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            match cf {
                crate::interpreter::ControlFlow::Return(v) => {
                    return_value = v;
                    break;
                }
                crate::interpreter::ControlFlow::Break
                | crate::interpreter::ControlFlow::Continue => break,
                crate::interpreter::ControlFlow::Exception(e) => {
                    return Err(format!("__EXCEPTION__:{}:{}", e.class_name, e.message));
                }
                crate::interpreter::ControlFlow::None => {}
            }
        }

        // Restore previous state
        self.variables = saved_variables;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Call a method on an object instance (internal helper)
    ///
    /// Lower-level method invocation used by eval_method_call.
    /// Supports positional arguments only (use call_method_on_object_with_arguments
    /// for named argument support).
    #[allow(dead_code)]
    pub(crate) fn call_method_on_object(
        &mut self,
        instance: &mut ObjectInstance,
        method: &crate::interpreter::UserFunction,
        args: &[Value],
        declaring_class: String,
    ) -> Result<Value, String> {
        // Save current state
        let saved_variables = self.variables.clone();
        let saved_current_object = self.current_object.take();
        let saved_current_class = self.current_class.take();

        // Set current object to this instance
        self.current_object = Some(instance.clone());
        self.current_class = Some(declaring_class);

        // Clear variables and set parameters
        self.variables.clear();

        // Bind arguments to parameters
        for (i, param) in method.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(ref default_expr) = param.default {
                self.eval_expr(default_expr)?
            } else {
                Value::Null
            };

            // Validate type hint if present
            if let Some(ref type_hint) = param.type_hint {
                if !value.matches_type(type_hint) {
                    return Err(format!(
                        "Argument {} for parameter ${} must be of type {}, {} given",
                        i + 1,
                        param.name,
                        Self::format_type_hint_for_method(type_hint),
                        value.type_name()
                    ));
                }
            }

            self.variables.insert(param.name.clone(), value);
        }

        // Execute method body
        let mut return_value = Value::Null;
        for stmt in &method.body {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            match cf {
                crate::interpreter::ControlFlow::Return(v) => {
                    return_value = v;
                    break;
                }
                crate::interpreter::ControlFlow::Break
                | crate::interpreter::ControlFlow::Continue => break,
                crate::interpreter::ControlFlow::Exception(e) => {
                    return Err(format!("__EXCEPTION__:{}:{}", e.class_name, e.message));
                }
                crate::interpreter::ControlFlow::None => {}
            }
        }

        // Validate return type if present
        if let Some(ref return_type) = method.return_type {
            self.validate_return_value_for_method(return_type, &return_value)?;
        }

        // Copy back any property changes from $this
        if let Some(ref obj) = self.current_object {
            *instance = obj.clone();
        }

        // Restore previous state
        self.variables = saved_variables;
        self.current_object = saved_current_object;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Call method on object with support for named arguments (PHP 8.0)
    ///
    /// Full-featured method invocation supporting both positional and named arguments.
    /// Used for both instance method calls and constructor invocation.
    pub(crate) fn call_method_on_object_with_arguments(
        &mut self,
        instance: &mut ObjectInstance,
        method: &crate::interpreter::UserFunction,
        args: &[Argument],
        declaring_class: String,
    ) -> Result<Value, String> {
        // Save current state
        let saved_variables = self.variables.clone();
        let saved_current_object = self.current_object.take();
        let saved_current_class = self.current_class.take();

        // Set current object to this instance
        self.current_object = Some(instance.clone());
        self.current_class = Some(declaring_class);

        // Clear variables
        self.variables.clear();

        // Evaluate all arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(&arg.value)?);
        }

        // Build a map of named arguments for quick lookup
        let mut named_args: std::collections::HashMap<String, Value> =
            std::collections::HashMap::new();
        let mut positional_idx = 0;

        for (i, arg) in args.iter().enumerate() {
            if let Some(ref name) = arg.name {
                named_args.insert(name.clone(), arg_values[i].clone());
            } else {
                positional_idx = i + 1;
            }
        }

        // Bind arguments to parameters
        let mut positional_arg_idx = 0;
        for param in &method.params {
            let value = if let Some(named_value) = named_args.get(&param.name) {
                named_value.clone()
            } else if positional_arg_idx < positional_idx {
                arg_values[positional_arg_idx].clone()
            } else if let Some(ref default_expr) = param.default {
                self.eval_expr(default_expr)?
            } else {
                Value::Null
            };

            if positional_arg_idx < positional_idx {
                positional_arg_idx += 1;
            }

            // Validate type hint if present
            if let Some(ref type_hint) = param.type_hint {
                if !value.matches_type(type_hint) {
                    return Err(format!(
                        "Argument for parameter ${} must be of type {}, {} given",
                        param.name,
                        Self::format_type_hint_for_method(type_hint),
                        value.type_name()
                    ));
                }
            }

            self.variables.insert(param.name.clone(), value);
        }

        // Check for unknown named arguments
        for arg in args {
            if let Some(ref name) = arg.name {
                if !method.params.iter().any(|p| p.name == *name) {
                    return Err(format!("Unknown named parameter ${}", name));
                }
            }
        }

        // Execute method body
        let mut return_value = Value::Null;
        for stmt in &method.body {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            match cf {
                crate::interpreter::ControlFlow::Return(v) => {
                    return_value = v;
                    break;
                }
                crate::interpreter::ControlFlow::Break
                | crate::interpreter::ControlFlow::Continue => break,
                crate::interpreter::ControlFlow::Exception(e) => {
                    return Err(format!("__EXCEPTION__:{}:{}", e.class_name, e.message));
                }
                crate::interpreter::ControlFlow::None => {}
            }
        }

        // Validate return type if present
        if let Some(ref return_type) = method.return_type {
            self.validate_return_value_for_method(return_type, &return_value)?;
        }

        // Copy back any property changes from $this
        if let Some(ref obj) = self.current_object {
            *instance = obj.clone();
        }

        // Restore previous state
        self.variables = saved_variables;
        self.current_object = saved_current_object;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Handle Fiber method calls
    fn call_fiber_method(&mut self, fiber_id: usize, method: &str, args: &[Argument]) -> Result<Value, String> {
        // Evaluate arguments
        let arg_values: Result<Vec<Value>, String> = args
            .iter()
            .map(|arg| self.eval_expr(&arg.value))
            .collect();
        let arg_values = arg_values?;

        match method.to_lowercase().as_str() {
            "start" => {
                self.fiber_start(fiber_id, arg_values)
            }
            "resume" => {
                let value = arg_values.first().cloned().unwrap_or(Value::Null);
                self.fiber_resume(fiber_id, value)
            }
            "throw" => {
                // TODO: Implement exception throwing into fiber
                Err("Fiber::throw() not yet implemented".to_string())
            }
            "getreturn" => {
                let fiber = self.fibers.get(&fiber_id)
                    .ok_or("Invalid fiber ID")?;
                
                if fiber.state != crate::interpreter::value::FiberState::Terminated {
                    return Err("Cannot get return value of non-terminated fiber".to_string());
                }
                
                Ok(fiber.return_value.as_ref().map(|v| v.as_ref()).unwrap_or(&Value::Null).clone())
            }
            "isstarted" => {
                let fiber = self.fibers.get(&fiber_id)
                    .ok_or("Invalid fiber ID")?;
                Ok(Value::Bool(fiber.state != crate::interpreter::value::FiberState::NotStarted))
            }
            "issuspended" => {
                let fiber = self.fibers.get(&fiber_id)
                    .ok_or("Invalid fiber ID")?;
                Ok(Value::Bool(fiber.state == crate::interpreter::value::FiberState::Suspended))
            }
            "isrunning" => {
                let fiber = self.fibers.get(&fiber_id)
                    .ok_or("Invalid fiber ID")?;
                Ok(Value::Bool(fiber.state == crate::interpreter::value::FiberState::Running))
            }
            "isterminated" => {
                let fiber = self.fibers.get(&fiber_id)
                    .ok_or("Invalid fiber ID")?;
                Ok(Value::Bool(fiber.state == crate::interpreter::value::FiberState::Terminated))
            }
            _ => Err(format!("Unknown Fiber method: {}", method))
        }
    }

    /// Format a type hint for error messages (method version)
    fn format_type_hint_for_method(hint: &crate::ast::TypeHint) -> String {
        use crate::ast::TypeHint;
        match hint {
            TypeHint::Simple(s) => s.clone(),
            TypeHint::Nullable(inner) => format!("?{}", Self::format_type_hint_for_method(inner)),
            TypeHint::Union(types) => types
                .iter()
                .map(|t| Self::format_type_hint_for_method(t))
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Intersection(types) => types
                .iter()
                .map(|t| Self::format_type_hint_for_method(t))
                .collect::<Vec<_>>()
                .join("&"),
            TypeHint::Class(name) => name.clone(),
            TypeHint::Void => "void".to_string(),
            TypeHint::Never => "never".to_string(),
            TypeHint::Static => "static".to_string(),
            TypeHint::SelfType => "self".to_string(),
            TypeHint::ParentType => "parent".to_string(),
        }
    }

    /// Validate return value against type hint (method version)
    fn validate_return_value_for_method(
        &self,
        return_type: &crate::ast::TypeHint,
        value: &Value,
    ) -> Result<(), String> {
        use crate::ast::TypeHint;
        match return_type {
            TypeHint::Void => {
                if !matches!(value, Value::Null) {
                    return Err(format!(
                        "Return value must be of type void, {} returned",
                        value.type_name()
                    ));
                }
            }
            TypeHint::Never => {
                return Err("never-returning method must not return".to_string());
            }
            _ => {
                if !value.matches_type(return_type) {
                    return Err(format!(
                        "Return value must be of type {}, {} returned",
                        Self::format_type_hint_for_method(return_type),
                        value.type_name()
                    ));
                }
            }
        }
        Ok(())
    }
}

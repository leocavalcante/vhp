//! Object-oriented programming features for the interpreter
//!
//! This module handles:
//! - Class instantiation (new ClassName)
//! - Property access and assignment
//! - Method calls
//! - Static method calls
//! - Object inheritance and composition

use crate::ast::{Argument, Property};
use crate::interpreter::value::{ObjectInstance, Value};
use crate::interpreter::Interpreter;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Collect all properties from class hierarchy
    pub(super) fn collect_properties(&mut self, class_name: &str) -> Result<Vec<Property>, String> {
        let class_def = self
            .classes
            .get(&class_name.to_lowercase())
            .cloned()
            .ok_or_else(|| format!("Class '{}' not found", class_name))?;

        let mut properties = if let Some(parent) = &class_def.parent {
            self.collect_properties(parent)?
        } else {
            Vec::new()
        };

        // Add/override properties from current class
        for prop in &class_def.properties {
            if let Some(existing) = properties.iter_mut().find(|p| p.name == prop.name) {
                *existing = prop.clone();
            } else {
                properties.push(prop.clone());
            }
        }

        Ok(properties)
    }

    /// Find method in class hierarchy
    pub(super) fn find_method(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<(crate::interpreter::UserFunction, String)> {
        let class_def = self.classes.get(&class_name.to_lowercase())?;

        if let Some(method) = class_def.methods.get(&method_name.to_lowercase()) {
            return Some((method.clone(), class_def.name.clone()));
        }

        if let Some(parent) = &class_def.parent {
            self.find_method(parent, method_name)
        } else {
            None
        }
    }

    /// Evaluate object instantiation (new ClassName(...))
    pub(super) fn eval_new(
        &mut self,
        class_name: &str,
        args: &[Argument],
    ) -> Result<Value, String> {
        let class_name_lower = class_name.to_lowercase();

        // Check if class exists
        if !self.classes.contains_key(&class_name_lower) {
            return Err(format!("Class '{}' not found", class_name));
        }

        // Collect properties from hierarchy
        let properties = self.collect_properties(class_name)?;

        // Create new object instance
        let mut instance = ObjectInstance::new(class_name.to_string());

        // Initialize properties with default values and track readonly
        for prop in properties {
            let default_val = if let Some(ref default_expr) = prop.default {
                self.eval_expr(default_expr)?
            } else {
                Value::Null
            };
            instance.properties.insert(prop.name.clone(), default_val);

            // Track readonly properties and mark with defaults as initialized
            if prop.readonly {
                instance.readonly_properties.insert(prop.name.clone());
                if prop.default.is_some() {
                    instance.initialized_readonly.insert(prop.name.clone());
                }
            }
        }

        // Get the readonly flag before we borrow class_def mutably
        let class_readonly = {
            let class_def = self.classes.get(&class_name_lower).unwrap();
            class_def.readonly
        };

        // Also handle constructor promoted properties
        let class_def = self.classes.get(&class_name_lower).unwrap();
        if let Some(constructor) = class_def.methods.get("__construct") {
            for param in &constructor.params {
                if param.visibility.is_some() && param.readonly {
                    instance.readonly_properties.insert(param.name.clone());
                }
            }
        }

        // Check for constructor (__construct)
        if let Some((constructor, declaring_class)) = self.find_method(class_name, "__construct") {
            // Call constructor with $this bound and named argument support
            self.call_method_on_object_with_arguments(
                &mut instance,
                &constructor,
                args,
                declaring_class,
            )?;
        }

        // After constructor completes, mark all current readonly properties as initialized
        for prop_name in instance.readonly_properties.iter() {
            if instance.properties.contains_key(prop_name) {
                instance.initialized_readonly.insert(prop_name.clone());
            }
        }

        // If class itself is readonly (PHP 8.2), all properties are implicitly readonly
        if class_readonly {
            // Get all property names from the instance and mark them as readonly
            let all_property_names: Vec<String> =
                instance.properties.keys().map(|k| k.to_string()).collect();

            // Add all properties to readonly set
            for prop_name in all_property_names {
                instance.readonly_properties.insert(prop_name.clone());
                // Mark as initialized since constructor has completed
                instance.initialized_readonly.insert(prop_name);
            }
        }

        Ok(Value::Object(instance))
    }

    /// Evaluate property access ($obj->property)
    pub(super) fn eval_property_access(
        &mut self,
        object: &crate::ast::Expr,
        property: &str,
    ) -> Result<Value, String> {
        let obj_value = self.eval_expr(object)?;

        // Handle enum case properties
        if let Value::EnumCase {
            enum_name,
            case_name,
            backing_value,
        } = obj_value
        {
            match property {
                "name" => return Ok(Value::String(case_name)),
                "value" => {
                    if let Some(val) = backing_value {
                        return Ok(*val);
                    } else {
                        return Err(format!(
                            "Pure enum case {}::{} does not have a 'value' property",
                            enum_name, case_name
                        ));
                    }
                }
                _ => {
                    return Err(format!(
                        "Enum case {}::{} does not have property '{}'",
                        enum_name, case_name, property
                    ));
                }
            }
        }

        // Handle object properties
        match obj_value {
            Value::Object(instance) => {
                if let Some(value) = instance.properties.get(property) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Err(format!(
                "Cannot access property on non-object ({})",
                obj_value.get_type()
            )),
        }
    }

    /// Evaluate method call ($obj->method(...))
    pub(super) fn eval_method_call(
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

    /// Evaluate property assignment ($obj->property = value)
    pub(super) fn eval_property_assign(
        &mut self,
        object: &crate::ast::Expr,
        property: &str,
        value: &crate::ast::Expr,
    ) -> Result<Value, String> {
        // For property assignment, we need to handle $this specially
        match object {
            crate::ast::Expr::This => {
                // Evaluate value first to avoid borrow conflicts
                let val = self.eval_expr(value)?;
                if let Some(ref mut obj) = self.current_object {
                    // Check if property is readonly and already initialized
                    if obj.readonly_properties.contains(property)
                        && obj.initialized_readonly.contains(property)
                    {
                        return Err(format!(
                            "Cannot modify readonly property {}::${}",
                            obj.class_name, property
                        ));
                    }

                    obj.properties.insert(property.to_string(), val.clone());

                    // If this is a readonly property, mark it as initialized
                    if obj.readonly_properties.contains(property) {
                        obj.initialized_readonly.insert(property.to_string());
                    }

                    Ok(val)
                } else {
                    Err("Cannot use $this outside of object context".to_string())
                }
            }
            crate::ast::Expr::Variable(var_name) => {
                // Evaluate value first
                let val = self.eval_expr(value)?;
                // Get the object from variable
                if let Some(Value::Object(mut instance)) = self.variables.get(var_name).cloned() {
                    // Check if property is readonly and already initialized
                    if instance.readonly_properties.contains(property)
                        && instance.initialized_readonly.contains(property)
                    {
                        return Err(format!(
                            "Cannot modify readonly property {}::${}",
                            instance.class_name, property
                        ));
                    }

                    instance
                        .properties
                        .insert(property.to_string(), val.clone());

                    // If this is a readonly property, mark it as initialized
                    if instance.readonly_properties.contains(property) {
                        instance.initialized_readonly.insert(property.to_string());
                    }

                    self.variables
                        .insert(var_name.clone(), Value::Object(instance));
                    Ok(val)
                } else {
                    Err(format!(
                        "Cannot access property on non-object variable ${}",
                        var_name
                    ))
                }
            }
            _ => {
                // For other expressions, evaluate and try to assign
                let obj_value = self.eval_expr(object)?;
                match obj_value {
                    Value::Object(_) => {
                        Err("Cannot assign property on temporary object expression".to_string())
                    }
                    _ => Err(format!(
                        "Cannot access property on non-object ({})",
                        obj_value.get_type()
                    )),
                }
            }
        }
    }

    /// Evaluate static method call (ClassName::method(...))
    pub(super) fn eval_static_method_call(
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
                crate::interpreter::ControlFlow::None => {}
            }
        }

        // Restore previous state
        self.variables = saved_variables;
        self.current_class = saved_current_class;

        Ok(return_value)
    }

    /// Call a method on an object instance
    #[allow(dead_code)]
    pub(super) fn call_method_on_object(
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
                crate::interpreter::ControlFlow::None => {}
            }
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
    pub(super) fn call_method_on_object_with_arguments(
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
                crate::interpreter::ControlFlow::None => {}
            }
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
}

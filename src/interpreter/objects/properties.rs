//! Property access and modification
//!
//! Handles:
//! - Reading object properties
//! - Assigning to object properties
//! - Readonly property enforcement (PHP 8.1+)
//! - Enum case properties (name and value)

use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Evaluate property access ($obj->property)
    ///
    /// Reads a property from an object or enum case. For enum cases,
    /// provides access to built-in 'name' and 'value' properties.
    pub(crate) fn eval_property_access(
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
                // Check for property hooks (PHP 8.4)
                let class = self.classes.get(&instance.class_name).cloned();
                if let Some(ref class) = class {
                    // Find property definition
                    if let Some(prop_def) = class.properties.iter().find(|p| p.name == property) {
                        // Check if property has a get hook
                        if let Some(get_hook) = prop_def.hooks.iter().find(|h| {
                            matches!(h.hook_type, crate::ast::PropertyHookType::Get)
                        }) {
                            // Execute the get hook
                            return self.execute_property_get_hook(&instance, get_hook);
                        }
                    }
                }

                // First check if property exists
                if let Some(value) = instance.properties.get(property) {
                    Ok(value.clone())
                } else {
                    // Check for __get magic method
                    if let Some(ref class) = class {
                        if let Some(method) = class.get_magic_method("__get") {
                            let class_name = instance.class_name.clone();
                            let mut inst_mut = instance.clone();
                            return self.call_method_on_object(
                                &mut inst_mut,
                                method,
                                &[Value::String(property.to_string())],
                                class_name,
                            );
                        }
                    }
                    // Return null for undefined property (PHP behavior)
                    Ok(Value::Null)
                }
            }
            _ => Err(format!(
                "Cannot access property on non-object ({})",
                obj_value.get_type()
            )),
        }
    }

    /// Evaluate property assignment ($obj->property = value)
    ///
    /// Modifies an object's property with readonly enforcement.
    /// Returns the assigned value.
    pub(crate) fn eval_property_assign(
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
                    // Check for property hooks (PHP 8.4)
                    let class_name = obj.class_name.clone();
                    let class = self.classes.get(&class_name).cloned();

                    // Check for set hook before borrowing obj mutably
                    let set_hook_info = if let Some(ref class) = class {
                        if let Some(prop_def) = class.properties.iter().find(|p| p.name == property) {
                            let has_get = prop_def.hooks.iter().any(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Get)
                            });
                            let has_set = prop_def.hooks.iter().any(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Set)
                            });

                            // Virtual property (get-only)
                            if has_get && !has_set {
                                return Err(format!(
                                    "Cannot write to read-only property {}::${}",
                                    class_name, property
                                ));
                            }

                            // Check if property has a set hook
                            prop_def.hooks.iter().find(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Set)
                            }).cloned()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Execute set hook if found
                    if let Some(set_hook) = set_hook_info {
                        let mut obj_clone = obj.clone();
                        let _ = obj; // Release the borrow
                        self.execute_property_set_hook(&mut obj_clone, &set_hook, val.clone())?;
                        self.current_object = Some(obj_clone);
                        return Ok(val);
                    }

                    // Check if property is readonly and already initialized
                    if obj.readonly_properties.contains(property)
                        && obj.initialized_readonly.contains(property)
                    {
                        return Err(format!(
                            "Cannot modify readonly property {}::${}",
                            obj.class_name, property
                        ));
                    }

                    // Check if this is a declared property
                    let is_declared = obj.properties.contains_key(property)
                        || obj.readonly_properties.contains(property);

                    if is_declared {
                        // Normal property assignment
                        obj.properties.insert(property.to_string(), val.clone());

                        // If this is a readonly property, mark it as initialized
                        if obj.readonly_properties.contains(property) {
                            obj.initialized_readonly.insert(property.to_string());
                        }

                        Ok(val)
                    } else {
                        // Check for __set magic method for undefined properties
                        let class = self.classes.get(&obj.class_name).cloned();
                        if let Some(class) = class {
                            if let Some(method) = class.get_magic_method("__set") {
                                // Need to clone obj to avoid borrow issues
                                let class_name = obj.class_name.clone();
                                let mut obj_clone = obj.clone();
                                // Call __set
                                self.call_method_on_object(
                                    &mut obj_clone,
                                    method,
                                    &[Value::String(property.to_string()), val.clone()],
                                    class_name,
                                )?;
                                return Ok(val);
                            }
                        }
                        // Allow dynamic property (with potential deprecation warning in future)
                        obj.properties.insert(property.to_string(), val.clone());
                        Ok(val)
                    }
                } else {
                    Err("Cannot use $this outside of object context".to_string())
                }
            }
            crate::ast::Expr::Variable(var_name) => {
                // Evaluate value first
                let val = self.eval_expr(value)?;
                // Get the object from variable
                if let Some(Value::Object(mut instance)) = self.variables.get(var_name).cloned() {
                    // Check for property hooks (PHP 8.4)
                    let class_name = instance.class_name.clone();
                    let class = self.classes.get(&class_name).cloned();

                    // Check for set hook
                    let set_hook_info = if let Some(ref class) = class {
                        if let Some(prop_def) = class.properties.iter().find(|p| p.name == property) {
                            let has_get = prop_def.hooks.iter().any(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Get)
                            });
                            let has_set = prop_def.hooks.iter().any(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Set)
                            });

                            // Virtual property (get-only)
                            if has_get && !has_set {
                                return Err(format!(
                                    "Cannot write to read-only property {}::${}",
                                    class_name, property
                                ));
                            }

                            // Check if property has a set hook
                            prop_def.hooks.iter().find(|h| {
                                matches!(h.hook_type, crate::ast::PropertyHookType::Set)
                            }).cloned()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Execute set hook if found
                    if let Some(set_hook) = set_hook_info {
                        self.execute_property_set_hook(&mut instance, &set_hook, val.clone())?;
                        self.variables
                            .insert(var_name.clone(), Value::Object(instance));
                        return Ok(val);
                    }

                    // Check if property is readonly and already initialized
                    if instance.readonly_properties.contains(property)
                        && instance.initialized_readonly.contains(property)
                    {
                        return Err(format!(
                            "Cannot modify readonly property {}::${}",
                            instance.class_name, property
                        ));
                    }

                    // Check if this is a declared property
                    let is_declared = instance.properties.contains_key(property)
                        || instance.readonly_properties.contains(property);

                    if is_declared {
                        // Normal property assignment
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
                        // Check for __set magic method for undefined properties
                        let class = self.classes.get(&instance.class_name).cloned();
                        if let Some(class) = class {
                            if let Some(method) = class.get_magic_method("__set") {
                                // Call __set but still need to update the variable
                                let class_name = instance.class_name.clone();
                                self.call_method_on_object(
                                    &mut instance,
                                    method,
                                    &[Value::String(property.to_string()), val.clone()],
                                    class_name,
                                )?;
                                self.variables
                                    .insert(var_name.clone(), Value::Object(instance));
                                return Ok(val);
                            }
                        }
                        // Allow dynamic property (with potential deprecation warning in future)
                        instance
                            .properties
                            .insert(property.to_string(), val.clone());
                        self.variables
                            .insert(var_name.clone(), Value::Object(instance));
                        Ok(val)
                    }
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

    /// Execute property get hook (PHP 8.4)
    fn execute_property_get_hook(
        &mut self,
        instance: &crate::interpreter::ObjectInstance,
        hook: &crate::ast::PropertyHook,
    ) -> Result<Value, String> {
        // Save current $this context
        let prev_this = self.current_object.clone();
        self.current_object = Some(instance.clone());

        let result = match &hook.body {
            crate::ast::PropertyHookBody::Expression(expr) => {
                // Evaluate the expression
                self.eval_expr(expr)
            }
            crate::ast::PropertyHookBody::Block(statements) => {
                // Execute statements, capture return value
                let mut result = Value::Null;
                for stmt in statements {
                    match stmt {
                        crate::ast::Stmt::Return(expr_opt) => {
                            result = if let Some(e) = expr_opt {
                                self.eval_expr(e)?
                            } else {
                                Value::Null
                            };
                            break;
                        }
                        _ => {
                            self.execute_stmt(stmt).map_err(|e| e.to_string())?;
                        }
                    }
                }
                Ok(result)
            }
        };

        // Restore $this context
        self.current_object = prev_this;

        result
    }

    /// Execute property set hook (PHP 8.4)
    fn execute_property_set_hook(
        &mut self,
        instance: &mut crate::interpreter::ObjectInstance,
        hook: &crate::ast::PropertyHook,
        value: Value,
    ) -> Result<(), String> {
        // Save current $this context
        let prev_this = self.current_object.clone();
        self.current_object = Some(instance.clone());

        // Store the incoming value in a special variable $value
        let prev_value = self.variables.get("value").cloned();
        self.variables.insert("value".to_string(), value);

        let result = match &hook.body {
            crate::ast::PropertyHookBody::Expression(expr) => {
                // For set hooks, expression form just evaluates (for side effects)
                self.eval_expr(expr)?;
                Ok(())
            }
            crate::ast::PropertyHookBody::Block(statements) => {
                // Execute statements
                for stmt in statements {
                    self.execute_stmt(stmt).map_err(|e| e.to_string())?;
                }
                Ok(())
            }
        };

        // Restore context
        self.current_object = prev_this;
        if let Some(v) = prev_value {
            self.variables.insert("value".to_string(), v);
        } else {
            self.variables.remove("value");
        }

        result
    }

    /// Get static property value (ClassName::$property, self::$property, parent::$property, static::$property)
    pub(crate) fn get_static_property(
        &mut self,
        class: &str,
        property: &str,
    ) -> Result<Value, String> {
        // Resolve class name (handle self, parent, static)
        let resolved_class = self.resolve_static_class_name(class)?;

        let class_key = resolved_class.to_lowercase();

        // Get static properties for this class
        let static_props = self
            .static_properties
            .get(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

        // Get the property value
        static_props
            .get(property)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Access to undeclared static property {}::${}",
                    resolved_class, property
                )
            })
    }

    /// Set static property value
    pub(crate) fn set_static_property(
        &mut self,
        class: &str,
        property: &str,
        value: Value,
    ) -> Result<(), String> {
        // Resolve class name (handle self, parent, static)
        let resolved_class = self.resolve_static_class_name(class)?;

        let class_key = resolved_class.to_lowercase();

        // Get mutable reference to static properties
        let static_props = self
            .static_properties
            .get_mut(&class_key)
            .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

        // Check if property exists (PHP doesn't allow creating new static props at runtime)
        if !static_props.contains_key(property) {
            return Err(format!(
                "Access to undeclared static property {}::${}",
                resolved_class, property
            ));
        }

        // Set the value
        static_props.insert(property.to_string(), value);
        Ok(())
    }

    /// Resolve class name for static context
    /// Handles "self", "parent", and "static" (late static binding)
    fn resolve_static_class_name(&self, class: &str) -> Result<String, String> {
        match class.to_lowercase().as_str() {
            "self" => {
                // Return the current class context
                self.current_class
                    .clone()
                    .ok_or_else(|| "Cannot use 'self' outside of class context".to_string())
            }
            "parent" => {
                // Return the parent class
                let current = self.current_class
                    .as_ref()
                    .ok_or_else(|| "Cannot use 'parent' outside of class context".to_string())?;

                let class_def = self.classes
                    .get(&current.to_lowercase())
                    .ok_or_else(|| format!("Current class '{}' not found", current))?;

                class_def.parent
                    .clone()
                    .ok_or_else(|| "Cannot use 'parent' in class with no parent".to_string())
            }
            "static" => {
                // Late static binding: return the called class, not the defined class
                // This requires tracking the "called class" in the call stack
                self.called_class
                    .clone()
                    .ok_or_else(|| "Cannot use 'static' outside of class context".to_string())
            }
            _ => {
                // Regular class name
                Ok(class.to_string())
            }
        }
    }
}

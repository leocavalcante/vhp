//! Property access and modification
//!
//! Handles:
//! - Reading object properties
//! - Assigning to object properties
//! - Readonly property enforcement (PHP 8.1+)
//! - Enum case properties (name and value)

use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use crate::ast::Visibility;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Check if class is a subclass of another class
    fn is_subclass(&self, child: &str, parent: &str) -> bool {
        let mut current = child.to_lowercase();
        while let Some(class_def) = self.classes.get(&current) {
            if let Some(class_parent) = &class_def.parent {
                if class_parent.eq_ignore_ascii_case(parent) {
                    return true;
                }
                current = class_parent.to_lowercase();
            } else {
                break;
            }
        }
        false
    }

    /// Check if write access is allowed for a property based on asymmetric visibility
    fn can_write_property(
        &self,
        class_name: &str,
        property: &str,
    ) -> Result<(), String> {
        // Search for property definition in class hierarchy
        let mut current_search = class_name.to_lowercase();
        let mut property_class = None;
        let mut prop_def = None;

        while let Some(class_def) = self.classes.get(&current_search) {
            if let Some(found_prop) = class_def.properties.iter().find(|p| p.name == property) {
                property_class = Some(current_search.clone());
                prop_def = Some(found_prop.clone());
                break;
            }
            // Search parent class
            if let Some(parent) = &class_def.parent {
                current_search = parent.to_lowercase();
            } else {
                break;
            }
        }

        // If property found, check write visibility
        if let (Some(prop_class), Some(prop)) = (property_class, prop_def) {
            // Get write visibility (use write_visibility if set, otherwise use read visibility)
            let write_vis = prop.write_visibility.unwrap_or(prop.visibility);

            // Determine if current context can write
            let can_write = match write_vis {
                Visibility::Public => true,
                Visibility::Protected => {
                    // Can write if we're in the same class hierarchy (parent, child, or same)
                    if let Some(current) = &self.current_class {
                        let current_lower = current.to_lowercase();
                        // Same class
                        current_lower == prop_class
                            // Current is a subclass of where it's defined
                            || self.is_subclass(&current_lower, &prop_class)
                            // Current is a parent of where it's "found" (property inherited from current)
                            || self.is_subclass(&prop_class, &current_lower)
                    } else {
                        false
                    }
                }
                Visibility::Private => {
                    // Can write only if we're in the exact class where it's defined
                    self.current_class
                        .as_ref()
                        .map(|current| current.to_lowercase() == prop_class)
                        .unwrap_or(false)
                }
            };

            if !can_write {
                return Err(format!(
                    "Cannot modify {} property {}::${}",
                    match write_vis {
                        Visibility::Public => "public",
                        Visibility::Protected => "protected",
                        Visibility::Private => "private",
                    },
                    class_name,
                    property
                ));
            }
        }

        Ok(())
    }
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
                let class = self
                    .classes
                    .get(&instance.class_name.to_lowercase())
                    .cloned();
                if let Some(ref class) = class {
                    // Find property definition
                    if let Some(prop_def) = class.properties.iter().find(|p| p.name == property) {
                        // Check if property has a get hook
                        if let Some(get_hook) = prop_def
                            .hooks
                            .iter()
                            .find(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Get))
                        {
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

                // Get class name and check for hooks before visibility check
                let class_name = if let Some(ref obj) = self.current_object {
                    obj.class_name.clone()
                } else {
                    return Err("Cannot use $this outside of object context".to_string());
                };

                // Check for property hooks first (hooks bypass visibility checks)
                let class = self.classes.get(&class_name.to_lowercase()).cloned();
                let has_hooks = if let Some(ref class) = class {
                    class.properties.iter()
                        .find(|p| p.name == property)
                        .map(|p| !p.hooks.is_empty())
                        .unwrap_or(false)
                } else {
                    false
                };

                // Check write visibility only if no hooks (hooks handle their own access control)
                if !has_hooks {
                    self.can_write_property(&class_name, property)?;
                }

                if let Some(ref mut obj) = self.current_object {
                    // Check for set hook
                    let set_hook_info = if let Some(ref class) = class {
                        if let Some(prop_def) = class.properties.iter().find(|p| p.name == property)
                        {
                            let has_get = prop_def
                                .hooks
                                .iter()
                                .any(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Get));
                            let has_set = prop_def
                                .hooks
                                .iter()
                                .any(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Set));

                            // Virtual property (get-only)
                            if has_get && !has_set {
                                return Err(format!(
                                    "Cannot write to read-only property {}::${}",
                                    class_name, property
                                ));
                            }

                            // Check if property has a set hook
                            prop_def
                                .hooks
                                .iter()
                                .find(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Set))
                                .cloned()
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
                        let class = self.classes.get(&obj.class_name.to_lowercase()).cloned();
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

                // Get class name for visibility check
                let class_name = if let Some(Value::Object(ref instance)) = self.variables.get(var_name) {
                    instance.class_name.clone()
                } else {
                    return Err(format!(
                        "Cannot access property on non-object variable ${}",
                        var_name
                    ));
                };

                // Check for property hooks first (hooks bypass visibility checks)
                let class = self.classes.get(&class_name.to_lowercase()).cloned();
                let has_hooks = if let Some(ref class) = class {
                    class.properties.iter()
                        .find(|p| p.name == property)
                        .map(|p| !p.hooks.is_empty())
                        .unwrap_or(false)
                } else {
                    false
                };

                // Check write visibility only if no hooks
                if !has_hooks {
                    self.can_write_property(&class_name, property)?;
                }

                // Get the object from variable
                if let Some(Value::Object(mut instance)) = self.variables.get(var_name).cloned() {
                    // Check for set hook
                    let set_hook_info = if let Some(ref class) = class {
                        if let Some(prop_def) = class.properties.iter().find(|p| p.name == property)
                        {
                            let has_get = prop_def
                                .hooks
                                .iter()
                                .any(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Get));
                            let has_set = prop_def
                                .hooks
                                .iter()
                                .any(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Set));

                            // Virtual property (get-only)
                            if has_get && !has_set {
                                return Err(format!(
                                    "Cannot write to read-only property {}::${}",
                                    class_name, property
                                ));
                            }

                            // Check if property has a set hook
                            prop_def
                                .hooks
                                .iter()
                                .find(|h| matches!(h.hook_type, crate::ast::PropertyHookType::Set))
                                .cloned()
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
                        let class = self
                            .classes
                            .get(&instance.class_name.to_lowercase())
                            .cloned();
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
        // Save current context
        let prev_this = self.current_object.clone();
        let prev_class = self.current_class.clone();

        // Set context for hook execution
        self.current_object = Some(instance.clone());
        self.current_class = Some(instance.class_name.clone());

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

        // Restore context
        self.current_object = prev_this;
        self.current_class = prev_class;

        result
    }

    /// Execute property set hook (PHP 8.4)
    fn execute_property_set_hook(
        &mut self,
        instance: &mut crate::interpreter::ObjectInstance,
        hook: &crate::ast::PropertyHook,
        value: Value,
    ) -> Result<(), String> {
        // Save current context
        let prev_this = self.current_object.clone();
        let prev_class = self.current_class.clone();

        // Set context for hook execution
        self.current_object = Some(instance.clone());
        self.current_class = Some(instance.class_name.clone());

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

        // Retrieve the modified instance before restoring context
        if let Some(modified_obj) = self.current_object.clone() {
            *instance = modified_obj;
        }

        // Restore context
        self.current_object = prev_this;
        self.current_class = prev_class;
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

        // Try to get from the resolved class first
        if let Some(static_props) = self.static_properties.get(&class_key) {
            if let Some(value) = static_props.get(property) {
                return Ok(value.clone());
            }
        }

        // If not found and this is a base class (self or parent resolution),
        // search up the inheritance hierarchy for the property
        if let Some(class_def) = self.classes.get(&class_key).cloned() {
            let mut current_parent = class_def.parent;
            while let Some(parent_name) = current_parent {
                let parent_key = parent_name.to_lowercase();
                if let Some(parent_statics) = self.static_properties.get(&parent_key) {
                    if let Some(value) = parent_statics.get(property) {
                        return Ok(value.clone());
                    }
                }
                // Move up the hierarchy
                if let Some(parent_class) = self.classes.get(&parent_key) {
                    current_parent = parent_class.parent.clone();
                } else {
                    break;
                }
            }
        }

        // Property not found anywhere in the hierarchy
        Err(format!(
            "Access to undeclared static property {}::${}",
            resolved_class, property
        ))
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

        // Check write visibility (asymmetric visibility)
        self.can_write_property(&resolved_class, property)?;

        // Check if property is readonly in this class
        if let Some(readonly_props) = self.static_readonly_properties.get(&class_key) {
            if readonly_props.contains(property) {
                return Err("Cannot modify readonly property".to_string());
            }
        }

        // Try to set in the resolved class first
        if let Some(static_props) = self.static_properties.get_mut(&class_key) {
            if static_props.contains_key(property) {
                static_props.insert(property.to_string(), value);
                return Ok(());
            }
        }

        // If not found in the resolved class, search up the inheritance hierarchy
        if let Some(class_def) = self.classes.get(&class_key).cloned() {
            let mut current_parent = class_def.parent;
            while let Some(parent_name) = current_parent {
                let parent_key = parent_name.to_lowercase();

                // Check readonly in parent
                if let Some(readonly_props) = self.static_readonly_properties.get(&parent_key) {
                    if readonly_props.contains(property) {
                        return Err("Cannot modify readonly property".to_string());
                    }
                }

                // Try to set in parent
                if let Some(parent_statics) = self.static_properties.get_mut(&parent_key) {
                    if parent_statics.contains_key(property) {
                        parent_statics.insert(property.to_string(), value);
                        return Ok(());
                    }
                }
                // Move up the hierarchy
                if let Some(parent_class) = self.classes.get(&parent_key) {
                    current_parent = parent_class.parent.clone();
                } else {
                    break;
                }
            }
        }

        // Property not found anywhere in the hierarchy
        Err(format!(
            "Access to undeclared static property {}::${}",
            resolved_class, property
        ))
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
                let current = self
                    .current_class
                    .as_ref()
                    .ok_or_else(|| "Cannot use 'parent' outside of class context".to_string())?;

                let class_def = self
                    .classes
                    .get(&current.to_lowercase())
                    .ok_or_else(|| format!("Current class '{}' not found", current))?;

                class_def
                    .parent
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

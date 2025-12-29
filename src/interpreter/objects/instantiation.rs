//! Object instantiation and construction
//!
//! Handles:
//! - Creating new object instances
//! - Calling constructors (__construct)
//! - Initializing properties with default values
//! - Constructor property promotion (PHP 8.0)
//! - Readonly property tracking (PHP 8.1+)
//! - Class-level readonly enforcement (PHP 8.2)

use crate::ast::Argument;
use crate::interpreter::value::ObjectInstance;
use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Evaluate object instantiation (new ClassName(...))
    ///
    /// Creates a new instance of the specified class and invokes its constructor
    /// if one exists. Handles property initialization with defaults and readonly
    /// properties.
    pub(crate) fn eval_new(
        &mut self,
        class_name: &str,
        args: &[Argument],
    ) -> Result<Value, String> {
        let class_name_lower = class_name.to_lowercase();

        // Check if class exists
        if !self.classes.contains_key(&class_name_lower) {
            return Err(format!("Class '{}' not found", class_name));
        }
        
        // Check if class is abstract
        {
            let class_def = self.classes.get(&class_name_lower).unwrap();
            if class_def.is_abstract {
                return Err(format!("Cannot instantiate abstract class {}", class_name));
            }
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
}

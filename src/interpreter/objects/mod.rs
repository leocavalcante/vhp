//! Object-oriented programming features for the interpreter
//!
//! This module handles:
//! - Class instantiation (new ClassName)
//! - Property access and assignment
//! - Method calls
//! - Static method calls
//! - Object inheritance and composition

mod instantiation;
mod methods;
mod properties;

use crate::interpreter::Interpreter;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Collect all properties from class hierarchy (recursive)
    ///
    /// Traverses the inheritance chain and merges properties from all classes.
    /// Properties from derived classes override parent properties.
    pub(crate) fn collect_properties(
        &mut self,
        class_name: &str,
    ) -> Result<Vec<crate::ast::Property>, String> {
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

    /// Find method in class hierarchy (recursive)
    ///
    /// Traverses the inheritance chain to find a method by name.
    /// Returns the method and the class that declares it.
    pub(crate) fn find_method(
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
}

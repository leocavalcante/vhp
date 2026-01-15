//! Class hierarchy support for VM
//!
//! This module contains functions for:
//! - Class instance checking (is_instance_of)
//! - Class inheritance checking (is_subclass_of)
//! - Interface inheritance checking (interface_extends)
//! - Class keyword resolution (self, parent, static)
//! - Class name normalization

use std::io::Write;

impl<W: Write> super::VM<W> {
    /// Check if a class is an instance of another class (including interfaces and parents)
    /// Checks: exact match, parent class, and implemented interfaces
    pub fn is_instance_of(&self, obj_class: &str, target_class: &str) -> bool {
        if obj_class.eq_ignore_ascii_case(target_class) {
            return true;
        }

        // Check parent classes
        if let Some(class_def) = self.classes.get(obj_class) {
            if let Some(ref parent) = class_def.parent {
                if self.is_instance_of(parent, target_class) {
                    return true;
                }
            }
            // Check interfaces (direct implementation)
            for interface in &class_def.interfaces {
                if interface.eq_ignore_ascii_case(target_class) {
                    return true;
                }
                // Also check if interface extends target
                if self.interface_extends(interface, target_class) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if an interface extends another interface (recursively)
    /// Traverses interface's parent chain to check for extension
    pub fn interface_extends(&self, interface: &str, target: &str) -> bool {
        if interface.eq_ignore_ascii_case(target) {
            return true;
        }
        if let Some(interface_def) = self.interfaces.get(interface) {
            for parent_interface in &interface_def.parents {
                if parent_interface.eq_ignore_ascii_case(target) {
                    return true;
                }
                if self.interface_extends(parent_interface, target) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a class is a subclass of another class
    /// Traverses parent chain to check for inheritance relationship
    pub fn is_subclass_of(&self, child: &str, parent: &str) -> bool {
        let mut current = child.to_string();
        while let Some(class) = self.classes.get(&current) {
            if let Some(ref parent_name) = class.parent {
                if parent_name == parent {
                    return true;
                }
                current = parent_name.clone();
            } else {
                return false;
            }
        }
        false
    }

    /// Normalize a class name by removing leading backslash if present
    /// A leading backslash indicates a fully qualified name from global namespace
    pub fn normalize_class_name(name: &str) -> String {
        if let Some(stripped) = name.strip_prefix('\\') {
            stripped.to_string()
        } else {
            name.to_string()
        }
    }

    /// Resolve self/static/parent to actual class name
    /// Returns appropriate class name based on keyword and current context
    pub fn resolve_class_keyword(&self, keyword: &str) -> Result<String, String> {
        match keyword {
            "self" => self
                .get_current_class()
                .ok_or_else(|| "Cannot use self:: outside of class".to_string()),
            "static" => {
                // Late static binding: use called_class if available, otherwise fall back to current class
                if let Some(frame) = self.frames.last() {
                    if let Some(called_class) = &frame.called_class {
                        return Ok(called_class.clone());
                    }
                }
                self.get_current_class()
                    .ok_or_else(|| "Cannot use static:: outside of class".to_string())
            }
            "parent" => {
                let current_class = self
                    .get_current_class()
                    .ok_or_else(|| "Cannot use parent:: outside of class".to_string())?;
                let class_def = self
                    .classes
                    .get(&current_class)
                    .ok_or_else(|| format!("Class '{}' not found", current_class))?;
                class_def
                    .parent
                    .clone()
                    .ok_or_else(|| format!("Class '{}' has no parent", current_class))
            }
            other => Ok(other.to_string()),
        }
    }

    /// Try to autoload a class by calling registered autoloaders
    /// Returns true if the class was successfully loaded, false otherwise
    #[allow(dead_code)]
    pub fn try_autoload_class(&mut self, class_name: &str) -> bool {
        use crate::runtime::builtins::spl;

        let normalized = Self::normalize_class_name(class_name);

        // First try PSR-4 autoloading if there's a mapping
        if spl::find_psr4_mapping(&normalized).is_some()
            && self.load_psr4_class(&normalized).unwrap_or(false)
        {
            return true;
        }

        // Then try registered autoloaders
        if spl::has_autoloaders() {
            spl::spl_autoload_call(self, &normalized)
        } else {
            false
        }
    }

    /// Get a class definition, attempting autoloading if not found
    /// Returns None if the class is not found even after autoloading
    #[allow(dead_code)]
    pub fn get_class_with_autoload(
        &mut self,
        class_name: &str,
    ) -> Option<std::sync::Arc<crate::vm::class::CompiledClass>> {
        let normalized = Self::normalize_class_name(class_name);

        // First try to get the class directly
        if let Some(class) = self.classes.get(&normalized) {
            return Some(class.clone());
        }

        // Don't try autoloading for enums - they must be defined in code
        if self.enums.contains_key(&normalized) {
            return None;
        }

        // Try autoloading
        if self.try_autoload_class(&normalized) {
            // Check again after autoloading
            self.classes.get(&normalized).cloned()
        } else {
            None
        }
    }
}

//! Method resolution for VM
//!
//! This module contains functions for:
//! - Finding methods in class hierarchies (including traits and parent classes)
//! - Resolving static and instance methods
//! - Calling methods synchronously (for magic methods like __toString)
//! - Value to string conversion (with __toString support)

use crate::runtime::ObjectInstance;
use crate::vm::frame::CallFrame;
use crate::vm::opcode::CompiledFunction;
use std::io::Write;
use std::sync::Arc;

impl<W: Write> super::VM<W> {
    /// Look up method through inheritance chain
    /// Searches through class hierarchy and traits to find a method
    pub fn find_method_in_chain(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<Arc<CompiledFunction>> {
        let mut current_class = Some(class_name.to_string());

        while let Some(class) = current_class {
            if let Some(class_def) = self.classes.get(&class) {
                // Try to find method in this class
                if let Some(method) = class_def.get_method(method_name) {
                    return Some(method.clone());
                }
                // Try traits (recursively)
                for trait_name in &class_def.traits {
                    if let Some(method) = self.find_method_in_trait(trait_name, method_name) {
                        return Some(method);
                    }
                }
                // Move to parent
                current_class = class_def.parent.clone();
            } else {
                break;
            }
        }
        None
    }

    /// Recursively look up method in trait and its used traits
    pub fn find_method_in_trait(
        &self,
        trait_name: &str,
        method_name: &str,
    ) -> Option<Arc<CompiledFunction>> {
        if let Some(trait_def) = self.traits.get(trait_name) {
            // Check if this trait has a method
            if let Some(method) = trait_def.methods.get(method_name) {
                return Some(method.clone());
            }
            // Recursively check traits used by this trait
            for used_trait in &trait_def.uses {
                if let Some(method) = self.find_method_in_trait(used_trait, method_name) {
                    return Some(method);
                }
            }
        }
        None
    }

    /// Look up static method through inheritance chain (case-insensitive per PHP spec)
    /// Returns (method, is_instance_method) where is_instance_method indicates
    /// if we had to fall back to an instance method (PHP allows static calls to instance methods)
    pub fn find_static_method_in_chain(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Option<(Arc<CompiledFunction>, bool)> {
        let mut current_class = Some(class_name.to_string());

        while let Some(class) = current_class {
            if let Some(class_def) = self.classes.get(&class) {
                // Try static methods first (case-insensitive)
                if let Some(method) = class_def.static_methods.get(method_name) {
                    return Some((method.clone(), false));
                }
                // Try case-insensitive static method lookup
                if let Some((_, method)) = class_def
                    .static_methods
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(method_name))
                    .map(|(k, v)| (k.clone(), v.clone()))
                {
                    return Some((method.clone(), false));
                }
                // Try instance methods (PHP allows calling them statically)
                if let Some(method) = class_def.get_method(method_name) {
                    return Some((method.clone(), true));
                }
                // Move to parent
                current_class = class_def.parent.clone();
            } else {
                break;
            }
        }
        None
    }

    /// Call a method synchronously and return its result
    /// This is used for magic methods like __toString that need immediate evaluation
    pub fn call_method_sync(
        &mut self,
        instance: ObjectInstance,
        method: Arc<CompiledFunction>,
    ) -> Result<crate::runtime::Value, String> {
        // Save current frame count to know when to stop
        let initial_frame_count = self.frames.len();

        // Create call frame
        let mut frame = CallFrame::new(method.clone(), self.stack.len());
        frame.locals[0] = crate::runtime::Value::Object(instance); // Set $this

        // Push frame
        self.frames.push(frame);

        // Execute until we return to original frame level
        loop {
            // Check if we've returned to original frame level
            if self.frames.len() <= initial_frame_count {
                return Ok(self.stack.pop().unwrap_or(crate::runtime::Value::Null));
            }

            // Get frame info without holding borrow
            let (bytecode_len, ip) = {
                let frame = self.frames.last().unwrap();
                (frame.function.bytecode.len(), frame.ip)
            };

            // Check if current frame is done
            if ip >= bytecode_len {
                let returned = self.stack.pop().unwrap_or(crate::runtime::Value::Null);
                self.frames.pop();

                if self.frames.len() <= initial_frame_count {
                    return Ok(returned);
                }

                self.stack.push(returned);
                continue;
            }

            // Get and execute next instruction
            let opcode = {
                let frame = self.frames.last_mut().unwrap();
                let op = frame.function.bytecode[frame.ip].clone();
                frame.ip += 1;
                op
            };

            // Handle return separately since execute_opcode returns Err for returns
            match self.execute_opcode(opcode) {
                Ok(()) => {}
                Err(e) if e == "__RETURN__" => {
                    // Return with value on stack
                    let returned = self.stack.pop().unwrap_or(crate::runtime::Value::Null);
                    self.frames.pop();

                    if self.frames.len() <= initial_frame_count {
                        return Ok(returned);
                    }
                    self.stack.push(returned);
                }
                Err(e) if e == "__RETURN__null" => {
                    // Return null
                    self.frames.pop();

                    if self.frames.len() <= initial_frame_count {
                        return Ok(crate::runtime::Value::Null);
                    }
                    self.stack.push(crate::runtime::Value::Null);
                }
                Err(e) if e == "__GENERATOR__" => {
                    // Generator yield - return generator object
                    let generator = self.stack.pop().unwrap_or(crate::runtime::Value::Null);
                    self.frames.pop();

                    if self.frames.len() <= initial_frame_count {
                        return Ok(generator);
                    }
                    self.stack.push(generator);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Convert a value to string, calling __toString for objects if available
    pub fn value_to_string(&mut self, value: crate::runtime::Value) -> Result<String, String> {
        match value {
            crate::runtime::Value::Object(ref instance) => {
                let class_name = instance.class_name.clone();
                // Look for __toString method
                if let Some(to_string_method) = self.find_method_in_chain(&class_name, "__toString")
                {
                    let result = self.call_method_sync(instance.clone(), to_string_method)?;
                    match result {
                        crate::runtime::Value::String(s) => Ok(s),
                        _ => Err("__toString must return a string".to_string()),
                    }
                } else {
                    // No __toString method - this is an error in PHP
                    Err(format!(
                        "Object of class {} could not be converted to string",
                        class_name
                    ))
                }
            }
            _ => Ok(value.to_string_val()),
        }
    }
}

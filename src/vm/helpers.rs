//! Value operations and builtin function helpers for VM
//!
//! This module contains:
//! - Arithmetic operations on values
//! - Value comparison operations
//! - Builtin function dispatcher

use crate::runtime::Value;
use crate::vm::{builtins, reflection, VM};
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    pub static ref REQUIRED_FILES: Arc<Mutex<std::collections::HashSet<String>>> =
        Arc::new(Mutex::new(std::collections::HashSet::new()));
}

/// Clear required files registry (useful for testing)
pub fn clear_required_files() {
    let mut required = REQUIRED_FILES.lock().unwrap();
    required.clear();
}

impl<W: std::io::Write> VM<W> {
    pub fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                for (k, v) in b {
                    if !result.iter().any(|(key, _)| key == k) {
                        result.push((k.clone(), v.clone()));
                    }
                }
                Ok(Value::Array(result))
            }
            _ => {
                let a = left.to_float();
                let b = right.to_float();
                Ok(Value::Float(a + b))
            }
        }
    }

    pub fn compare_values(&self, left: &Value, right: &Value) -> Result<i64, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok((*a).cmp(b) as i64),
            (Value::Float(a), Value::Float(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Integer(a), Value::Float(b)) => {
                let a = *a as f64;
                if a < *b {
                    Ok(-1)
                } else if a > *b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                let b = *b as f64;
                if a < &b {
                    Ok(-1)
                } else if a > &b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b) as i64),
            _ => {
                let a = left.to_float();
                let b = right.to_float();
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn call_reflection_or_builtin(
        &mut self,
        func_name: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        match func_name {
            "get_class_attributes" => {
                if args.is_empty() {
                    return Err("get_class_attributes() expects 1 argument".to_string());
                }
                let class_name = args[0].to_string_val();
                reflection::get_class_attributes(&class_name, &self.classes)
            }
            "get_property_attributes" => {
                if args.len() < 2 {
                    return Err("get_property_attributes() expects 2 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let property_name = args[1].to_string_val();
                reflection::get_property_attributes(&class_name, &property_name, &self.classes)
            }
            "get_method_attributes" => {
                if args.len() < 2 {
                    return Err("get_method_attributes() expects 2 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let method_name = args[1].to_string_val();
                reflection::get_method_attributes(&class_name, &method_name, &self.classes)
            }
            "get_method_parameter_attributes" => {
                if args.len() < 3 {
                    return Err("get_method_parameter_attributes() expects 3 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let method_name = args[1].to_string_val();
                let parameter_name = args[2].to_string_val();
                reflection::get_method_parameter_attributes(
                    &class_name,
                    &method_name,
                    &parameter_name,
                    &self.classes,
                )
            }
            "get_function_attributes" => {
                if args.is_empty() {
                    return Err("get_function_attributes() expects 1 argument".to_string());
                }
                let function_name = args[0].to_string_val();
                reflection::get_function_attributes(&function_name, &self.functions)
            }
            "get_parameter_attributes" => {
                if args.len() < 2 {
                    return Err("get_parameter_attributes() expects 2 arguments".to_string());
                }
                let function_name = args[0].to_string_val();
                let parameter_name = args[1].to_string_val();
                reflection::get_parameter_attributes(
                    &function_name,
                    &parameter_name,
                    &self.functions,
                )
            }
            "get_interface_attributes" => {
                if args.is_empty() {
                    return Err("get_interface_attributes() expects 1 argument".to_string());
                }
                let interface_name = args[0].to_string_val();
                reflection::get_interface_attributes(&interface_name, &self.interfaces)
            }
            "get_trait_attributes" => {
                if args.is_empty() {
                    return Err("get_trait_attributes() expects 1 argument".to_string());
                }
                let trait_name = args[0].to_string_val();
                reflection::get_trait_attributes(&trait_name, &self.traits)
            }
            "interface_exists" => {
                if args.is_empty() {
                    return Err("interface_exists() expects at least 1 parameter".to_string());
                }
                let interface_name = args[0].to_string_val();
                // Check parameter is string
                match &args[0] {
                    Value::String(_) => {}
                    _ => return Ok(Value::Bool(false)),
                }
                // Case-insensitive lookup
                let name_lower = interface_name.to_lowercase();
                let exists = self
                    .interfaces
                    .iter()
                    .any(|(k, _)| k.to_lowercase() == name_lower);
                Ok(Value::Bool(exists))
            }
            "trait_exists" => {
                if args.is_empty() {
                    return Err("trait_exists() expects at least 1 parameter".to_string());
                }
                let trait_name = args[0].to_string_val();
                // Check parameter is string
                match &args[0] {
                    Value::String(_) => {}
                    _ => return Ok(Value::Bool(false)),
                }
                // Case-insensitive lookup
                let name_lower = trait_name.to_lowercase();
                let exists = self
                    .traits
                    .iter()
                    .any(|(k, _)| k.to_lowercase() == name_lower);
                Ok(Value::Bool(exists))
            }
            "require" => self.require(args),
            "require_once" => self.require_once(args),
            "load_psr4_class" => {
                if args.is_empty() {
                    return Err("load_psr4_class() expects 1 argument".to_string());
                }
                let class_name = args[0].to_string_val();
                match self.load_psr4_class(&class_name) {
                    Ok(true) => Ok(Value::Bool(true)),
                    Ok(false) => Ok(Value::Bool(false)),
                    Err(e) => Err(e),
                }
            }
            _ => builtins::call_builtin(func_name, args, &mut self.output),
        }
    }

    /// require - Include and evaluate a PHP file
    /// Returns the return value of the included file, or false on failure
    pub fn require(&mut self, args: &[Value]) -> Result<Value, String> {
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        use std::fs;

        if args.is_empty() {
            return Err("require() expects at least 1 argument".to_string());
        }

        let filename = args[0].to_string_val();

        let source = match fs::read_to_string(&filename) {
            Ok(content) => content,
            Err(e) => {
                return Err(format!("require(): Failed to open '{}': {}", filename, e));
            }
        };

        let mut lexer = Lexer::new(&source);
        let tokens = lexer
            .tokenize()
            .map_err(|e| format!("Lexing error in {}: {}", filename, e))?;

        let mut parser = Parser::new(tokens);
        let program = parser
            .parse()
            .map_err(|e| format!("Parse error in {}: {}", filename, e))?;

        let compiler = crate::vm::compiler::Compiler::new(filename.clone());
        let compilation = compiler
            .compile_program(&program)
            .map_err(|e| format!("Compilation error in {}: {}", filename, e))?;

        for (name, func) in compilation.functions {
            self.functions.entry(name).or_insert(func);
        }

        for (name, class) in compilation.classes {
            self.classes.entry(name).or_insert(class);
        }

        for (name, interface) in compilation.interfaces {
            self.interfaces.entry(name).or_insert(interface);
        }

        for (name, trait_) in compilation.traits {
            self.traits.entry(name).or_insert(trait_);
        }

        for (name, enum_) in compilation.enums {
            self.enums.entry(name).or_insert(enum_);
        }

        // Execute the file's main function
        let result = self.execute_simple_function(&compilation.main);
        result.map_err(|e| format!("Runtime error in {}: {}", filename, e))
    }

    /// require_once - Include and evaluate a PHP file only once
    /// Returns the return value of the included file, or false on failure
    /// If the file has already been included, returns true without re-including
    pub fn require_once(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("require_once() expects at least 1 argument".to_string());
        }

        let filename = args[0].to_string_val();

        // Check if already required
        let required_files = REQUIRED_FILES.lock().unwrap();
        if required_files.contains(&filename) {
            drop(required_files);
            return Ok(Value::Bool(true));
        }
        drop(required_files);

        // Mark as required before including (in case of error, still marked)
        {
            let mut required_files = REQUIRED_FILES.lock().unwrap();
            required_files.insert(filename.clone());
        }

        // Call require to do the actual inclusion
        self.require(args)
    }

    /// Execute a function's bytecode without using the full VM loop
    /// This is used by require() to execute file bytecode
    fn execute_simple_function(
        &mut self,
        function: &std::sync::Arc<crate::vm::opcode::CompiledFunction>,
    ) -> Result<Value, String> {
        use crate::vm::frame::CallFrame;

        let stack_base = self.stack.len();
        let frame = CallFrame::new(function.clone(), stack_base);
        self.frames.push(frame);

        loop {
            // Check if we've returned to caller (our frame was popped)
            if self.frames.len() <= stack_base.saturating_sub(1) {
                return Ok(self.stack.pop().unwrap_or(Value::Null));
            }

            let frame = match self.frames.last_mut() {
                Some(f) => f,
                None => {
                    return Ok(self.stack.pop().unwrap_or(Value::Null));
                }
            };

            if frame.ip >= frame.function.bytecode.len() {
                let returned = self.stack.pop().unwrap_or(Value::Null);
                self.frames.pop();
                return Ok(returned);
            }

            let opcode = frame.function.bytecode[frame.ip].clone();
            frame.ip += 1;

            match self.execute_opcode(opcode) {
                Ok(()) => {}
                Err(e) => {
                    if e.starts_with("__RETURN__") {
                        let returned = if e == "__RETURN__" {
                            self.stack.pop().unwrap_or(Value::Null)
                        } else {
                            let value_str = e.strip_prefix("__RETURN__").unwrap();
                            if value_str == "null" {
                                Value::Null
                            } else {
                                self.stack.pop().unwrap_or(Value::Null)
                            }
                        };
                        self.frames.pop();
                        return Ok(returned);
                    } else if e.starts_with("__BREAK__") {
                        return Err("Cannot break outside of loop".to_string());
                    } else if e.starts_with("__CONTINUE__") {
                        return Err("Cannot continue outside of loop".to_string());
                    } else if e.starts_with("__EXCEPTION__") {
                        return Err(e);
                    } else if e == "__FINALLY_RETURN__" {
                        if let Some(value) = self.pending_return.take() {
                            self.frames.pop();
                            return Ok(value);
                        }
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Load a class using PSR-4 autoloading
    ///
    /// This function finds the PSR-4 mapping for the given class name,
    /// reads the corresponding file, compiles it, and registers the class.
    ///
    /// # Arguments
    /// * `class_name` - The fully qualified class name
    ///
    /// # Returns
    /// Ok(true) if the class was loaded, Ok(false) if no mapping was found,
    /// or an error if loading failed
    pub fn load_psr4_class(&mut self, class_name: &str) -> Result<bool, String> {
        use crate::runtime::builtins::spl;
        use crate::vm::compiler::Compiler;

        let normalized = spl::normalize_class_name(class_name);

        if let Some((prefix, base_dir)) = spl::find_psr4_mapping(&normalized) {
            let file_path = spl::namespace_to_path(&normalized, &base_dir, &prefix);

            let source = match std::fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(e) => {
                    return Err(format!(
                        "load_psr4_class(): Failed to open '{}': {}",
                        file_path, e
                    ));
                }
            };

            let mut lexer = crate::lexer::Lexer::new(&source);
            let tokens = lexer
                .tokenize()
                .map_err(|e| format!("Lexing error in {}: {}", file_path, e))?;

            let mut parser = crate::parser::Parser::new(tokens);
            let program = parser
                .parse()
                .map_err(|e| format!("Parse error in {}: {}", file_path, e))?;

            let compiler = Compiler::new(file_path.clone());
            let compilation = compiler
                .compile_program(&program)
                .map_err(|e| format!("Compilation error in {}: {}", file_path, e))?;

            for (name, func) in compilation.functions {
                self.functions.insert(name, func);
            }

            for (name, class) in compilation.classes {
                self.classes.insert(name, class);
            }

            for (name, interface) in compilation.interfaces {
                self.interfaces.insert(name, interface);
            }

            for (name, trait_) in compilation.traits {
                self.traits.insert(name, trait_);
            }

            for (name, enum_) in compilation.enums {
                self.enums.insert(name, enum_);
            }

            // Execute the file's main function
            if let Err(e) = self.execute_simple_function(&compilation.main) {
                return Err(format!("Runtime error in {}: {}", file_path, e));
            }

            return Ok(true);
        }

        Ok(false)
    }
}

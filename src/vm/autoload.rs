//! Autoloader support for VM
//!
//! This module provides helper methods for calling PHP functions,
//! methods, and closures from the autoloader context.

use crate::runtime::{Closure, Value};
use crate::vm::frame::CallFrame;

use super::VM;

impl<W: std::io::Write> VM<W> {
    /// Call a named function with arguments (used by autoloader)
    /// Returns the function result or an error
    pub fn call_function(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let normalized = name.trim_start_matches('\\').to_string();

        if let Some(func) = self.get_function(&normalized) {
            for arg in args.iter().rev() {
                self.stack.push(arg.clone());
            }

            let stack_base = self.stack.len() - args.len();
            let mut frame = CallFrame::new(func.clone(), stack_base);

            for (i, arg) in args.iter().enumerate() {
                if i < frame.locals.len() {
                    frame.locals[i] = arg.clone();
                }
            }

            self.frames.push(frame);
            let result = self.execute_function()?;
            self.frames.pop();

            Ok(result)
        } else {
            Err(format!("Function '{}' not found", name))
        }
    }

    /// Call a closure with arguments (used by autoloader)
    /// Returns the closure result or an error
    pub fn call_closure(&mut self, closure: &Closure, args: &[Value]) -> Result<Value, String> {
        let func_name = match &closure.body {
            crate::runtime::ClosureBody::FunctionRef(name) => name.clone(),
            crate::runtime::ClosureBody::MethodRef {
                class_name,
                method_name,
            } => format!("{}::{}", class_name, method_name),
            crate::runtime::ClosureBody::StaticMethodRef {
                class_name,
                method_name,
            } => format!("{}::{}", class_name, method_name),
            crate::runtime::ClosureBody::Expression(_) => {
                return Err("Expression closures not yet supported in autoloader".to_string());
            }
        };

        for arg in args.iter().rev() {
            self.stack.push(arg.clone());
        }

        if let Some(func) = self.get_function(&func_name) {
            let stack_base = self.stack.len() - args.len();
            let mut frame = CallFrame::new(func.clone(), stack_base);

            for (i, arg) in args.iter().enumerate() {
                if i < frame.locals.len() {
                    frame.locals[i] = arg.clone();
                }
            }

            for (var_name, value) in &closure.captured_vars {
                let slot = frame
                    .function
                    .local_names
                    .iter()
                    .position(|name| name == var_name)
                    .map(|i| i as u16);

                if let Some(slot) = slot {
                    if (slot as usize) < frame.locals.len() {
                        let captured_value: Value = value.clone();
                        frame.locals[slot as usize] = captured_value;
                    }
                }
            }

            self.frames.push(frame);
            let result = self.execute_function()?;
            self.frames.pop();

            Ok(result)
        } else {
            Err(format!("Closure function '{}' not found", func_name))
        }
    }

    /// Execute the current top frame's function
    fn execute_function(&mut self) -> Result<Value, String> {
        loop {
            let frame = self.frames.last_mut().expect("No call frame");
            let ip = frame.ip;

            if ip >= frame.function.bytecode.len() {
                return Ok(Value::Null);
            }

            let opcode = frame.function.bytecode[frame.ip].clone();
            frame.ip += 1;

            match self.execute_opcode(opcode) {
                Ok(()) => {}
                Err(e) => {
                    if e.starts_with("__RETURN__") {
                        return Ok(Value::Null);
                    }
                    return Err(e);
                }
            }
        }
    }
}

//! Bytecode Virtual Machine for VHP
//!
//! This module implements a stack-based bytecode VM that executes
//! compiled PHP bytecode. The VM is designed to be faster than
//! tree-walking interpretation for hot paths and repeated execution.

pub mod builtins;
pub mod class;
pub mod compiler;
pub mod frame;
pub mod opcode;

use crate::interpreter::{ArrayKey, Interpreter, Value};
use class::{CompiledClass, CompiledEnum, CompiledInterface, CompiledTrait};
use frame::{CallFrame, ExceptionHandler, LoopContext};
use opcode::{CastType, CompiledFunction, Constant, Opcode};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

/// The bytecode virtual machine
pub struct VM<W: Write> {
    /// Value stack for operands
    stack: Vec<Value>,
    /// Call frame stack
    frames: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Loop contexts for break/continue
    loops: Vec<LoopContext>,
    /// Exception handlers for try/catch/finally
    handlers: Vec<ExceptionHandler>,
    /// Output writer
    output: W,
    /// Reference to interpreter for built-in functions and classes
    interpreter: *mut Interpreter<W>,
    /// User-defined functions
    functions: HashMap<String, Arc<CompiledFunction>>,
    /// Class definitions
    classes: HashMap<String, Arc<CompiledClass>>,
    /// Interface definitions
    interfaces: HashMap<String, Arc<CompiledInterface>>,
    /// Trait definitions
    traits: HashMap<String, Arc<CompiledTrait>>,
    /// Enum definitions
    enums: HashMap<String, Arc<CompiledEnum>>,
}

impl<W: Write> VM<W> {
    /// Create a new VM instance
    pub fn new(output: W, interpreter: *mut Interpreter<W>) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            loops: Vec::new(),
            handlers: Vec::new(),
            output,
            interpreter,
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    /// Register user-defined functions
    pub fn register_functions(&mut self, functions: HashMap<String, Arc<CompiledFunction>>) {
        self.functions = functions;
    }

    /// Register class definitions
    pub fn register_classes(&mut self, classes: HashMap<String, Arc<CompiledClass>>) {
        self.classes = classes;
    }

    /// Register interface definitions
    pub fn register_interfaces(&mut self, interfaces: HashMap<String, Arc<CompiledInterface>>) {
        self.interfaces = interfaces;
    }

    /// Register trait definitions
    pub fn register_traits(&mut self, traits: HashMap<String, Arc<CompiledTrait>>) {
        self.traits = traits;
    }

    /// Register enum definitions
    pub fn register_enums(&mut self, enums: HashMap<String, Arc<CompiledEnum>>) {
        self.enums = enums;
    }

    /// Execute a compiled function
    pub fn execute(&mut self, function: Arc<CompiledFunction>) -> Result<Value, String> {
        // Create initial call frame
        let frame = CallFrame::new(function.clone(), 0);
        self.frames.push(frame);

        // Main execution loop
        loop {
            // Get current frame
            let frame = match self.frames.last_mut() {
                Some(f) => f,
                None => {
                    // No more frames, return top of stack or null
                    return Ok(self.stack.pop().unwrap_or(Value::Null));
                }
            };

            // Check if we've reached the end of the function
            if frame.ip >= frame.function.bytecode.len() {
                // Pop frame and continue with caller
                let returned = self.stack.pop().unwrap_or(Value::Null);
                self.frames.pop();

                if self.frames.is_empty() {
                    return Ok(returned);
                }

                // Push return value for caller
                self.stack.push(returned);
                continue;
            }

            // Fetch instruction
            let opcode = frame.function.bytecode[frame.ip].clone();
            frame.ip += 1;

            // Execute instruction
            match self.execute_opcode(opcode) {
                Ok(()) => {}
                Err(e) => {
                    // Check for control flow signals
                    if e.starts_with("__RETURN__") {
                        // Return from function
                        let value_str = e.strip_prefix("__RETURN__").unwrap();
                        let value = if value_str == "null" {
                            Value::Null
                        } else {
                            self.stack.pop().unwrap_or(Value::Null)
                        };

                        self.frames.pop();
                        if self.frames.is_empty() {
                            return Ok(value);
                        }
                        self.stack.push(value);
                        continue;
                    } else if e.starts_with("__BREAK__") {
                        // Handle break
                        if let Some(loop_ctx) = self.loops.last() {
                            let break_target = loop_ctx.break_target as usize;
                            if let Some(frame) = self.frames.last_mut() {
                                frame.jump_to(break_target);
                            }
                        } else {
                            return Err("Cannot break outside of loop".to_string());
                        }
                        continue;
                    } else if e.starts_with("__CONTINUE__") {
                        // Handle continue
                        if let Some(loop_ctx) = self.loops.last() {
                            let continue_target = loop_ctx.continue_target as usize;
                            if let Some(frame) = self.frames.last_mut() {
                                frame.jump_to(continue_target);
                            }
                        } else {
                            return Err("Cannot continue outside of loop".to_string());
                        }
                        continue;
                    } else if e.starts_with("__EXCEPTION__") {
                        // Handle exception - look for handler
                        let parts: Vec<&str> = e.splitn(3, ':').collect();
                        if parts.len() >= 3 {
                            let _class = parts[1];
                            let _message = parts[2];
                            // TODO: Implement exception handling
                            return Err(e);
                        }
                        return Err(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    /// Execute a single opcode
    fn execute_opcode(&mut self, opcode: Opcode) -> Result<(), String> {
        match opcode {
            // ==================== Literals & Constants ====================
            Opcode::PushNull => {
                self.stack.push(Value::Null);
            }
            Opcode::PushTrue => {
                self.stack.push(Value::Bool(true));
            }
            Opcode::PushFalse => {
                self.stack.push(Value::Bool(false));
            }
            Opcode::PushInt(n) => {
                self.stack.push(Value::Integer(n));
            }
            Opcode::PushFloat(f) => {
                self.stack.push(Value::Float(f));
            }
            Opcode::PushString(idx) => {
                let s = self.current_frame().get_string(idx).to_string();
                self.stack.push(Value::String(s));
            }
            Opcode::LoadConst(idx) => {
                let constant = self.current_frame().get_constant(idx).clone();
                let value = match constant {
                    Constant::Null => Value::Null,
                    Constant::Bool(b) => Value::Bool(b),
                    Constant::Int(n) => Value::Integer(n),
                    Constant::Float(f) => Value::Float(f),
                    Constant::String(s) => Value::String(s),
                };
                self.stack.push(value);
            }

            // ==================== Variables ====================
            Opcode::LoadVar(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.globals.get(&name).cloned().unwrap_or(Value::Null);
                self.stack.push(value);
            }
            Opcode::StoreVar(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.globals.insert(name, value);
            }
            Opcode::LoadFast(slot) => {
                let value = self.current_frame().get_local(slot).clone();
                self.stack.push(value);
            }
            Opcode::StoreFast(slot) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.current_frame_mut().set_local(slot, value);
            }
            Opcode::LoadGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.globals.get(&name).cloned().unwrap_or(Value::Null);
                self.stack.push(value);
            }
            Opcode::StoreGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.globals.insert(name, value);
            }

            // ==================== Arithmetic ====================
            Opcode::Add => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.add_values(left, right)?;
                self.stack.push(result);
            }
            Opcode::Sub => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = match (&left, &right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
                    _ => {
                        let a = left.to_float();
                        let b = right.to_float();
                        Value::Float(a - b)
                    }
                };
                self.stack.push(result);
            }
            Opcode::Mul => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = match (&left, &right) {
                    (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
                    (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                    (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 * b),
                    (Value::Float(a), Value::Integer(b)) => Value::Float(a * *b as f64),
                    _ => {
                        let a = left.to_float();
                        let b = right.to_float();
                        Value::Float(a * b)
                    }
                };
                self.stack.push(result);
            }
            Opcode::Div => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let a = left.to_float();
                let b = right.to_float();
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(Value::Float(a / b));
            }
            Opcode::Mod => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let a = left.to_int();
                let b = right.to_int();
                if b == 0 {
                    return Err("Modulo by zero".to_string());
                }
                self.stack.push(Value::Integer(a % b));
            }
            Opcode::Pow => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let base = left.to_float();
                let exp = right.to_float();
                self.stack.push(Value::Float(base.powf(exp)));
            }
            Opcode::Neg => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let result = match value {
                    Value::Integer(n) => Value::Integer(-n),
                    Value::Float(f) => Value::Float(-f),
                    _ => Value::Integer(-value.to_int()),
                };
                self.stack.push(result);
            }

            // ==================== String Operations ====================
            Opcode::Concat => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = Value::String(format!("{}{}", left.to_string_val(), right.to_string_val()));
                self.stack.push(result);
            }

            // ==================== Comparison ====================
            Opcode::Eq => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(left.loose_equals(&right)));
            }
            Opcode::Ne => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(!left.loose_equals(&right)));
            }
            Opcode::Identical => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(left.type_equals(&right)));
            }
            Opcode::NotIdentical => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(!left.type_equals(&right)));
            }
            Opcode::Lt => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.compare_values(&left, &right)? < 0;
                self.stack.push(Value::Bool(result));
            }
            Opcode::Le => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.compare_values(&left, &right)? <= 0;
                self.stack.push(Value::Bool(result));
            }
            Opcode::Gt => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.compare_values(&left, &right)? > 0;
                self.stack.push(Value::Bool(result));
            }
            Opcode::Ge => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.compare_values(&left, &right)? >= 0;
                self.stack.push(Value::Bool(result));
            }
            Opcode::Spaceship => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = self.compare_values(&left, &right)?;
                self.stack.push(Value::Integer(result));
            }

            // ==================== Logical ====================
            Opcode::Not => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(!value.to_bool()));
            }
            Opcode::And => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(left.to_bool() && right.to_bool()));
            }
            Opcode::Or => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack.push(Value::Bool(left.to_bool() || right.to_bool()));
            }

            // ==================== Control Flow ====================
            Opcode::Jump(offset) => {
                self.current_frame_mut().jump_to(offset as usize);
            }
            Opcode::JumpIfFalse(offset) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                if !value.to_bool() {
                    self.current_frame_mut().jump_to(offset as usize);
                }
            }
            Opcode::JumpIfTrue(offset) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                if value.to_bool() {
                    self.current_frame_mut().jump_to(offset as usize);
                }
            }
            Opcode::JumpIfNull(offset) => {
                let value = self.stack.last().ok_or("Stack underflow")?;
                if matches!(value, Value::Null) {
                    self.current_frame_mut().jump_to(offset as usize);
                }
            }
            Opcode::JumpIfNotNull(offset) => {
                let value = self.stack.last().ok_or("Stack underflow")?;
                if !matches!(value, Value::Null) {
                    self.current_frame_mut().jump_to(offset as usize);
                }
            }
            Opcode::Return => {
                return Err("__RETURN__".to_string());
            }
            Opcode::ReturnNull => {
                return Err("__RETURN__null".to_string());
            }

            // ==================== Loop Control ====================
            Opcode::Break => {
                return Err("__BREAK__".to_string());
            }
            Opcode::Continue => {
                return Err("__CONTINUE__".to_string());
            }
            Opcode::LoopStart(continue_target, break_target) => {
                self.loops.push(LoopContext {
                    continue_target,
                    break_target,
                    stack_depth: self.stack.len(),
                });
            }
            Opcode::LoopEnd => {
                self.loops.pop();
            }

            // ==================== Arrays ====================
            Opcode::NewArray(count) => {
                let mut arr = Vec::new();
                for _ in 0..count {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let key = self.stack.pop().ok_or("Stack underflow")?;
                    arr.push((ArrayKey::from_value(&key), value));
                }
                arr.reverse();
                self.stack.push(Value::Array(arr));
            }
            Opcode::ArrayGet => {
                let key = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match array {
                    Value::Array(arr) => {
                        let array_key = ArrayKey::from_value(&key);
                        let value = arr
                            .iter()
                            .find(|(k, _)| k == &array_key)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Null);
                        self.stack.push(value);
                    }
                    _ => return Err("Cannot use [] on non-array".to_string()),
                }
            }
            Opcode::ArraySet => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let key = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match array {
                    Value::Array(mut arr) => {
                        let array_key = ArrayKey::from_value(&key);
                        // Update existing key or append
                        if let Some(pos) = arr.iter().position(|(k, _)| k == &array_key) {
                            arr[pos] = (array_key, value);
                        } else {
                            arr.push((array_key, value));
                        }
                        self.stack.push(Value::Array(arr));
                    }
                    _ => return Err("Cannot use [] on non-array".to_string()),
                }
            }
            Opcode::ArrayAppend => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match array {
                    Value::Array(mut arr) => {
                        let next_idx = arr
                            .iter()
                            .filter_map(|(k, _)| match k {
                                ArrayKey::Integer(n) => Some(*n),
                                _ => None,
                            })
                            .max()
                            .unwrap_or(-1)
                            + 1;
                        arr.push((ArrayKey::Integer(next_idx), value));
                        self.stack.push(Value::Array(arr));
                    }
                    _ => return Err("Cannot append to non-array".to_string()),
                }
            }
            Opcode::ArrayCount => {
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match array {
                    Value::Array(arr) => {
                        self.stack.push(Value::Integer(arr.len() as i64));
                    }
                    _ => self.stack.push(Value::Integer(0)),
                }
            }
            Opcode::ArrayGetKeyAt => {
                let index = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match (array, index) {
                    (Value::Array(arr), Value::Integer(idx)) => {
                        if idx >= 0 && (idx as usize) < arr.len() {
                            let (key, _) = &arr[idx as usize];
                            self.stack.push(key.to_value());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    _ => self.stack.push(Value::Null),
                }
            }
            Opcode::ArrayGetValueAt => {
                let index = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;
                match (array, index) {
                    (Value::Array(arr), Value::Integer(idx)) => {
                        if idx >= 0 && (idx as usize) < arr.len() {
                            let (_, value) = &arr[idx as usize];
                            self.stack.push(value.clone());
                        } else {
                            self.stack.push(Value::Null);
                        }
                    }
                    _ => self.stack.push(Value::Null),
                }
            }

            // ==================== Stack Manipulation ====================
            Opcode::Pop => {
                self.stack.pop().ok_or("Stack underflow")?;
            }
            Opcode::Dup => {
                let value = self.stack.last().ok_or("Stack underflow")?.clone();
                self.stack.push(value);
            }
            Opcode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err("Stack underflow".to_string());
                }
                self.stack.swap(len - 1, len - 2);
            }

            // ==================== Type Operations ====================
            Opcode::Cast(cast_type) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let result = match cast_type {
                    CastType::Int => Value::Integer(value.to_int()),
                    CastType::Float => Value::Float(value.to_float()),
                    CastType::String => Value::String(value.to_string_val()),
                    CastType::Bool => Value::Bool(value.to_bool()),
                    CastType::Array => match value {
                        Value::Array(_) => value,
                        _ => return Err("Cannot cast to array".to_string()),
                    },
                    CastType::Object => match value {
                        Value::Object(_) => value,
                        _ => return Err("Cannot cast to object".to_string()),
                    },
                };
                self.stack.push(result);
            }

            // ==================== Null Coalescing ====================
            Opcode::NullCoalesce => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                let result = if matches!(left, Value::Null) {
                    right
                } else {
                    left
                };
                self.stack.push(result);
            }

            // ==================== Output ====================
            Opcode::Echo => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                write!(self.output, "{}", value.to_output_string()).map_err(|e| e.to_string())?;
            }
            Opcode::Print => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                write!(self.output, "{}", value.to_output_string()).map_err(|e| e.to_string())?;
                self.stack.push(Value::Integer(1));
            }

            // ==================== Function Calls ====================
            Opcode::Call(name_idx, arg_count) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop arguments from stack
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse(); // Arguments were pushed in order, popped in reverse

                // 1. Check user-defined functions first
                if let Some(func) = self.functions.get(&func_name).cloned() {
                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(func, stack_base);

                    // Set up parameter locals
                    for (i, arg) in args.into_iter().enumerate() {
                        if i < frame.locals.len() {
                            frame.locals[i] = arg;
                        }
                    }

                    self.frames.push(frame);
                }
                // 2. Fall back to built-in functions
                else if builtins::is_builtin(&func_name) {
                    let result = builtins::call_builtin(&func_name, &args, &mut self.output)?;
                    self.stack.push(result);
                }
                // 3. Unknown function
                else {
                    return Err(format!("Undefined function: {}", func_name));
                }
            }

            Opcode::CallBuiltin(name_idx, arg_count) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop arguments from stack
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Call the built-in function
                let result = builtins::call_builtin(&func_name, &args, &mut self.output)?;
                self.stack.push(result);
            }

            // ==================== OOP Opcodes ====================
            Opcode::NewObject(class_idx) => {
                let class_name = self.current_frame().get_string(class_idx).to_string();

                // Look up class definition
                let class_def = self.classes.get(&class_name)
                    .ok_or_else(|| format!("Class '{}' not found", class_name))?
                    .clone();

                if class_def.is_abstract {
                    return Err(format!("Cannot instantiate abstract class {}", class_name));
                }

                // Create new object instance
                let mut instance = crate::interpreter::ObjectInstance::with_hierarchy(
                    class_name.clone(),
                    class_def.parent.clone(),
                    class_def.interfaces.clone(),
                );

                // Initialize properties with defaults
                for prop in &class_def.properties {
                    let default_val = prop.default.clone().unwrap_or(Value::Null);
                    instance.properties.insert(prop.name.clone(), default_val);
                    if prop.readonly {
                        instance.readonly_properties.insert(prop.name.clone());
                    }
                }

                // TODO: Call constructor if present
                // For now, just push the object
                self.stack.push(Value::Object(instance));
            }

            Opcode::LoadProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        let value = instance.properties.get(&prop_name)
                            .cloned()
                            .unwrap_or(Value::Null);
                        self.stack.push(value);
                    }
                    _ => return Err(format!("Cannot access property of non-object: {:?}", object)),
                }
            }

            Opcode::StoreProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(mut instance) => {
                        // Check readonly constraint
                        if instance.readonly_properties.contains(&prop_name)
                            && instance.initialized_readonly.contains(&prop_name) {
                            return Err(format!("Cannot modify readonly property {}", prop_name));
                        }
                        instance.properties.insert(prop_name.clone(), value.clone());
                        if instance.readonly_properties.contains(&prop_name) {
                            instance.initialized_readonly.insert(prop_name);
                        }
                        self.stack.push(Value::Object(instance));
                    }
                    _ => return Err("Cannot set property on non-object".to_string()),
                }
            }

            Opcode::CallMethod(method_idx, arg_count) => {
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Pop object
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        let class_name = instance.class_name.clone();
                        let class_def = self.classes.get(&class_name)
                            .ok_or_else(|| format!("Class '{}' not found", class_name))?
                            .clone();

                        // Find method
                        let method = class_def.get_method(&method_name)
                            .ok_or_else(|| format!("Method '{}' not found on class '{}'", method_name, class_name))?
                            .clone();

                        // Create new call frame with $this as first local
                        let stack_base = self.stack.len();
                        let mut frame = CallFrame::new(method, stack_base);

                        // Set $this (slot 0)
                        frame.locals[0] = Value::Object(instance);

                        // Set up parameter locals (starting from slot 1)
                        for (i, arg) in args.into_iter().enumerate() {
                            if i + 1 < frame.locals.len() {
                                frame.locals[i + 1] = arg;
                            }
                        }

                        self.frames.push(frame);
                    }
                    _ => return Err("Cannot call method on non-object".to_string()),
                }
            }

            Opcode::CallStaticMethod(class_idx, method_idx, arg_count) => {
                let class_name = self.current_frame().get_string(class_idx).to_string();
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Handle self/static/parent keywords
                let resolved_class = if class_name == "self" || class_name == "static" {
                    // TODO: Get current class from context
                    class_name.clone()
                } else {
                    class_name.clone()
                };

                let class_def = self.classes.get(&resolved_class)
                    .ok_or_else(|| format!("Class '{}' not found", resolved_class))?
                    .clone();

                // Find static method
                let method = class_def.static_methods.get(&method_name)
                    .ok_or_else(|| format!("Static method '{}' not found on class '{}'", method_name, resolved_class))?
                    .clone();

                // Create new call frame (no $this for static methods)
                let stack_base = self.stack.len();
                let mut frame = CallFrame::new(method, stack_base);

                // Set up parameter locals
                for (i, arg) in args.into_iter().enumerate() {
                    if i < frame.locals.len() {
                        frame.locals[i] = arg;
                    }
                }

                self.frames.push(frame);
            }

            Opcode::LoadStaticProp(class_idx, prop_idx) => {
                let class_name = self.current_frame().get_string(class_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();

                let class_def = self.classes.get(&class_name)
                    .ok_or_else(|| format!("Class '{}' not found", class_name))?;

                let value = class_def.static_properties.get(&prop_name)
                    .cloned()
                    .unwrap_or(Value::Null);
                self.stack.push(value);
            }

            Opcode::StoreStaticProp(class_idx, prop_idx) => {
                let class_name = self.current_frame().get_string(class_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;

                // Need mutable access to update static property
                let class_def = self.classes.get_mut(&class_name)
                    .ok_or_else(|| format!("Class '{}' not found", class_name))?;
                Arc::make_mut(class_def).static_properties.insert(prop_name, value);
            }

            Opcode::LoadThis => {
                // $this is stored in slot 0 for instance methods
                let frame = self.current_frame();
                let this = frame.locals.get(0)
                    .cloned()
                    .ok_or("No $this available in current context")?;
                self.stack.push(this);
            }

            Opcode::InstanceOf(class_idx) => {
                let class_name = self.current_frame().get_string(class_idx).to_string();
                let object = self.stack.pop().ok_or("Stack underflow")?;

                let result = match object {
                    Value::Object(instance) => {
                        instance.class_name == class_name
                            || instance.parent_class.as_ref() == Some(&class_name)
                            || instance.interfaces.contains(&class_name)
                    }
                    _ => false,
                };
                self.stack.push(Value::Bool(result));
            }

            Opcode::Clone => {
                let object = self.stack.pop().ok_or("Stack underflow")?;
                match object {
                    Value::Object(instance) => {
                        // Clone the object
                        let cloned = instance.clone();
                        // TODO: Call __clone if present
                        self.stack.push(Value::Object(cloned));
                    }
                    _ => return Err("Cannot clone non-object".to_string()),
                }
            }

            // ==================== Exception Handling ====================
            Opcode::Throw => {
                let exception = self.stack.pop().ok_or("Stack underflow")?;
                // For now, just return as error
                return Err(format!("Uncaught exception: {:?}", exception));
            }

            Opcode::TryStart(_, _) | Opcode::TryEnd | Opcode::FinallyStart | Opcode::FinallyEnd => {
                // Basic exception handling - just continue execution
                // Full implementation would track exception handlers
            }

            // ==================== Closures ====================
            Opcode::CreateClosure(func_idx, _capture_count) => {
                let func_name = self.current_frame().get_string(func_idx).to_string();
                // For now, store closure as a callable string
                // Full implementation would capture variables
                self.stack.push(Value::String(func_name));
            }

            // ==================== Array Operations ====================
            Opcode::ArrayUnpack => {
                let array = self.stack.pop().ok_or("Stack underflow")?;
                // Unpack array into individual elements
                // This is used for spread operator
                if let Value::Array(elements) = array {
                    for (_, value) in elements {
                        self.stack.push(value);
                    }
                }
            }

            // ==================== Not Yet Implemented ====================
            _ => {
                return Err(format!("Opcode not yet implemented: {:?}", opcode));
            }
        }

        Ok(())
    }

    /// Helper: Add two values (handles numeric and array concatenation)
    fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
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

    /// Helper: Compare two values (returns -1, 0, or 1)
    fn compare_values(&self, left: &Value, right: &Value) -> Result<i64, String> {
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

    /// Get the current call frame (immutable)
    #[inline]
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("No call frame available")
    }

    /// Get the current call frame (mutable)
    #[inline]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("No call frame available")
    }
}

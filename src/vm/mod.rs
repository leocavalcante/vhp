//! Bytecode Virtual Machine for VHP
//!
//! This module implements a stack-based bytecode VM that executes
//! compiled PHP bytecode. The VM is designed to be faster than
//! tree-walking interpretation for hot paths and repeated execution.

pub mod builtins;
pub mod class;
pub mod compiler;
pub mod frame;
pub mod methods;
pub mod objects;
pub mod opcode;
pub mod reflection;

mod ops;

use crate::runtime::{ArrayKey, ClosureBody, Value};
use class::{CompiledClass, CompiledEnum, CompiledInterface, CompiledProperty, CompiledTrait};
use frame::{CallFrame, ExceptionHandler, LoopContext, ThisSource};
use opcode::{CastType, CompiledFunction, Constant, Opcode};
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

/// The bytecode virtual machine
#[allow(dead_code)] // current_fiber field not yet used
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
    /// Pending return value (saved while executing finally block)
    pending_return: Option<Value>,
    /// Current running fiber (for Fiber::getCurrent())
    current_fiber: Option<Value>,
    /// Output writer
    output: W,
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
    pub fn new(output: W) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            frames: Vec::with_capacity(64),
            globals: HashMap::new(),
            loops: Vec::new(),
            handlers: Vec::new(),
            pending_return: None,
            current_fiber: None,
            output,
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

    /// Register class definitions (merges with existing built-in classes)
    pub fn register_classes(&mut self, classes: HashMap<String, Arc<CompiledClass>>) {
        // Merge user classes into existing (preserves built-ins)
        for (name, class) in classes {
            self.classes.insert(name, class);
        }
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

    /// Register built-in classes like Exception
    pub fn register_builtins(&mut self) {
        // Register Exception class with properties and methods
        let mut exception = CompiledClass::new("Exception".to_string());

        // Add message property
        exception.properties.push(CompiledProperty {
            name: "message".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::String(String::new())),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add code property
        exception.properties.push(CompiledProperty {
            name: "code".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Integer(0)),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Create __construct method: __construct(string $message = "", int $code = 0)
        let mut construct = CompiledFunction::new("Exception::__construct".to_string());
        construct.param_count = 2;
        construct.required_param_count = 0; // Both have defaults
        construct.local_count = 3; // $this, $message, $code
        construct.local_names = vec![
            "this".to_string(),
            "message".to_string(),
            "code".to_string(),
        ];

        // Bytecode for constructor:
        // Store $message to $this->message
        construct.bytecode.push(Opcode::LoadFast(1)); // Load $message
        construct.strings.push("message".to_string());
        construct.bytecode.push(Opcode::StoreThisProperty(0)); // Store to $this->message

        // Store $code to $this->code
        construct.bytecode.push(Opcode::LoadFast(2)); // Load $code
        construct.strings.push("code".to_string());
        construct.bytecode.push(Opcode::StoreThisProperty(1)); // Store to $this->code

        construct.bytecode.push(Opcode::ReturnNull);
        exception
            .methods
            .insert("__construct".to_string(), Arc::new(construct));

        // Create getMessage method
        let mut get_message = CompiledFunction::new("Exception::getMessage".to_string());
        get_message.param_count = 0;
        get_message.local_count = 1; // $this
        get_message.local_names = vec!["this".to_string()];

        // Bytecode: return $this->message
        get_message.strings.push("message".to_string());
        get_message.bytecode.push(Opcode::LoadThis);
        get_message.bytecode.push(Opcode::LoadProperty(0)); // Load $this->message
        get_message.bytecode.push(Opcode::Return);
        exception
            .methods
            .insert("getMessage".to_string(), Arc::new(get_message));

        // Create getCode method
        let mut get_code = CompiledFunction::new("Exception::getCode".to_string());
        get_code.param_count = 0;
        get_code.local_count = 1; // $this
        get_code.local_names = vec!["this".to_string()];

        // Bytecode: return $this->code
        get_code.strings.push("code".to_string());
        get_code.bytecode.push(Opcode::LoadThis);
        get_code.bytecode.push(Opcode::LoadProperty(0)); // Load $this->code
        get_code.bytecode.push(Opcode::Return);
        exception
            .methods
            .insert("getCode".to_string(), Arc::new(get_code));

        self.classes
            .insert("Exception".to_string(), Arc::new(exception));

        // Register Error class (same structure as Exception)
        let mut error = CompiledClass::new("Error".to_string());

        // Add message property
        error.properties.push(CompiledProperty {
            name: "message".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::String(String::new())),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add code property
        error.properties.push(CompiledProperty {
            name: "code".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Integer(0)),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Create __construct method
        let mut error_construct = CompiledFunction::new("Error::__construct".to_string());
        error_construct.param_count = 2;
        error_construct.required_param_count = 0;
        error_construct.local_count = 3;
        error_construct.local_names = vec![
            "this".to_string(),
            "message".to_string(),
            "code".to_string(),
        ];
        error_construct.bytecode.push(Opcode::LoadFast(1));
        error_construct.strings.push("message".to_string());
        error_construct.bytecode.push(Opcode::StoreThisProperty(0));
        error_construct.bytecode.push(Opcode::LoadFast(2));
        error_construct.strings.push("code".to_string());
        error_construct.bytecode.push(Opcode::StoreThisProperty(1));
        error_construct.bytecode.push(Opcode::ReturnNull);
        error
            .methods
            .insert("__construct".to_string(), Arc::new(error_construct));

        // Create getMessage method
        let mut error_get_message = CompiledFunction::new("Error::getMessage".to_string());
        error_get_message.param_count = 0;
        error_get_message.local_count = 1;
        error_get_message.local_names = vec!["this".to_string()];
        error_get_message.strings.push("message".to_string());
        error_get_message.bytecode.push(Opcode::LoadThis);
        error_get_message.bytecode.push(Opcode::LoadProperty(0));
        error_get_message.bytecode.push(Opcode::Return);
        error
            .methods
            .insert("getMessage".to_string(), Arc::new(error_get_message));

        // Create getCode method
        let mut error_get_code = CompiledFunction::new("Error::getCode".to_string());
        error_get_code.param_count = 0;
        error_get_code.local_count = 1;
        error_get_code.local_names = vec!["this".to_string()];
        error_get_code.strings.push("code".to_string());
        error_get_code.bytecode.push(Opcode::LoadThis);
        error_get_code.bytecode.push(Opcode::LoadProperty(0));
        error_get_code.bytecode.push(Opcode::Return);
        error
            .methods
            .insert("getCode".to_string(), Arc::new(error_get_code));

        self.classes.insert("Error".to_string(), Arc::new(error));

        // Register TypeError
        let mut type_error = CompiledClass::new("TypeError".to_string());
        type_error.parent = Some("Error".to_string());
        self.classes
            .insert("TypeError".to_string(), Arc::new(type_error));

        // Register InvalidArgumentException
        let mut invalid_arg = CompiledClass::new("InvalidArgumentException".to_string());
        invalid_arg.parent = Some("Exception".to_string());
        self.classes.insert(
            "InvalidArgumentException".to_string(),
            Arc::new(invalid_arg),
        );

        // Register UnhandledMatchError
        let mut unhandled_match = CompiledClass::new("UnhandledMatchError".to_string());
        unhandled_match.parent = Some("Error".to_string());
        self.classes
            .insert("UnhandledMatchError".to_string(), Arc::new(unhandled_match));

        // Register Fiber class
        let mut fiber = CompiledClass::new("Fiber".to_string());

        // Add callback property (stores the callable)
        fiber.properties.push(CompiledProperty {
            name: "__callback".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Null),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add started property (bool)
        fiber.properties.push(CompiledProperty {
            name: "__started".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Bool(false)),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add suspended property (bool)
        fiber.properties.push(CompiledProperty {
            name: "__suspended".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Bool(false)),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add terminated property (bool)
        fiber.properties.push(CompiledProperty {
            name: "__terminated".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Bool(false)),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // Add return_value property (null initially)
        fiber.properties.push(CompiledProperty {
            name: "__return_value".to_string(),
            visibility: crate::ast::Visibility::Private,
            write_visibility: None,
            default: Some(Value::Null),
            readonly: false,
            is_static: false,
            type_hint: None,
            attributes: Vec::new(),
            get_hook: None,
            set_hook: None,
        });

        // __construct method - stores the callback
        let mut construct = CompiledFunction::new("Fiber::__construct".to_string());
        construct.param_count = 1;
        construct.required_param_count = 1;
        construct.local_count = 2; // $this, $callback
        construct.local_names = vec!["this".to_string(), "callback".to_string()];

        // Store $callback to $this->__callback
        construct.bytecode.push(Opcode::LoadFast(1));
        construct.strings.push("__callback".to_string());
        construct.bytecode.push(Opcode::StoreThisProperty(0));
        construct.bytecode.push(Opcode::ReturnNull);
        fiber
            .methods
            .insert("__construct".to_string(), Arc::new(construct));

        // start() method - executes the callback
        let mut start = CompiledFunction::new("Fiber::start".to_string());
        start.param_count = 0;
        start.local_count = 1; // $this
        start.local_names = vec!["this".to_string()];

        // Mark as started and terminated (synchronous execution)
        start.strings.push("__started".to_string());
        start.bytecode.push(Opcode::PushTrue);
        start.strings.push("__started".to_string());
        start.bytecode.push(Opcode::LoadThis);
        start.bytecode.push(Opcode::StoreProperty(0)); // $this->__started = true

        start.strings.push("__terminated".to_string());
        start.bytecode.push(Opcode::PushTrue);
        start.strings.push("__terminated".to_string());
        start.bytecode.push(Opcode::LoadThis);
        start.bytecode.push(Opcode::StoreProperty(1)); // $this->__terminated = true

        // Load and call the callback
        start.strings.push("__callback".to_string());
        start.bytecode.push(Opcode::LoadThis);
        start.bytecode.push(Opcode::LoadProperty(2)); // Load $this->__callback
        start.strings.push("__callback".to_string());
        start.bytecode.push(Opcode::CallCallable(0)); // Call the callback with 0 args
                                                      // Stack now: [$this, result]

        // Store the return value - use local to preserve $this
        start.bytecode.push(Opcode::LoadFast(0)); // Load $this (preserves result)
                                                  // Stack now: [$this, result, $this]

        start.strings.push("__return_value".to_string());
        start.bytecode.push(Opcode::Swap); // Swap to get: [$this, $this, result]
        start.strings.push("__return_value".to_string()); // Stack now: [$this, result]

        start.bytecode.push(Opcode::StoreProperty(3)); // $this->__return_value = result
                                                       // Stack now: [$this]

        // Return the result - load it and return
        start.strings.push("__return_value".to_string());
        start.bytecode.push(Opcode::LoadProperty(3));
        start.strings.push("__return_value".to_string());
        start.bytecode.push(Opcode::Return);

        fiber.methods.insert("start".to_string(), Arc::new(start));

        // getReturn() method - returns stored return value
        let mut get_return = CompiledFunction::new("Fiber::getReturn".to_string());
        get_return.param_count = 0;
        get_return.local_count = 1; // $this
        get_return.local_names = vec!["this".to_string()];

        get_return.strings.push("__return_value".to_string());
        get_return.bytecode.push(Opcode::LoadThis);
        get_return.bytecode.push(Opcode::LoadProperty(0)); // Load $this->__return_value
        get_return.bytecode.push(Opcode::Return);
        fiber
            .methods
            .insert("getReturn".to_string(), Arc::new(get_return));

        // isStarted() method
        let mut is_started = CompiledFunction::new("Fiber::isStarted".to_string());
        is_started.param_count = 0;
        is_started.local_count = 1; // $this
        is_started.local_names = vec!["this".to_string()];

        is_started.strings.push("__started".to_string());
        is_started.bytecode.push(Opcode::LoadThis);
        is_started.bytecode.push(Opcode::LoadProperty(0)); // Load $this->__started
        is_started.bytecode.push(Opcode::Return);
        fiber
            .methods
            .insert("isStarted".to_string(), Arc::new(is_started));

        // isSuspended() method
        let mut is_suspended = CompiledFunction::new("Fiber::isSuspended".to_string());
        is_suspended.param_count = 0;
        is_suspended.local_count = 1; // $this
        is_suspended.local_names = vec!["this".to_string()];

        is_suspended.strings.push("__suspended".to_string());
        is_suspended.bytecode.push(Opcode::LoadThis);
        is_suspended.bytecode.push(Opcode::LoadProperty(0)); // Load $this->__suspended
        is_suspended.bytecode.push(Opcode::Return);
        fiber
            .methods
            .insert("isSuspended".to_string(), Arc::new(is_suspended));

        // isTerminated() method
        let mut is_terminated = CompiledFunction::new("Fiber::isTerminated".to_string());
        is_terminated.param_count = 0;
        is_terminated.local_count = 1; // $this
        is_terminated.local_names = vec!["this".to_string()];

        is_terminated.strings.push("__terminated".to_string());
        is_terminated.bytecode.push(Opcode::LoadThis);
        is_terminated.bytecode.push(Opcode::LoadProperty(0)); // Load $this->__terminated
        is_terminated.bytecode.push(Opcode::Return);
        fiber
            .methods
            .insert("isTerminated".to_string(), Arc::new(is_terminated));

        // Static method: getCurrent() - returns current running fiber or null
        let mut get_current = CompiledFunction::new("Fiber::getCurrent".to_string());
        get_current.param_count = 0;
        get_current.local_count = 0;

        // For now, always return null (we don't track current fiber in this implementation)
        get_current.bytecode.push(Opcode::PushNull);
        get_current.bytecode.push(Opcode::Return);
        fiber
            .static_methods
            .insert("getCurrent".to_string(), Arc::new(get_current));

        self.classes.insert("Fiber".to_string(), Arc::new(fiber));
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
                        // Get the frame before popping to check if it's a constructor
                        let frame = self.frames.last().expect("No frame");
                        let current_ip = frame.ip as u32;
                        let is_constructor = frame.is_constructor;
                        let this_source = frame.this_source.clone();
                        // Only get modified $this if this is a method call (has locals with $this in slot 0)
                        let modified_this = if !matches!(this_source, ThisSource::None)
                            && !frame.locals.is_empty()
                        {
                            Some(frame.locals[0].clone())
                        } else {
                            None
                        };

                        let value = if is_constructor {
                            // For constructors, return $this (slot 0)
                            frame.locals[0].clone()
                        } else if matches!(this_source, ThisSource::PropertySetHook) {
                            // For property set hooks, return the modified $this instead of return value
                            frame.locals[0].clone()
                        } else {
                            let value_str = e.strip_prefix("__RETURN__").unwrap();
                            if value_str == "null" {
                                Value::Null
                            } else {
                                self.stack.pop().unwrap_or(Value::Null)
                            }
                        };

                        // Check if there's a finally block to execute
                        // Find an active handler for this frame with a finally block
                        let finally_jump = self.handlers.iter().rev().find_map(|h| {
                            if h.frame_depth == self.frames.len()
                                && h.finally_offset > 0
                                && current_ip > h.try_start
                                && current_ip <= h.finally_offset
                            {
                                Some(h.finally_offset as usize)
                            } else {
                                None
                            }
                        });

                        if let Some(finally_offset) = finally_jump {
                            // Save the return value and jump to finally
                            self.pending_return = Some(value);
                            if let Some(frame) = self.frames.last_mut() {
                                frame.jump_to(finally_offset);
                            }
                            continue;
                        }

                        self.frames.pop();

                        // Update the source variable with modified $this
                        if let Some(modified) = modified_this {
                            match this_source {
                                ThisSource::LocalSlot(slot) => {
                                    if let Some(caller_frame) = self.frames.last_mut() {
                                        caller_frame.set_local(slot, modified);
                                    }
                                }
                                ThisSource::GlobalVar(var_name) => {
                                    self.globals.insert(var_name, modified);
                                }
                                ThisSource::None | ThisSource::PropertySetHook => {}
                            }
                        }

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
                    } else if e == "__FINALLY_RETURN__" {
                        // Complete a return that was delayed by finally block
                        if let Some(value) = self.pending_return.take() {
                            let frame = self.frames.last().expect("No frame");
                            let this_source = frame.this_source.clone();
                            let modified_this = if !matches!(this_source, ThisSource::None)
                                && !frame.locals.is_empty()
                            {
                                Some(frame.locals[0].clone())
                            } else {
                                None
                            };

                            self.frames.pop();

                            // Update the source variable with modified $this
                            if let Some(modified) = modified_this {
                                match this_source {
                                    ThisSource::LocalSlot(slot) => {
                                        if let Some(caller_frame) = self.frames.last_mut() {
                                            caller_frame.set_local(slot, modified);
                                        }
                                    }
                                    ThisSource::GlobalVar(var_name) => {
                                        self.globals.insert(var_name, modified);
                                    }
                                    ThisSource::None | ThisSource::PropertySetHook => {}
                                }
                            }

                            if self.frames.is_empty() {
                                return Ok(value);
                            }
                            self.stack.push(value);
                            continue;
                        }
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
            Opcode::PushNull => ops::execute_push_null(self),
            Opcode::PushTrue => ops::execute_push_true(self),
            Opcode::PushFalse => ops::execute_push_false(self),
            Opcode::PushInt(n) => ops::execute_push_int(self, n),
            Opcode::PushFloat(f) => ops::execute_push_float(self, f),
            Opcode::PushString(idx) => {
                let s = self.current_frame().get_string(idx).to_string();
                ops::execute_push_string(self, s);
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
                ops::execute_load_var(self, name);
            }
            Opcode::StoreVar(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                ops::execute_store_var(self, name);
            }
            Opcode::LoadFast(slot) => ops::execute_load_fast(self, slot),
            Opcode::StoreFast(slot) => ops::execute_store_fast(self, slot),
            Opcode::LoadGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.globals.get(&name).cloned().unwrap_or(Value::Null);
                self.stack.push(value);
            }
            Opcode::StoreGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.globals.insert(name, value.clone());
                self.stack.push(value); // Assignment returns the assigned value
            }

            // ==================== Arithmetic ====================
            Opcode::Add => ops::execute_add(self)?,
            Opcode::Sub => ops::execute_sub(self)?,
            Opcode::Mul => ops::execute_mul(self)?,
            Opcode::Div => ops::execute_div(self)?,
            Opcode::Mod => ops::execute_mod(self)?,
            Opcode::Pow => ops::execute_pow(self)?,
            Opcode::Neg => ops::execute_neg(self)?,

            // ==================== String Operations ====================
            Opcode::Concat => ops::execute_concat(self)?,

            // ==================== Comparison ====================
            Opcode::Eq => ops::execute_eq(self),
            Opcode::Ne => ops::execute_ne(self),
            Opcode::Identical => ops::execute_identical(self),
            Opcode::NotIdentical => ops::execute_not_identical(self),
            Opcode::Lt => ops::execute_lt(self)?,
            Opcode::Le => ops::execute_le(self)?,
            Opcode::Gt => ops::execute_gt(self)?,
            Opcode::Ge => ops::execute_ge(self)?,
            Opcode::Spaceship => ops::execute_spaceship(self)?,

            // ==================== Logical ====================
            Opcode::Not => ops::execute_not(self),
            Opcode::And => ops::execute_and(self),
            Opcode::Or => ops::execute_or(self),
            Opcode::Xor => ops::execute_xor(self),

            // ==================== Control Flow ====================
            Opcode::Jump(offset) => ops::execute_jump(self, offset),
            Opcode::JumpIfFalse(offset) => ops::execute_jump_if_false(self, offset),
            Opcode::JumpIfTrue(offset) => ops::execute_jump_if_true(self, offset),
            Opcode::JumpIfNull(offset) => ops::execute_jump_if_null(self, offset),
            Opcode::JumpIfNotNull(offset) => ops::execute_jump_if_not_null(self, offset),
            Opcode::Return => ops::execute_return(self)?,
            Opcode::Yield => ops::execute_yield(self)?,
            Opcode::YieldFrom => ops::execute_yield_from(self)?,
            Opcode::ReturnNull => ops::execute_return_null(self)?,

            // ==================== Loop Control ====================
            Opcode::Break => ops::execute_break(self)?,
            Opcode::Continue => ops::execute_continue(self)?,
            Opcode::LoopStart(continue_target, break_target) => {
                ops::execute_loop_start(self, continue_target, break_target);
            }
            Opcode::LoopEnd => ops::execute_loop_end(self),

            // ==================== Arrays ====================
            Opcode::NewArray(count) => ops::execute_new_array(self, count)?,
            Opcode::ArrayGet => ops::execute_array_get(self)?,
            Opcode::ArraySet => ops::execute_array_set(self)?,
            Opcode::ArrayAppend => ops::execute_array_append(self)?,
            Opcode::ArrayMerge => ops::execute_array_merge(self)?,
            Opcode::ArrayCount => ops::execute_array_count(self),
            Opcode::ArrayGetKeyAt => ops::execute_array_get_key_at(self),
            Opcode::ArrayGetValueAt => ops::execute_array_get_value_at(self),

            // ==================== Stack Manipulation ====================
            Opcode::Pop => ops::execute_pop(self),
            Opcode::Dup => ops::execute_dup(self),
            Opcode::Swap => ops::execute_swap(self)?,

            // ==================== Type Operations ====================
            Opcode::Cast(cast_type) => ops::execute_cast(self, cast_type)?,

            // ==================== Null Coalescing ====================
            Opcode::NullCoalesce => ops::execute_null_coalesce(self),

            // ==================== Output ====================
            Opcode::Echo => ops::execute_echo(self)?,
            Opcode::Print => ops::execute_print(self)?,

            // ==================== Function Calls ====================
            Opcode::Call(name_idx, arg_count) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();
                ops::execute_call(self, func_name, arg_count)?;
            }

            Opcode::CallSpread(name_idx) => {
                ops::execute_call_spread(self, name_idx)?;
            }

            Opcode::CallNamed(name_idx) => {
                ops::execute_call_named(self, name_idx)?;
            }

            Opcode::CallBuiltin(name_idx, arg_count) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();
                ops::execute_call_builtin(self, func_name, arg_count)?;
            }

            Opcode::CallBuiltinSpread(name_idx) => {
                ops::execute_call_builtin_spread(self, name_idx)?;
            }

            Opcode::CallBuiltinNamed(name_idx) => {
                ops::execute_call_builtin_named(self, name_idx)?;
            }

            Opcode::CallCallable(arg_count) => {
                ops::execute_call_callable(self, arg_count)?;
            }

            // ==================== OOP Opcodes ====================
            Opcode::NewObject(class_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                ops::execute_new_object(self, class_name)?
            }

            Opcode::NewFiber => {
                // Pop callback from stack
                let callback = self.stack.pop().ok_or("Stack underflow")?;

                // Look up Fiber class definition
                let fiber_class = self
                    .classes
                    .get("Fiber")
                    .ok_or("Fiber class not found")?
                    .clone();

                // Create new Fiber instance
                let mut instance = crate::runtime::ObjectInstance::with_hierarchy(
                    "Fiber".to_string(),
                    fiber_class.parent.clone(),
                    fiber_class.interfaces.clone(),
                );

                // Initialize properties with defaults
                for prop in &fiber_class.properties {
                    let default_val = prop.default.clone().unwrap_or(Value::Null);
                    instance
                        .properties
                        .insert(prop.name.clone(), default_val.clone());
                    if prop.readonly {
                        instance.readonly_properties.insert(prop.name.clone());
                        if prop.default.is_some() {
                            instance.initialized_readonly.insert(prop.name.clone());
                        }
                    }
                }

                // Store callback manually (before constructor)
                instance
                    .properties
                    .insert("__callback".to_string(), callback);

                // Push Fiber object
                self.stack.push(Value::Object(instance));

                // Constructor will be called separately via CallConstructor opcode
            }

            Opcode::LoadProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_load_property(self, prop_name)?
            }

            Opcode::StoreProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_store_property(self, prop_name)?
            }

            Opcode::StoreCloneProperty(prop_idx) => {
                ops::execute_store_clone_property(self, prop_idx)?
            }

            Opcode::UnsetProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_unset_property(self, prop_name)?
            }

            Opcode::IssetProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_isset_property(self, prop_name);
            }

            Opcode::UnsetPropertyOnLocal(slot, prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_unset_property_on_local(self, slot, prop_name)?
            }

            Opcode::UnsetPropertyOnGlobal(var_idx, prop_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_unset_property_on_global(self, var_name, prop_name)?
            }

            Opcode::IssetPropertyOnLocal(slot, prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_isset_property_on_local(self, slot, prop_name);
            }

            Opcode::IssetPropertyOnGlobal(var_idx, prop_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_isset_property_on_global(self, var_name, prop_name);
            }

            Opcode::UnsetVar(var_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                self.globals.remove(&var_name);
            }

            Opcode::UnsetArrayElement => {
                let key = self.stack.pop().ok_or("Stack underflow")?;
                let array = self.stack.pop().ok_or("Stack underflow")?;

                match array {
                    Value::Array(mut arr) => {
                        let array_key = match key {
                            Value::Integer(n) => ArrayKey::Integer(n),
                            Value::String(s) => ArrayKey::String(s),
                            _ => return Err(format!("Invalid array key type: {:?}", key)),
                        };
                        // Remove element with matching key
                        arr.retain(|(k, _)| k != &array_key);
                        // Note: We don't push anything back - unset doesn't return a value
                    }
                    _ => return Err("Cannot unset element of non-array".to_string()),
                }
            }

            Opcode::StoreThisProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_store_this_property(self, prop_name)?
            }

            Opcode::CallMethod(method_idx, arg_count) => {
                let method_name = self.current_frame().get_string(method_idx).to_string();
                ops::execute_call_method(self, method_name, arg_count)?
            }

            Opcode::CallMethodOnLocal(var_slot, method_idx, arg_count) => {
                let method_name = self.current_frame().get_string(method_idx).to_string();
                ops::execute_call_method_on_local(self, var_slot, method_name, arg_count)?
            }

            Opcode::CallMethodOnGlobal(var_idx, method_idx, arg_count) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let method_name = self.current_frame().get_string(method_idx).to_string();
                ops::execute_call_method_on_global(self, var_name, method_name, arg_count)?
            }

            Opcode::CallStaticMethod(class_idx, method_idx, arg_count) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let method_name = self.current_frame().get_string(method_idx).to_string();
                ops::execute_call_static_method(self, class_name, method_name, arg_count)?
            }

            Opcode::CallStaticMethodNamed(class_idx, method_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let method_name = self.current_frame().get_string(method_idx).to_string();
                ops::execute_call_static_method_named(self, class_name, method_name)?
            }

            Opcode::LoadStaticProp(class_idx, prop_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_load_static_prop(self, class_name, prop_name)?
            }

            Opcode::StoreStaticProp(class_idx, prop_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                ops::execute_store_static_prop(self, class_name, prop_name)?
            }

            Opcode::LoadThis => ops::execute_load_this(self)?,

            Opcode::InstanceOf(class_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                ops::execute_instance_of(self, class_name);
            }

            Opcode::Clone => ops::execute_clone(self)?,

            Opcode::LoadEnumCase(enum_idx, case_idx) => {
                let enum_name =
                    Self::normalize_class_name(self.current_frame().get_string(enum_idx));
                let case_name = self.current_frame().get_string(case_idx).to_string();
                ops::execute_load_enum_case(self, enum_name, case_name)?
            }

            Opcode::CallConstructor(arg_count) => ops::execute_call_constructor(self, arg_count)?,

            Opcode::CallConstructorNamed => ops::execute_call_constructor_named(self)?,

            // ==================== Exception Handling ====================
            Opcode::Throw => ops::execute_throw(self)?,

            Opcode::TryStart(catch_offset, finally_offset) => {
                ops::execute_try_start(self, catch_offset, finally_offset);
            }

            Opcode::TryEnd => {
                ops::execute_try_end(self);
            }

            Opcode::FinallyStart => {
                ops::execute_finally_start(self);
            }

            Opcode::FinallyEnd => ops::execute_finally_end(self)?,

            // ==================== Closures ====================
            Opcode::CreateClosure(func_idx, capture_count) => {
                let func_name = self.current_frame().get_string(func_idx).to_string();

                // Pop captured variables from stack (in reverse order)
                let mut captured_vars: Vec<(String, Value)> = Vec::new();
                for _ in 0..capture_count {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let var_name = self.stack.pop().ok_or("Stack underflow")?;
                    if let Value::String(name) = var_name {
                        captured_vars.push((name, value));
                    } else {
                        return Err("CaptureVar expects variable name as string".to_string());
                    }
                }

                // Create a proper Closure value with FunctionRef
                use crate::runtime::Closure;
                let closure = Closure {
                    params: Vec::new(), // Params are handled by the compiled function
                    body: ClosureBody::FunctionRef(func_name),
                    captured_vars,
                };
                self.stack.push(Value::Closure(Box::new(closure)));
            }

            Opcode::CaptureVar(var_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();

                // Load the variable value
                let value = {
                    let frame = self.current_frame();
                    // Search for the variable in local_names to find its slot
                    let slot = frame
                        .function
                        .local_names
                        .iter()
                        .position(|name| name == &var_name)
                        .map(|i| i as u16);

                    if let Some(slot) = slot {
                        // It's a local variable
                        frame.locals[slot as usize].clone()
                    } else {
                        // Try global scope
                        self.globals.get(&var_name).cloned().unwrap_or(Value::Null)
                    }
                };

                // Push variable name and value onto stack
                self.stack.push(Value::String(var_name));
                self.stack.push(value);
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

    /// Look up function case-insensitively (PHP functions are case-insensitive)
    fn get_function(&self, name: &str) -> Option<Arc<CompiledFunction>> {
        // Try exact match first
        if let Some(func) = self.functions.get(name) {
            return Some(func.clone());
        }
        // Try case-insensitive match
        let name_lower = name.to_lowercase();
        self.functions
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.clone())
    }

    /// Get the current class name from the function name (format: "ClassName::methodName")
    fn get_current_class(&self) -> Option<String> {
        let frame = self.frames.last()?;
        let func_name = &frame.function.name;
        // Function names are formatted as "ClassName::methodName" for methods
        func_name.find("::").map(|pos| func_name[..pos].to_string())
    }

    /// Check if a value matches a type hint (strict mode - no coercion)
    /// Used for return type validation which is always strict in PHP
    fn value_matches_type_strict(&self, value: &Value, type_hint: &crate::ast::TypeHint) -> bool {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type_strict(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type_strict(value, inner)
            }
            TypeHint::Union(types) => types
                .iter()
                .any(|t| self.value_matches_type_strict(value, t)),
            TypeHint::Intersection(types) => types
                .iter()
                .all(|t| self.value_matches_type_strict(value, t)),
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                intersections.iter().any(|group| {
                    // All types in the group must match
                    group
                        .iter()
                        .all(|t| self.value_matches_type_strict(value, t))
                })
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,       // void is for return types only
            TypeHint::Never => false,      // never is for return types only
            TypeHint::Static => false,     // Requires class context
            TypeHint::SelfType => false,   // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }

    /// Helper for strict type matching (no coercion) - used for return types
    fn value_matches_simple_type_strict(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            ("string", Value::String(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            ("bool", Value::Bool(_)) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("null", Value::Null) => true,
            ("mixed", _) => true, // mixed accepts anything
            _ => {
                // Check if it's a class/interface/enum type
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, type_name)
                } else if let Value::EnumCase { enum_name, .. } = value {
                    enum_name == type_name
                } else {
                    false
                }
            }
        }
    }

    /// Check if a value matches a type hint (includes coercive mode for scalars)
    fn value_matches_type(&self, value: &Value, type_hint: &crate::ast::TypeHint) -> bool {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => self.value_matches_simple_type(value, name),
            TypeHint::Nullable(inner) => {
                matches!(value, Value::Null) || self.value_matches_type(value, inner)
            }
            TypeHint::Union(types) => types.iter().any(|t| self.value_matches_type(value, t)),
            TypeHint::Intersection(types) => {
                types.iter().all(|t| self.value_matches_type(value, t))
            }
            TypeHint::DNF(intersections) => {
                // DNF: (A&B)|(C&D)|E
                // Value must match at least one intersection group
                intersections.iter().any(|group| {
                    // All types in the group must match
                    group.iter().all(|t| self.value_matches_type(value, t))
                })
            }
            TypeHint::Class(class_name) => {
                if let Value::Object(obj) = value {
                    self.is_instance_of(&obj.class_name, class_name)
                } else {
                    false
                }
            }
            TypeHint::Void => false,       // void is for return types only
            TypeHint::Never => false,      // never is for return types only
            TypeHint::Static => false,     // Requires class context
            TypeHint::SelfType => false,   // Requires class context
            TypeHint::ParentType => false, // Requires class context
        }
    }

    /// Helper to check simple type matches (includes coercive mode for scalars)
    fn value_matches_simple_type(&self, value: &Value, type_name: &str) -> bool {
        match (type_name, value) {
            ("int", Value::Integer(_)) => true,
            // Coercive mode: float can be coerced to int
            ("int", Value::Float(_)) => true,
            // Coercive mode: only numeric strings can be coerced to int
            ("int", Value::String(s)) => self.is_numeric_string(s),
            // Coercive mode: bool can be coerced to int
            ("int", Value::Bool(_)) => true,
            ("string", Value::String(_)) => true,
            // Coercive mode: scalars can be coerced to string
            ("string", Value::Integer(_)) => true,
            ("string", Value::Float(_)) => true,
            ("string", Value::Bool(_)) => true,
            ("float", Value::Float(_)) => true,
            ("float", Value::Integer(_)) => true, // int is compatible with float
            // Coercive mode: only numeric strings can be coerced to float
            ("float", Value::String(s)) => self.is_numeric_string(s),
            ("bool", Value::Bool(_)) => true,
            // Coercive mode: any scalar can be coerced to bool
            ("bool", Value::Integer(_)) => true,
            ("bool", Value::Float(_)) => true,
            ("bool", Value::String(_)) => true,
            ("bool", Value::Null) => true,
            ("array", Value::Array(_)) => true,
            ("object", Value::Object(_)) => true,
            ("object", Value::Fiber(_)) => true,
            ("object", Value::Closure(_)) => true,
            ("object", Value::EnumCase { .. }) => true,
            ("callable", Value::Closure(_)) => true,
            ("callable", Value::String(_)) => true, // function name
            ("iterable", Value::Array(_)) => true,
            ("mixed", _) => true,
            ("null", Value::Null) => true,
            ("false", Value::Bool(false)) => true,
            ("true", Value::Bool(true)) => true,
            _ => false,
        }
    }

    /// Check if a string is numeric (can be coerced to int/float)
    fn is_numeric_string(&self, s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        // Try to parse as a number - must be a valid numeric string
        // PHP considers strings like "123", "123.45", "1e5", "-42" as numeric
        if trimmed.parse::<i64>().is_ok() {
            return true;
        }
        if trimmed.parse::<f64>().is_ok() {
            return true;
        }
        false
    }

    /// Check if a type hint requires strict type checking (class/interface types)
    /// Simple scalar types (int, string, etc.) use PHP's coercive mode by default
    fn requires_strict_type_check(&self, type_hint: &crate::ast::TypeHint) -> bool {
        use crate::ast::TypeHint;
        match type_hint {
            // Class types always require strict checking
            TypeHint::Class(_) => true,
            // Simple types - only check for class names (not scalar types)
            TypeHint::Simple(name) => {
                // These are scalar types that can be coerced
                !matches!(
                    name.as_str(),
                    "int"
                        | "string"
                        | "float"
                        | "bool"
                        | "array"
                        | "mixed"
                        | "null"
                        | "callable"
                        | "iterable"
                        | "object"
                )
            }
            // Check the inner type for nullable
            TypeHint::Nullable(inner) => self.requires_strict_type_check(inner),
            // For union/intersection/DNF, check if any part requires strict checking
            TypeHint::Union(types) => types.iter().any(|t| self.requires_strict_type_check(t)),
            TypeHint::Intersection(types) => {
                types.iter().any(|t| self.requires_strict_type_check(t))
            }
            TypeHint::DNF(groups) => groups
                .iter()
                .any(|g| g.iter().any(|t| self.requires_strict_type_check(t))),
            // These special types are for return types
            TypeHint::Void
            | TypeHint::Never
            | TypeHint::Static
            | TypeHint::SelfType
            | TypeHint::ParentType => false,
        }
    }

    /// Get the type name for error messages
    fn get_value_type_name(&self, value: &Value) -> &'static str {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Closure(_) => "Closure",
            Value::Fiber(_) => "Fiber",
            Value::Generator(_) => "Generator",
            Value::EnumCase { .. } => "enum",
            Value::Exception(_) => "Exception",
        }
    }

    /// Coerce a value to match a type hint (for coercive mode)
    fn coerce_value_to_type(&self, value: Value, type_hint: &crate::ast::TypeHint) -> Value {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => {
                match name.as_str() {
                    "int" => {
                        // Convert to int using PHP rules
                        match &value {
                            Value::Integer(_) => value,
                            Value::Float(f) => Value::Integer(*f as i64),
                            Value::Bool(b) => Value::Integer(if *b { 1 } else { 0 }),
                            Value::String(s) => {
                                // Parse leading digits from string
                                let trimmed = s.trim_start();
                                if trimmed.is_empty() {
                                    return Value::Integer(0);
                                }
                                // Try to parse as integer or float, taking only leading valid part
                                let mut end_pos = 0;
                                let chars: Vec<char> = trimmed.chars().collect();
                                // Handle optional sign
                                if !chars.is_empty() && (chars[0] == '+' || chars[0] == '-') {
                                    end_pos = 1;
                                }
                                // Collect digits
                                while end_pos < chars.len() && chars[end_pos].is_ascii_digit() {
                                    end_pos += 1;
                                }
                                if end_pos == 0
                                    || (end_pos == 1 && (chars[0] == '+' || chars[0] == '-'))
                                {
                                    // No digits found - coerce to 0
                                    return Value::Integer(0);
                                }
                                // Parse the numeric part
                                let numeric_part: String = chars[..end_pos].iter().collect();
                                Value::Integer(numeric_part.parse().unwrap_or(0))
                            }
                            _ => Value::Integer(value.to_int()),
                        }
                    }
                    "float" => Value::Float(value.to_float()),
                    "string" => Value::String(value.to_string_val()),
                    "bool" => Value::Bool(value.to_bool()),
                    _ => value, // For other types, don't coerce
                }
            }
            TypeHint::Nullable(inner) => {
                if matches!(value, Value::Null) {
                    value
                } else {
                    self.coerce_value_to_type(value, inner)
                }
            }
            _ => value, // For complex types, don't coerce
        }
    }

    /// Format a type hint for error messages
    fn format_type_hint(&self, type_hint: &crate::ast::TypeHint) -> String {
        use crate::ast::TypeHint;
        match type_hint {
            TypeHint::Simple(name) => name.clone(),
            TypeHint::Nullable(inner) => format!("?{}", self.format_type_hint(inner)),
            TypeHint::Union(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Intersection(types) => types
                .iter()
                .map(|t| self.format_type_hint(t))
                .collect::<Vec<_>>()
                .join("&"),
            TypeHint::DNF(groups) => groups
                .iter()
                .map(|group| {
                    let inner = group
                        .iter()
                        .map(|t| self.format_type_hint(t))
                        .collect::<Vec<_>>()
                        .join("&");
                    if group.len() > 1 {
                        format!("({})", inner)
                    } else {
                        inner
                    }
                })
                .collect::<Vec<_>>()
                .join("|"),
            TypeHint::Class(name) => name.clone(),
            TypeHint::Void => "void".to_string(),
            TypeHint::Never => "never".to_string(),
            TypeHint::Static => "static".to_string(),
            TypeHint::SelfType => "self".to_string(),
            TypeHint::ParentType => "parent".to_string(),
        }
    }

    /// Call a reflection function or regular builtin function
    fn call_reflection_or_builtin(
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
            _ => {
                // Call the regular built-in function
                builtins::call_builtin(func_name, args, &mut self.output)
            }
        }
    }
}

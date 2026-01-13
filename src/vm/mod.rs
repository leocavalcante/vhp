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
pub mod ops;
pub mod reflection;
pub mod values;

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
                self.globals.insert(name, value.clone());
                self.stack.push(value); // Assignment returns the assigned value
            }
            Opcode::LoadFast(slot) => {
                let value = self.current_frame().get_local(slot).clone();
                self.stack.push(value);
            }
            Opcode::StoreFast(slot) => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                self.current_frame_mut().set_local(slot, value.clone());
                self.stack.push(value); // Assignment returns the assigned value
            }
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
                    return Err("Division by zero".to_string());
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

                // Convert left to string, handling __toString for objects
                let left_str = self.value_to_string(left)?;
                let right_str = self.value_to_string(right)?;

                let result = Value::String(format!("{}{}", left_str, right_str));
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
                self.stack
                    .push(Value::Bool(left.to_bool() && right.to_bool()));
            }
            Opcode::Or => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack
                    .push(Value::Bool(left.to_bool() || right.to_bool()));
            }
            Opcode::Xor => {
                let right = self.stack.pop().ok_or("Stack underflow")?;
                let left = self.stack.pop().ok_or("Stack underflow")?;
                self.stack
                    .push(Value::Bool(left.to_bool() ^ right.to_bool()));
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
                // Validate return type if present
                if let Some(ref return_type) = self.current_frame().function.return_type.clone() {
                    // Check for void return type - this is always an error for return with value
                    if matches!(return_type, crate::ast::TypeHint::Void) {
                        return Err(format!(
                            "{}(): Return value must be of type void",
                            self.current_frame().function.name
                        ));
                    }
                    // Validate return value against type hint
                    // Return types are ALWAYS strictly checked in PHP (no coercion)
                    let return_value = self.stack.last().cloned().unwrap_or(Value::Null);
                    if !self.value_matches_type_strict(&return_value, return_type) {
                        let type_name = self.format_type_hint(return_type);
                        let given_type = self.get_value_type_name(&return_value);
                        return Err(format!(
                            "Return value must be of type {}, {} returned",
                            type_name, given_type
                        ));
                    }
                }
                return Err("__RETURN__".to_string());
            }
            Opcode::Yield => {
                // For now, generators are not fully implemented in execution
                // Create a placeholder generator object
                let value = self.stack.pop().unwrap_or(Value::Null);
                let gen = crate::runtime::GeneratorInstance {
                    id: 0,
                    position: 0,
                    values: vec![value.clone()],
                    statements: vec![],
                    current_statement: 0,
                    variables: std::collections::HashMap::new(),
                    finished: false,
                };
                self.stack.push(Value::Generator(Box::new(gen)));
                return Err("__GENERATOR__".to_string());
            }
            Opcode::YieldFrom => {
                // For now, generators are not fully implemented in execution
                // Create a placeholder generator object
                let value = self.stack.pop().unwrap_or(Value::Null);
                let gen = crate::runtime::GeneratorInstance {
                    id: 0,
                    position: 0,
                    values: vec![value.clone()],
                    statements: vec![],
                    current_statement: 0,
                    variables: std::collections::HashMap::new(),
                    finished: false,
                };
                self.stack.push(Value::Generator(Box::new(gen)));
                return Err("__GENERATOR__".to_string());
            }
            Opcode::ReturnNull => {
                // Validate return type if present
                if let Some(ref return_type) = self.current_frame().function.return_type.clone() {
                    // void is OK for return null (implicit return)
                    if !matches!(return_type, crate::ast::TypeHint::Void) {
                        // Validate null against return type
                        // Return types are ALWAYS strictly checked in PHP (no coercion)
                        if !self.value_matches_type_strict(&Value::Null, return_type) {
                            let type_name = self.format_type_hint(return_type);
                            return Err(format!(
                                "Return value must be of type {}, null returned",
                                type_name
                            ));
                        }
                    }
                }
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
            Opcode::ArrayMerge => {
                let array2 = self.stack.pop().ok_or("Stack underflow")?;
                let array1 = self.stack.pop().ok_or("Stack underflow")?;
                match (array1, array2) {
                    (Value::Array(mut arr1), Value::Array(arr2)) => {
                        // Merge array2 into array1
                        // Re-index to maintain sequential integer keys
                        let next_idx = arr1
                            .iter()
                            .filter_map(|(k, _)| match k {
                                ArrayKey::Integer(n) => Some(*n),
                                _ => None,
                            })
                            .max()
                            .unwrap_or(-1)
                            + 1;

                        for (i, (_, value)) in arr2.into_iter().enumerate() {
                            arr1.push((ArrayKey::Integer(next_idx + i as i64), value));
                        }
                        self.stack.push(Value::Array(arr1));
                    }
                    _ => return Err("Cannot merge non-arrays".to_string()),
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
                // Check if it's an object with __toString
                if let Value::Object(ref instance) = value {
                    if let Some(to_string_method) =
                        self.find_method_in_chain(&instance.class_name, "__toString")
                    {
                        // Call __toString and get the result
                        let result = self.call_method_sync(instance.clone(), to_string_method)?;
                        match result {
                            Value::String(s) => {
                                write!(self.output, "{}", s).map_err(|e| e.to_string())?
                            }
                            _ => {
                                return Err(format!(
                                    "Return value must be of type string, {} returned",
                                    result.get_type()
                                ))
                            }
                        }
                    } else {
                        return Err(format!(
                            "Object of class {} could not be converted to string",
                            instance.class_name
                        ));
                    }
                } else {
                    write!(self.output, "{}", value.to_output_string())
                        .map_err(|e| e.to_string())?;
                }
            }
            Opcode::Print => {
                let value = self.stack.pop().ok_or("Stack underflow")?;
                // Check if it's an object with __toString
                if let Value::Object(ref instance) = value {
                    if let Some(to_string_method) =
                        self.find_method_in_chain(&instance.class_name, "__toString")
                    {
                        // Call __toString and get the result
                        let result = self.call_method_sync(instance.clone(), to_string_method)?;
                        match result {
                            Value::String(s) => {
                                write!(self.output, "{}", s).map_err(|e| e.to_string())?
                            }
                            _ => {
                                return Err(format!(
                                    "Return value must be of type string, {} returned",
                                    result.get_type()
                                ))
                            }
                        }
                    } else {
                        return Err(format!(
                            "Object of class {} could not be converted to string",
                            instance.class_name
                        ));
                    }
                } else {
                    write!(self.output, "{}", value.to_output_string())
                        .map_err(|e| e.to_string())?;
                }
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

                // 1. Check user-defined functions first (case-insensitive)
                if let Some(func) = self.get_function(&func_name) {
                    // Check minimum argument count
                    if arg_count < func.required_param_count {
                        return Err(format!(
                            "Too few arguments to function {}(), {} passed in, at least {} expected",
                            func.name, arg_count, func.required_param_count
                        ));
                    }

                    // Validate parameter types
                    // If strict_types is enabled, validate all types strictly
                    // Otherwise, only validate class types strictly (scalars use coercive mode)
                    for (i, arg) in args.iter().enumerate() {
                        if i < func.param_types.len() {
                            if let Some(ref type_hint) = func.param_types[i] {
                                let use_strict =
                                    func.strict_types || self.requires_strict_type_check(type_hint);
                                if use_strict {
                                    if !self.value_matches_type_strict(arg, type_hint) {
                                        let type_name = self.format_type_hint(type_hint);
                                        let given_type = self.get_value_type_name(arg);
                                        return Err(format!(
                                            "Argument {} passed to {}() must be of type {}, {} given",
                                            i + 1, func.name, type_name, given_type
                                        ));
                                    }
                                } else {
                                    // Coercive mode - check if value can be coerced
                                    if !self.value_matches_type(arg, type_hint) {
                                        let type_name = self.format_type_hint(type_hint);
                                        let given_type = self.get_value_type_name(arg);
                                        return Err(format!(
                                            "Argument {} passed to {}() must be of type {}, {} given",
                                            i + 1, func.name, type_name, given_type
                                        ));
                                    }
                                }
                            }
                        }
                    }

                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(func.clone(), stack_base);

                    // Handle variadic functions
                    if func.is_variadic && func.param_count > 0 {
                        let variadic_slot = (func.param_count - 1) as usize;
                        // Set regular params first (with type coercion unless strict_types)
                        for i in 0..variadic_slot {
                            if i < args.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            let coerced = self
                                                .coerce_value_to_type(args[i].clone(), type_hint);
                                            // Validate that coercion succeeded (type matches)
                                            if !self.value_matches_type(&coerced, type_hint) {
                                                let type_name = self.format_type_hint(type_hint);
                                                let given_type = self.get_value_type_name(&args[i]);
                                                return Err(format!(
                                                    "must be of type {}, {} given",
                                                    type_name, given_type
                                                ));
                                            }
                                            coerced
                                        } else {
                                            args[i].clone()
                                        }
                                    } else {
                                        args[i].clone()
                                    }
                                } else {
                                    args[i].clone()
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                        // Collect remaining args into array for variadic param
                        let variadic_args: Vec<(ArrayKey, Value)> = args
                            .into_iter()
                            .skip(variadic_slot)
                            .enumerate()
                            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                            .collect();
                        frame.locals[variadic_slot] = Value::Array(variadic_args);
                    } else {
                        // Set up parameter locals normally (with type coercion unless strict_types)
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < frame.locals.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            let coerced =
                                                self.coerce_value_to_type(arg.clone(), type_hint);
                                            // Validate that coercion succeeded (type matches)
                                            if !self.value_matches_type(&coerced, type_hint) {
                                                let type_name = self.format_type_hint(type_hint);
                                                let given_type = self.get_value_type_name(&arg);
                                                return Err(format!(
                                                    "must be of type {}, {} given",
                                                    type_name, given_type
                                                ));
                                            }
                                            coerced
                                        } else {
                                            arg
                                        }
                                    } else {
                                        arg
                                    }
                                } else {
                                    arg
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                    }

                    self.frames.push(frame);
                }
                // 2. Fall back to built-in functions
                else if builtins::is_builtin(&func_name) {
                    // Check if it's a reflection function that needs VM context
                    let result = self.call_reflection_or_builtin(&func_name, &args)?;
                    self.stack.push(result);
                }
                // 3. Unknown function
                else {
                    return Err(format!("undefined function: {}", func_name));
                }
            }

            Opcode::CallSpread(name_idx) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop args array from stack
                let args_array = self.stack.pop().ok_or("Stack underflow")?;
                let args = match args_array {
                    Value::Array(arr) => {
                        // Extract values from array in order
                        arr.into_iter().map(|(_, v)| v).collect::<Vec<_>>()
                    }
                    _ => return Err("CallSpread expects an array of arguments".to_string()),
                };

                let arg_count = args.len();

                // 1. Check user-defined functions first (case-insensitive)
                if let Some(func) = self.get_function(&func_name) {
                    // Check minimum argument count
                    if (arg_count as u8) < func.required_param_count {
                        return Err(format!(
                            "Too few arguments to function {}(), {} passed in, at least {} expected",
                            func.name, arg_count, func.required_param_count
                        ));
                    }

                    // Validate parameter types
                    for (i, arg) in args.iter().enumerate() {
                        if i < func.param_types.len() {
                            if let Some(ref type_hint) = func.param_types[i] {
                                let use_strict =
                                    func.strict_types || self.requires_strict_type_check(type_hint);
                                if use_strict {
                                    if !self.value_matches_type_strict(arg, type_hint) {
                                        let type_name = self.format_type_hint(type_hint);
                                        let given_type = self.get_value_type_name(arg);
                                        return Err(format!(
                                            "Argument {} passed to {}() must be of type {}, {} given",
                                            i + 1, func.name, type_name, given_type
                                        ));
                                    }
                                } else if !self.value_matches_type(arg, type_hint) {
                                    let type_name = self.format_type_hint(type_hint);
                                    let given_type = self.get_value_type_name(arg);
                                    return Err(format!(
                                        "Argument {} passed to {}() must be of type {}, {} given",
                                        i + 1,
                                        func.name,
                                        type_name,
                                        given_type
                                    ));
                                }
                            }
                        }
                    }

                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(func.clone(), stack_base);

                    // Handle variadic functions
                    if func.is_variadic && func.param_count > 0 {
                        let variadic_slot = (func.param_count - 1) as usize;
                        for i in 0..variadic_slot {
                            if i < args.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            self.coerce_value_to_type(args[i].clone(), type_hint)
                                        } else {
                                            args[i].clone()
                                        }
                                    } else {
                                        args[i].clone()
                                    }
                                } else {
                                    args[i].clone()
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                        let variadic_args: Vec<(ArrayKey, Value)> = args
                            .into_iter()
                            .skip(variadic_slot)
                            .enumerate()
                            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                            .collect();
                        frame.locals[variadic_slot] = Value::Array(variadic_args);
                    } else {
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < frame.locals.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            self.coerce_value_to_type(arg.clone(), type_hint)
                                        } else {
                                            arg
                                        }
                                    } else {
                                        arg
                                    }
                                } else {
                                    arg
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                    }

                    self.frames.push(frame);
                }
                // 2. Fall back to built-in functions
                else if builtins::is_builtin(&func_name) {
                    let result = self.call_reflection_or_builtin(&func_name, &args)?;
                    self.stack.push(result);
                }
                // 3. Unknown function
                else {
                    return Err(format!("undefined function: {}", func_name));
                }
            }

            Opcode::CallNamed(name_idx) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop args array from stack (mixed positional/named)
                let args_array = self.stack.pop().ok_or("Stack underflow")?;
                let (positional_args, named_args) = match args_array {
                    Value::Array(arr) => {
                        // Separate positional (integer keys) from named (string keys)
                        let mut positional = Vec::new();
                        let mut named = std::collections::HashMap::new();

                        for (k, v) in arr {
                            match k {
                                ArrayKey::Integer(i) => {
                                    positional.push((i as usize, v));
                                }
                                ArrayKey::String(name) => {
                                    named.insert(name, v);
                                }
                            }
                        }

                        // Sort positional by index
                        positional.sort_by_key(|(i, _)| *i);
                        let positional: Vec<Value> =
                            positional.into_iter().map(|(_, v)| v).collect();

                        (positional, named)
                    }
                    _ => return Err("CallNamed expects an array of arguments".to_string()),
                };

                // 1. Check user-defined functions first (case-insensitive)
                if let Some(func) = self.get_function(&func_name) {
                    // Map positional and named arguments to function parameters
                    let mut args = Vec::with_capacity(func.param_count as usize);

                    for i in 0..func.param_count as usize {
                        if i < func.parameters.len() {
                            let param_name = &func.parameters[i].name;

                            // Try positional first, then named
                            if i < positional_args.len() {
                                args.push(positional_args[i].clone());
                            } else if let Some(value) = named_args.get(param_name) {
                                args.push(value.clone());
                            } else if func.parameters[i].default.is_some() {
                                // Use default value - push null for now, will be handled in frame setup
                                args.push(Value::Null); // Marker for "use default"
                            } else if i < func.required_param_count as usize {
                                return Err(format!(
                                    "Missing required argument '{}' for function {}()",
                                    param_name, func.name
                                ));
                            } else {
                                args.push(Value::Null);
                            }
                        }
                    }

                    // Check for unknown parameters
                    for name in named_args.keys() {
                        if !func.parameters.iter().any(|p| &p.name == name) {
                            return Err(format!(
                                "Unknown named parameter '{}' for function {}()",
                                name, func.name
                            ));
                        }
                    }

                    // Validate and create call frame (reuse existing logic)
                    let _arg_count = args.len();

                    // Validate parameter types
                    for (i, arg) in args.iter().enumerate() {
                        if i < func.param_types.len() {
                            if let Some(ref type_hint) = func.param_types[i] {
                                let use_strict =
                                    func.strict_types || self.requires_strict_type_check(type_hint);
                                if use_strict {
                                    if !self.value_matches_type_strict(arg, type_hint) {
                                        let type_name = self.format_type_hint(type_hint);
                                        let given_type = self.get_value_type_name(arg);
                                        return Err(format!(
                                            "Argument {} passed to {}() must be of type {}, {} given",
                                            i + 1, func.name, type_name, given_type
                                        ));
                                    }
                                } else if !self.value_matches_type(arg, type_hint) {
                                    let type_name = self.format_type_hint(type_hint);
                                    let given_type = self.get_value_type_name(arg);
                                    return Err(format!(
                                        "Argument {} passed to {}() must be of type {}, {} given",
                                        i + 1,
                                        func.name,
                                        type_name,
                                        given_type
                                    ));
                                }
                            }
                        }
                    }

                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(func.clone(), stack_base);

                    // Handle variadic functions
                    if func.is_variadic && func.param_count > 0 {
                        let variadic_slot = (func.param_count - 1) as usize;
                        for i in 0..variadic_slot {
                            if i < args.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            self.coerce_value_to_type(args[i].clone(), type_hint)
                                        } else {
                                            args[i].clone()
                                        }
                                    } else {
                                        args[i].clone()
                                    }
                                } else {
                                    args[i].clone()
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                        let variadic_args: Vec<(ArrayKey, Value)> = args
                            .into_iter()
                            .skip(variadic_slot)
                            .enumerate()
                            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                            .collect();
                        frame.locals[variadic_slot] = Value::Array(variadic_args);
                    } else {
                        for (i, arg) in args.into_iter().enumerate() {
                            if i < frame.locals.len() {
                                let coerced_arg = if i < func.param_types.len() {
                                    if let Some(ref type_hint) = func.param_types[i] {
                                        let use_strict = func.strict_types
                                            || self.requires_strict_type_check(type_hint);
                                        if !use_strict {
                                            self.coerce_value_to_type(arg.clone(), type_hint)
                                        } else {
                                            arg
                                        }
                                    } else {
                                        arg
                                    }
                                } else {
                                    arg
                                };
                                frame.locals[i] = coerced_arg;
                            }
                        }
                    }

                    self.frames.push(frame);
                }
                // 2. Fall back to built-in functions
                else if builtins::is_builtin(&func_name) {
                    // For built-ins, use positional args first, then add named
                    let mut args = positional_args;
                    for (_, v) in named_args {
                        args.push(v);
                    }
                    let result = self.call_reflection_or_builtin(&func_name, &args)?;
                    self.stack.push(result);
                }
                // 3. Unknown function
                else {
                    return Err(format!("undefined function: {}", func_name));
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

                // Call reflection or builtin function
                let result = self.call_reflection_or_builtin(&func_name, &args)?;
                self.stack.push(result);
            }

            Opcode::CallBuiltinSpread(name_idx) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop args array from stack
                let args_array = self.stack.pop().ok_or("Stack underflow")?;
                let args = match args_array {
                    Value::Array(arr) => arr.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
                    _ => return Err("CallBuiltinSpread expects an array of arguments".to_string()),
                };

                // Call reflection or builtin function
                let result = self.call_reflection_or_builtin(&func_name, &args)?;
                self.stack.push(result);
            }

            Opcode::CallBuiltinNamed(name_idx) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();

                // Pop args associative array from stack
                let args_array = self.stack.pop().ok_or("Stack underflow")?;
                let named_args = match args_array {
                    Value::Array(arr) => arr
                        .into_iter()
                        .filter_map(|(k, v)| {
                            if let ArrayKey::String(name) = k {
                                Some((name, v))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<(String, Value)>>(),
                    _ => {
                        return Err("CallBuiltinNamed expects an associative array of arguments"
                            .to_string())
                    }
                };

                // For built-ins, convert named args to positional
                let args: Vec<Value> = named_args.into_iter().map(|(_, v)| v).collect();

                // Call reflection or builtin function
                let result = self.call_reflection_or_builtin(&func_name, &args)?;
                self.stack.push(result);
            }

            Opcode::CallCallable(arg_count) => {
                // Pop callable from stack (on top after args)
                let callable = self.stack.pop().ok_or("Stack underflow")?;

                // Pop arguments from stack
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                match callable {
                    Value::String(func_name) => {
                        // First-class callable: function name as string
                        // Check user-defined functions first (case-insensitive)
                        if let Some(func) = self.get_function(&func_name) {
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(func.clone(), stack_base);

                            // Handle variadic functions
                            if func.is_variadic && func.param_count > 0 {
                                let variadic_slot = (func.param_count - 1) as usize;
                                for i in 0..variadic_slot {
                                    if i < args.len() {
                                        frame.locals[i] = args[i].clone();
                                    }
                                }
                                let variadic_args: Vec<(ArrayKey, Value)> = args
                                    .into_iter()
                                    .skip(variadic_slot)
                                    .enumerate()
                                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                                    .collect();
                                frame.locals[variadic_slot] = Value::Array(variadic_args);
                            } else {
                                for (i, arg) in args.into_iter().enumerate() {
                                    if i < frame.locals.len() {
                                        frame.locals[i] = arg;
                                    }
                                }
                            }
                            self.frames.push(frame);
                        }
                        // Fall back to built-in functions
                        else if builtins::is_builtin(&func_name) {
                            let result =
                                builtins::call_builtin(&func_name, &args, &mut self.output)?;
                            self.stack.push(result);
                        } else {
                            return Err(format!("undefined function: {}", func_name));
                        }
                    }
                    Value::Closure(closure) => {
                        // Closure call - depends on the closure body type
                        match &closure.body {
                            ClosureBody::FunctionRef(func_name) => {
                                // First-class callable or arrow function
                                if let Some(func) = self.get_function(func_name) {
                                    let stack_base = self.stack.len();
                                    let mut frame = CallFrame::new(func, stack_base);

                                    // First, populate captured variables
                                    let mut next_slot = 0;
                                    for (var_name, value) in &closure.captured_vars {
                                        // Find the slot for this captured variable
                                        if let Some(slot) = frame
                                            .function
                                            .local_names
                                            .iter()
                                            .position(|n| n == var_name)
                                        {
                                            frame.locals[slot] = value.clone();
                                            next_slot = next_slot.max(slot + 1);
                                        }
                                    }

                                    // Then set the arguments (after captured vars)
                                    for (i, arg) in args.into_iter().enumerate() {
                                        if i + next_slot < frame.locals.len() {
                                            frame.locals[i + next_slot] = arg;
                                        }
                                    }

                                    self.frames.push(frame);
                                } else if builtins::is_builtin(func_name) {
                                    let result =
                                        builtins::call_builtin(func_name, &args, &mut self.output)?;
                                    self.stack.push(result);
                                } else {
                                    return Err(format!("undefined function: {}", func_name));
                                }
                            }
                            ClosureBody::Expression(_body_expr) => {
                                // Arrow function - need to evaluate the expression
                                // For VM, arrow functions should be compiled to named functions
                                // Check if we have a compiled version
                                return Err(
                                    "Arrow function expression evaluation not yet supported in VM"
                                        .to_string(),
                                );
                            }
                            _ => return Err("Unsupported closure type".to_string()),
                        }
                    }
                    Value::Object(instance) => {
                        // Object with __invoke magic method
                        let class_name = instance.class_name.clone();
                        if let Some(method) = self.find_method_in_chain(&class_name, "__invoke") {
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(method, stack_base);
                            // Set $this in slot 0
                            frame.locals[0] = Value::Object(instance.clone());
                            // Set arguments starting at slot 1
                            for (i, arg) in args.into_iter().enumerate() {
                                if i + 1 < frame.locals.len() {
                                    frame.locals[i + 1] = arg;
                                }
                            }
                            self.frames.push(frame);
                        } else {
                            return Err(format!("Object of class {} is not callable", class_name));
                        }
                    }
                    _ => return Err(format!("Value is not callable: {:?}", callable)),
                }
            }

            // ==================== OOP Opcodes ====================
            Opcode::NewObject(class_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));

                // Look up class definition
                let class_def = self
                    .classes
                    .get(&class_name)
                    .ok_or_else(|| format!("Class '{}' not found", class_name))?
                    .clone();

                if class_def.is_abstract {
                    return Err(format!("Cannot instantiate abstract class {}", class_name));
                }

                // Create new object instance
                let mut instance = crate::runtime::ObjectInstance::with_hierarchy(
                    class_name.clone(),
                    class_def.parent.clone(),
                    class_def.interfaces.clone(),
                );

                // Initialize properties with defaults from the class and all parents
                // First collect parent properties (reversed order so child overrides parent)
                let mut parent_chain = Vec::new();
                let mut current_parent = class_def.parent.as_ref();
                while let Some(parent_name) = current_parent {
                    if let Some(parent_def) = self.classes.get(parent_name) {
                        parent_chain.push(parent_def.clone());
                        current_parent = parent_def.parent.as_ref();
                    } else {
                        break;
                    }
                }
                // Initialize parent properties first (from oldest ancestor to direct parent)
                for parent_def in parent_chain.iter().rev() {
                    for prop in &parent_def.properties {
                        let default_val = prop.default.clone().unwrap_or(Value::Null);
                        instance
                            .properties
                            .insert(prop.name.clone(), default_val.clone());
                        if prop.readonly {
                            instance.readonly_properties.insert(prop.name.clone());
                            // If property has a default value, mark as initialized
                            if prop.default.is_some() {
                                instance.initialized_readonly.insert(prop.name.clone());
                            }
                        }
                    }
                }
                // Then initialize current class properties (can override parents)
                for prop in &class_def.properties {
                    let default_val = prop.default.clone().unwrap_or(Value::Null);
                    instance
                        .properties
                        .insert(prop.name.clone(), default_val.clone());
                    if prop.readonly {
                        instance.readonly_properties.insert(prop.name.clone());
                        // If property has a default value, mark as initialized
                        if prop.default.is_some() {
                            instance.initialized_readonly.insert(prop.name.clone());
                        }
                    }
                }

                // Push the object
                self.stack.push(Value::Object(instance));

                // Constructor will be called separately via CallConstructor opcode
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
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        // Check for property hook (PHP 8.4)
                        if let Some(class) = self.classes.get(&instance.class_name).cloned() {
                            if let Some(prop_def) =
                                class.properties.iter().find(|p| p.name == prop_name)
                            {
                                if let Some(ref hook_method_name) = prop_def.get_hook {
                                    // Call the get hook method
                                    if let Some(hook_method) =
                                        class.methods.get(hook_method_name).cloned()
                                    {
                                        let stack_base = self.stack.len();
                                        let mut frame = CallFrame::new(hook_method, stack_base);
                                        frame.locals[0] = Value::Object(instance); // $this
                                        self.frames.push(frame);
                                        return Ok(()); // Return value will be pushed by method
                                    }
                                }
                            }
                        }

                        // No hook, try to get the property directly
                        if let Some(value) = instance.properties.get(&prop_name).cloned() {
                            self.stack.push(value);
                        } else {
                            // Property not found, try __get magic method
                            if let Some(get_method) =
                                self.find_method_in_chain(&instance.class_name, "__get")
                            {
                                // Push the property name as argument, then call __get
                                self.stack.push(Value::String(prop_name));
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(get_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = self.stack.pop().unwrap(); // property name
                                self.frames.push(frame);
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                    }
                    Value::EnumCase {
                        enum_name,
                        case_name,
                        backing_value,
                    } => {
                        // Enum cases have ->name and ->value properties
                        let value = match prop_name.as_str() {
                            "name" => Value::String(case_name),
                            "value" => {
                                if let Some(bv) = backing_value {
                                    *bv
                                } else {
                                    return Err(format!(
                                        "Pure enum case {}::{} does not have a 'value' property",
                                        enum_name, case_name
                                    ));
                                }
                            }
                            _ => {
                                return Err(format!(
                                    "Undefined property: {}::{}",
                                    enum_name, prop_name
                                ))
                            }
                        };
                        self.stack.push(value);
                    }
                    _ => {
                        return Err(format!(
                            "Cannot access property of non-object: {:?}",
                            object
                        ))
                    }
                }
            }

            Opcode::StoreProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        // Check for property hooks and visibility (PHP 8.4)
                        if let Some(class) = self.classes.get(&instance.class_name).cloned() {
                            if let Some(prop_def) =
                                class.properties.iter().find(|p| p.name == prop_name)
                            {
                                // Check asymmetric write visibility (PHP 8.4)
                                if let Some(write_vis) = &prop_def.write_visibility {
                                    let current_class = self.get_current_class();
                                    let can_write = match write_vis {
                                        crate::ast::Visibility::Private => {
                                            // Only same class can write
                                            current_class.as_ref() == Some(&instance.class_name)
                                        }
                                        crate::ast::Visibility::Protected => {
                                            // Same class or subclass can write
                                            if let Some(ref curr) = current_class {
                                                curr == &instance.class_name
                                                    || self
                                                        .is_subclass_of(curr, &instance.class_name)
                                            } else {
                                                false
                                            }
                                        }
                                        crate::ast::Visibility::Public => true,
                                    };
                                    if !can_write {
                                        let vis_str = match write_vis {
                                            crate::ast::Visibility::Private => "private",
                                            crate::ast::Visibility::Protected => "protected",
                                            crate::ast::Visibility::Public => "public",
                                        };
                                        return Err(format!(
                                            "Cannot modify {} property {}",
                                            vis_str, prop_name
                                        ));
                                    }
                                }

                                // Check if property has a get hook but no set hook (read-only computed property)
                                if prop_def.get_hook.is_some() && prop_def.set_hook.is_none() {
                                    return Err(format!(
                                        "Cannot write to read-only property {}",
                                        prop_name
                                    ));
                                }

                                if let Some(ref hook_method_name) = prop_def.set_hook {
                                    // Call the set hook method with $value as argument
                                    if let Some(hook_method) =
                                        class.methods.get(hook_method_name).cloned()
                                    {
                                        // DON'T push old object - we'll push modified $this after hook returns
                                        let stack_base = self.stack.len();
                                        let mut frame = CallFrame::new(hook_method, stack_base);
                                        frame.locals[0] = Value::Object(instance); // $this
                                        frame.locals[1] = value; // $value
                                                                 // Track that we need to push modified $this to stack on return
                                        frame.this_source = ThisSource::PropertySetHook;
                                        self.frames.push(frame);
                                        return Ok(());
                                    }
                                }
                            }
                        }

                        // Check if property exists in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // If property is NOT defined in class, try __set magic method
                        if !prop_defined_in_class && !instance.properties.contains_key(&prop_name) {
                            if let Some(set_method) =
                                self.find_method_in_chain(&instance.class_name, "__set")
                            {
                                // Call __set($name, $value)
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(set_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.locals[2] = value; // value
                                frame.this_source = ThisSource::PropertySetHook;
                                self.frames.push(frame);
                                return Ok(());
                            }
                        }

                        // No __set magic method, check readonly constraint
                        let mut instance = instance;
                        if instance.readonly_properties.contains(&prop_name)
                            && instance.initialized_readonly.contains(&prop_name)
                        {
                            return Err(format!("Cannot modify readonly property {}", prop_name));
                        }
                        instance.properties.insert(prop_name.clone(), value.clone());
                        if instance.readonly_properties.contains(&prop_name) {
                            instance.initialized_readonly.insert(prop_name);
                        }
                        // Push modified object back (assignment semantics for chaining)
                        self.stack.push(Value::Object(instance));
                    }
                    _ => return Err("Cannot set property on non-object".to_string()),
                }
            }

            Opcode::StoreCloneProperty(prop_idx) => {
                // Like StoreProperty but validates that the property exists
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(mut instance) => {
                        // Check that property exists on this object
                        if !instance.properties.contains_key(&prop_name) {
                            return Err(format!(
                                "Property '{}' does not exist on class '{}'",
                                prop_name, instance.class_name
                            ));
                        }

                        // Check readonly constraint (clone with allows modifying readonly)
                        // Unlike normal StoreProperty, clone with can modify readonly properties

                        instance.properties.insert(prop_name.clone(), value.clone());
                        // Mark as initialized if it's readonly
                        if instance.readonly_properties.contains(&prop_name) {
                            instance.initialized_readonly.insert(prop_name);
                        }
                        // Push modified object back
                        self.stack.push(Value::Object(instance));
                    }
                    _ => return Err("Cannot set property on non-object".to_string()),
                }
            }

            Opcode::UnsetProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(mut instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // If property is NOT defined in class, try __unset magic method
                        if !prop_defined_in_class {
                            if let Some(unset_method) =
                                self.find_method_in_chain(&instance.class_name, "__unset")
                            {
                                // Call __unset($name)
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(unset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.this_source = ThisSource::PropertySetHook;
                                self.frames.push(frame);
                                return Ok(());
                            }
                        }

                        // No __unset or property is defined in class - try to remove directly
                        if instance.properties.contains_key(&prop_name) {
                            instance.properties.remove(&prop_name);
                        }
                        // If property doesn't exist and no __unset, silently do nothing (PHP behavior)
                    }
                    _ => return Err("Cannot unset property on non-object".to_string()),
                }
            }

            Opcode::IssetProperty(prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // First check if property exists directly on the object
                        if let Some(value) = instance.properties.get(&prop_name) {
                            // Property exists - check if it's not null
                            let is_set = !matches!(value, Value::Null);
                            self.stack.push(Value::Bool(is_set));
                        } else if !prop_defined_in_class {
                            // Property not defined in class, try __isset magic method
                            if let Some(isset_method) =
                                self.find_method_in_chain(&instance.class_name, "__isset")
                            {
                                // Call __isset($name) - it returns a bool
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(isset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                self.frames.push(frame);
                                // The return value of __isset will be pushed to stack
                            } else {
                                // No __isset method and property doesn't exist
                                self.stack.push(Value::Bool(false));
                            }
                        } else {
                            // Property defined in class but not set on instance
                            self.stack.push(Value::Bool(false));
                        }
                    }
                    Value::Null => {
                        // isset on null is always false
                        self.stack.push(Value::Bool(false));
                    }
                    _ => {
                        // For non-objects, property access doesn't make sense
                        self.stack.push(Value::Bool(false));
                    }
                }
            }

            Opcode::UnsetPropertyOnLocal(slot, prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.current_frame().locals[slot as usize].clone();

                match object {
                    Value::Object(mut instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // If property is NOT defined in class, try __unset magic method
                        if !prop_defined_in_class {
                            if let Some(unset_method) =
                                self.find_method_in_chain(&instance.class_name, "__unset")
                            {
                                // Call __unset($name)
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(unset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.this_source = ThisSource::LocalSlot(slot);
                                self.frames.push(frame);
                                return Ok(());
                            }
                        }

                        // No __unset or property is defined in class - try to remove directly
                        if instance.properties.contains_key(&prop_name) {
                            instance.properties.remove(&prop_name);
                            // Update the local variable with the modified instance
                            if let Some(frame) = self.frames.last_mut() {
                                frame.set_local(slot, Value::Object(instance));
                            }
                        }
                    }
                    _ => return Err("Cannot unset property on non-object".to_string()),
                }
            }

            Opcode::UnsetPropertyOnGlobal(var_idx, prop_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.globals.get(&var_name).cloned().unwrap_or(Value::Null);

                match object {
                    Value::Object(mut instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // If property is NOT defined in class, try __unset magic method
                        if !prop_defined_in_class {
                            if let Some(unset_method) =
                                self.find_method_in_chain(&instance.class_name, "__unset")
                            {
                                // Call __unset($name)
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(unset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.this_source = ThisSource::GlobalVar(var_name);
                                self.frames.push(frame);
                                return Ok(());
                            }
                        }

                        // No __unset or property is defined in class - try to remove directly
                        if instance.properties.contains_key(&prop_name) {
                            instance.properties.remove(&prop_name);
                            // Update the global variable with the modified instance
                            self.globals.insert(var_name, Value::Object(instance));
                        }
                    }
                    _ => return Err("Cannot unset property on non-object".to_string()),
                }
            }

            Opcode::IssetPropertyOnLocal(slot, prop_idx) => {
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.current_frame().locals[slot as usize].clone();

                match object {
                    Value::Object(instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // First check if property exists directly on the object
                        if let Some(value) = instance.properties.get(&prop_name) {
                            // Property exists - check if it's not null
                            let is_set = !matches!(value, Value::Null);
                            self.stack.push(Value::Bool(is_set));
                        } else if !prop_defined_in_class {
                            // Property not defined in class, try __isset magic method
                            if let Some(isset_method) =
                                self.find_method_in_chain(&instance.class_name, "__isset")
                            {
                                // Call __isset($name) - it returns a bool
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(isset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.this_source = ThisSource::LocalSlot(slot);
                                self.frames.push(frame);
                            } else {
                                // No __isset method and property doesn't exist
                                self.stack.push(Value::Bool(false));
                            }
                        } else {
                            // Property defined in class but not set on instance
                            self.stack.push(Value::Bool(false));
                        }
                    }
                    Value::Null => {
                        self.stack.push(Value::Bool(false));
                    }
                    _ => {
                        self.stack.push(Value::Bool(false));
                    }
                }
            }

            Opcode::IssetPropertyOnGlobal(var_idx, prop_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let object = self.globals.get(&var_name).cloned().unwrap_or(Value::Null);

                match object {
                    Value::Object(instance) => {
                        // Check if property is defined in class definition
                        let prop_defined_in_class =
                            if let Some(class) = self.classes.get(&instance.class_name) {
                                class.properties.iter().any(|p| p.name == prop_name)
                            } else {
                                false
                            };

                        // First check if property exists directly on the object
                        if let Some(value) = instance.properties.get(&prop_name) {
                            // Property exists - check if it's not null
                            let is_set = !matches!(value, Value::Null);
                            self.stack.push(Value::Bool(is_set));
                        } else if !prop_defined_in_class {
                            // Property not defined in class, try __isset magic method
                            if let Some(isset_method) =
                                self.find_method_in_chain(&instance.class_name, "__isset")
                            {
                                // Call __isset($name) - it returns a bool
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(isset_method, stack_base);
                                frame.locals[0] = Value::Object(instance); // $this
                                frame.locals[1] = Value::String(prop_name); // property name
                                frame.this_source = ThisSource::GlobalVar(var_name);
                                self.frames.push(frame);
                            } else {
                                // No __isset method and property doesn't exist
                                self.stack.push(Value::Bool(false));
                            }
                        } else {
                            // Property defined in class but not set on instance
                            self.stack.push(Value::Bool(false));
                        }
                    }
                    Value::Null => {
                        self.stack.push(Value::Bool(false));
                    }
                    _ => {
                        self.stack.push(Value::Bool(false));
                    }
                }
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
                let value = self.stack.pop().ok_or("Stack underflow")?;

                // Get $this from slot 0, modify it, put it back
                let this = self.current_frame().get_local(0).clone();
                match this {
                    Value::Object(mut instance) => {
                        // Check readonly constraint
                        if instance.readonly_properties.contains(&prop_name)
                            && instance.initialized_readonly.contains(&prop_name)
                        {
                            return Err(format!("Cannot modify readonly property {}", prop_name));
                        }
                        instance.properties.insert(prop_name.clone(), value.clone());
                        if instance.readonly_properties.contains(&prop_name) {
                            instance.initialized_readonly.insert(prop_name);
                        }
                        // Update slot 0 with modified $this
                        self.current_frame_mut()
                            .set_local(0, Value::Object(instance));
                        // Push the assigned value as result (for expression contexts)
                        self.stack.push(value);
                    }
                    _ => return Err("$this is not an object".to_string()),
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

                        // Find method in class or parent chain
                        if let Some(method) = self.find_method_in_chain(&class_name, &method_name) {
                            // Validate parameter types (class type hints only - scalars use coercive mode)
                            for (i, arg) in args.iter().enumerate() {
                                if i < method.param_types.len() {
                                    if let Some(ref type_hint) = method.param_types[i] {
                                        // Only validate class type hints strictly
                                        if self.requires_strict_type_check(type_hint)
                                            && !self.value_matches_type(arg, type_hint)
                                        {
                                            let type_name = self.format_type_hint(type_hint);
                                            let given_type = self.get_value_type_name(arg);
                                            return Err(format!(
                                                "Argument {} passed to {}::{}() must be of type {}, {} given",
                                                i + 1, class_name, method_name, type_name, given_type
                                            ));
                                        }
                                    }
                                }
                            }

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
                        } else if let Some(magic_call) =
                            self.find_method_in_chain(&class_name, "__call")
                        {
                            // Fall back to __call magic method
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(magic_call, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);
                            // Set method name (slot 1)
                            frame.locals[1] = Value::String(method_name);
                            // Set args array (slot 2)
                            let args_array: Vec<(ArrayKey, Value)> = args
                                .into_iter()
                                .enumerate()
                                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                                .collect();
                            frame.locals[2] = Value::Array(args_array);

                            self.frames.push(frame);
                        } else {
                            return Err(format!(
                                "Method '{}' not found on class '{}'",
                                method_name, class_name
                            ));
                        }
                    }
                    _ => return Err("Cannot call method on non-object".to_string()),
                }
            }

            Opcode::CallMethodOnLocal(var_slot, method_idx, arg_count) => {
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Get object from local variable slot in current frame
                let object = self.current_frame().get_local(var_slot).clone();

                match object {
                    Value::Object(instance) => {
                        let class_name = instance.class_name.clone();

                        // Find method in class or parent chain
                        if let Some(method) = self.find_method_in_chain(&class_name, &method_name) {
                            // Validate parameter types
                            for (i, arg) in args.iter().enumerate() {
                                if i < method.param_types.len() {
                                    if let Some(ref type_hint) = method.param_types[i] {
                                        if self.requires_strict_type_check(type_hint)
                                            && !self.value_matches_type(arg, type_hint)
                                        {
                                            let type_name = self.format_type_hint(type_hint);
                                            let given_type = self.get_value_type_name(arg);
                                            return Err(format!(
                                                "Argument {} passed to {}::{}() must be of type {}, {} given",
                                                i + 1, class_name, method_name, type_name, given_type
                                            ));
                                        }
                                    }
                                }
                            }

                            // Create new call frame with $this as first local
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(method, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);

                            // Track source so we can update after method returns
                            frame.this_source = ThisSource::LocalSlot(var_slot);

                            // Set up parameter locals (starting from slot 1)
                            for (i, arg) in args.into_iter().enumerate() {
                                if i + 1 < frame.locals.len() {
                                    frame.locals[i + 1] = arg;
                                }
                            }

                            self.frames.push(frame);
                        } else if let Some(magic_call) =
                            self.find_method_in_chain(&class_name, "__call")
                        {
                            // Fall back to __call magic method
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(magic_call, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);
                            frame.this_source = ThisSource::LocalSlot(var_slot);
                            // Set method name (slot 1)
                            frame.locals[1] = Value::String(method_name);
                            // Set args array (slot 2)
                            let args_array: Vec<(ArrayKey, Value)> = args
                                .into_iter()
                                .enumerate()
                                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                                .collect();
                            frame.locals[2] = Value::Array(args_array);

                            self.frames.push(frame);
                        } else {
                            return Err(format!(
                                "Method '{}' not found on class '{}'",
                                method_name, class_name
                            ));
                        }
                    }
                    _ => return Err("Cannot call method on non-object".to_string()),
                }
            }

            Opcode::CallMethodOnGlobal(var_idx, method_idx, arg_count) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Get object from global variable
                let object = self.globals.get(&var_name).cloned().unwrap_or(Value::Null);

                match object {
                    Value::Object(instance) => {
                        let class_name = instance.class_name.clone();

                        // Find method in class or parent chain
                        if let Some(method) = self.find_method_in_chain(&class_name, &method_name) {
                            // Validate parameter types
                            for (i, arg) in args.iter().enumerate() {
                                if i < method.param_types.len() {
                                    if let Some(ref type_hint) = method.param_types[i] {
                                        if self.requires_strict_type_check(type_hint)
                                            && !self.value_matches_type(arg, type_hint)
                                        {
                                            let type_name = self.format_type_hint(type_hint);
                                            let given_type = self.get_value_type_name(arg);
                                            return Err(format!(
                                                "Argument {} passed to {}::{}() must be of type {}, {} given",
                                                i + 1, class_name, method_name, type_name, given_type
                                            ));
                                        }
                                    }
                                }
                            }

                            // Create new call frame with $this as first local
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(method, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);

                            // Track source so we can update after method returns
                            frame.this_source = ThisSource::GlobalVar(var_name.clone());

                            // Set up parameter locals (starting from slot 1)
                            for (i, arg) in args.into_iter().enumerate() {
                                if i + 1 < frame.locals.len() {
                                    frame.locals[i + 1] = arg;
                                }
                            }

                            self.frames.push(frame);
                        } else if let Some(magic_call) =
                            self.find_method_in_chain(&class_name, "__call")
                        {
                            // Fall back to __call magic method
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(magic_call, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);
                            frame.this_source = ThisSource::GlobalVar(var_name.clone());
                            // Set method name (slot 1)
                            frame.locals[1] = Value::String(method_name);
                            // Set args array (slot 2)
                            let args_array: Vec<(ArrayKey, Value)> = args
                                .into_iter()
                                .enumerate()
                                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                                .collect();
                            frame.locals[2] = Value::Array(args_array);

                            self.frames.push(frame);
                        } else {
                            return Err(format!(
                                "Method '{}' not found on class '{}'",
                                method_name, class_name
                            ));
                        }
                    }
                    _ => return Err("Cannot call method on non-object".to_string()),
                }
            }

            Opcode::CallStaticMethod(class_idx, method_idx, arg_count) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Resolve self/static/parent keywords
                let resolved_class = self.resolve_class_keyword(&class_name)?;

                // Find method through inheritance chain
                if let Some((method, is_instance_method)) =
                    self.find_static_method_in_chain(&resolved_class, &method_name)
                {
                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(method, stack_base);

                    // Set called_class for late static binding
                    frame.called_class = Some(resolved_class.clone());

                    // Set up parameter locals
                    // Instance methods have $this in slot 0, so params start at slot 1
                    // Static methods don't have $this, so params start at slot 0
                    let param_start = if is_instance_method { 1 } else { 0 };
                    for (i, arg) in args.into_iter().enumerate() {
                        let slot = param_start + i;
                        if slot < frame.locals.len() {
                            frame.locals[slot] = arg;
                        }
                    }

                    self.frames.push(frame);
                } else if let Some((magic_call_static, _)) =
                    self.find_static_method_in_chain(&resolved_class, "__callStatic")
                {
                    // Fall back to __callStatic magic method
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(magic_call_static, stack_base);

                    // Set called_class for late static binding
                    frame.called_class = Some(resolved_class.clone());

                    // Set method name (slot 0)
                    frame.locals[0] = Value::String(method_name);
                    // Set args array (slot 1)
                    let args_array: Vec<(ArrayKey, Value)> = args
                        .into_iter()
                        .enumerate()
                        .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                        .collect();
                    frame.locals[1] = Value::Array(args_array);

                    self.frames.push(frame);
                } else if let Some(enum_def) = self.enums.get(&resolved_class).cloned() {
                    // Check for enum built-in static methods
                    match method_name.as_str() {
                        "cases" => {
                            // Return array of all enum cases in declaration order
                            let cases: Vec<(ArrayKey, Value)> = enum_def
                                .case_order
                                .iter()
                                .enumerate()
                                .filter_map(|(i, name)| {
                                    enum_def.cases.get(name).map(|value| {
                                        (
                                            ArrayKey::Integer(i as i64),
                                            Value::EnumCase {
                                                enum_name: resolved_class.clone(),
                                                case_name: name.clone(),
                                                backing_value: value.clone().map(Box::new),
                                            },
                                        )
                                    })
                                })
                                .collect();
                            self.stack.push(Value::Array(cases));
                        }
                        "from" => {
                            // Find case by backing value
                            if args.is_empty() {
                                return Err("from() requires exactly one argument".to_string());
                            }
                            let search_value = &args[0];
                            let mut found = None;
                            for (name, backing) in &enum_def.cases {
                                if let Some(bv) = backing {
                                    if bv == search_value {
                                        found = Some((name.clone(), backing.clone()));
                                        break;
                                    }
                                }
                            }
                            if let Some((name, backing)) = found {
                                self.stack.push(Value::EnumCase {
                                    enum_name: resolved_class.clone(),
                                    case_name: name,
                                    backing_value: backing.map(Box::new),
                                });
                            } else {
                                let value_str = match &search_value {
                                    Value::Integer(n) => n.to_string(),
                                    Value::String(s) => format!("'{}'", s),
                                    other => format!("{:?}", other),
                                };
                                return Err(format!(
                                    "Value '{}' is not a valid backing value for enum {}",
                                    value_str, resolved_class
                                ));
                            }
                        }
                        "tryFrom" => {
                            // Find case by backing value, return null if not found
                            if args.is_empty() {
                                return Err("tryFrom() requires exactly one argument".to_string());
                            }
                            let search_value = &args[0];
                            let mut found = None;
                            for (name, backing) in &enum_def.cases {
                                if let Some(bv) = backing {
                                    if bv == search_value {
                                        found = Some((name.clone(), backing.clone()));
                                        break;
                                    }
                                }
                            }
                            if let Some((name, backing)) = found {
                                self.stack.push(Value::EnumCase {
                                    enum_name: resolved_class.clone(),
                                    case_name: name,
                                    backing_value: backing.map(Box::new),
                                });
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                        _ => {
                            // Check user-defined enum methods
                            if let Some(method) = enum_def.methods.get(&method_name) {
                                let stack_base = self.stack.len();
                                let mut frame = CallFrame::new(method.clone(), stack_base);
                                for (i, arg) in args.into_iter().enumerate() {
                                    if i < frame.locals.len() {
                                        frame.locals[i] = arg;
                                    }
                                }
                                self.frames.push(frame);
                            } else {
                                return Err(format!(
                                    "Static method '{}' not found on enum '{}'",
                                    method_name, resolved_class
                                ));
                            }
                        }
                    }
                } else {
                    return Err(format!(
                        "Static method '{}' not found on class '{}'",
                        method_name, resolved_class
                    ));
                }
            }

            Opcode::CallStaticMethodNamed(class_idx, method_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let method_name = self.current_frame().get_string(method_idx).to_string();

                // Pop args array from stack
                let args_array = self.stack.pop().ok_or("Stack underflow")?;

                // Resolve self/static/parent keywords
                let resolved_class = self.resolve_class_keyword(&class_name)?;

                // Extract arguments from array (mixed positional and named)
                let args_map = if let Value::Array(arr) = args_array {
                    arr
                } else {
                    return Err("Named static method args must be an array".to_string());
                };

                // Separate positional (integer keys) from named (string keys)
                let mut positional = Vec::new();
                let mut named = std::collections::HashMap::new();

                for (key, value) in args_map {
                    match key {
                        ArrayKey::Integer(idx) => {
                            positional.push((idx as usize, value));
                        }
                        ArrayKey::String(name) => {
                            named.insert(name, value);
                        }
                    }
                }

                // Sort positional by index
                positional.sort_by_key(|(idx, _)| *idx);

                // Find method through inheritance chain
                if let Some((method, is_instance_method)) =
                    self.find_static_method_in_chain(&resolved_class, &method_name)
                {
                    // Build final args array by matching to parameters
                    let param_count = method.param_count as usize;
                    let mut final_args = vec![Value::Null; param_count];

                    // First, fill in positional arguments
                    for (i, (_, value)) in positional.into_iter().enumerate() {
                        if i < param_count {
                            final_args[i] = value;
                        }
                    }

                    // Then, fill in named arguments by matching parameter names
                    for (param_idx, param) in method.parameters.iter().enumerate() {
                        if let Some(value) = named.get(&param.name) {
                            if param_idx < param_count {
                                final_args[param_idx] = value.clone();
                            }
                        }
                    }

                    // Create new call frame
                    let stack_base = self.stack.len();
                    let mut frame = CallFrame::new(method, stack_base);

                    // Set called_class for late static binding
                    frame.called_class = Some(resolved_class.clone());

                    // Set up parameter locals
                    let param_start = if is_instance_method { 1 } else { 0 };
                    for (i, arg) in final_args.into_iter().enumerate() {
                        let slot = param_start + i;
                        if slot < frame.locals.len() {
                            frame.locals[slot] = arg;
                        }
                    }

                    self.frames.push(frame);
                } else {
                    return Err(format!(
                        "Static method '{}' not found on class '{}'",
                        method_name, resolved_class
                    ));
                }
            }

            Opcode::LoadStaticProp(class_idx, prop_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let prop_name = self.current_frame().get_string(prop_idx).to_string();

                // Resolve self/static/parent keywords
                let resolved_class = self.resolve_class_keyword(&class_name)?;

                let class_def = self
                    .classes
                    .get(&resolved_class)
                    .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

                let value = class_def
                    .static_properties
                    .get(&prop_name)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Access to undeclared static property {}::${}",
                            resolved_class, prop_name
                        )
                    })?;
                self.stack.push(value);
            }

            Opcode::StoreStaticProp(class_idx, prop_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
                let prop_name = self.current_frame().get_string(prop_idx).to_string();
                let value = self.stack.pop().ok_or("Stack underflow")?;

                // Resolve self/static/parent keywords
                let resolved_class = self.resolve_class_keyword(&class_name)?;

                // Check visibility and readonly constraints
                if let Some(class_def) = self.classes.get(&resolved_class) {
                    // Check readonly
                    if class_def.readonly_static_properties.contains(&prop_name) {
                        return Err(format!(
                            "Cannot modify readonly property {}::${}",
                            resolved_class, prop_name
                        ));
                    }

                    // Check asymmetric write visibility
                    if let Some(prop_def) = class_def
                        .properties
                        .iter()
                        .find(|p| p.name == prop_name && p.is_static)
                    {
                        if let Some(write_vis) = &prop_def.write_visibility {
                            let current_class = self.get_current_class();
                            let can_write = match write_vis {
                                crate::ast::Visibility::Private => {
                                    current_class.as_ref() == Some(&resolved_class)
                                }
                                crate::ast::Visibility::Protected => {
                                    if let Some(ref curr) = current_class {
                                        curr == &resolved_class
                                            || self.is_subclass_of(curr, &resolved_class)
                                    } else {
                                        false
                                    }
                                }
                                crate::ast::Visibility::Public => true,
                            };
                            if !can_write {
                                let vis_str = match write_vis {
                                    crate::ast::Visibility::Private => "private",
                                    crate::ast::Visibility::Protected => "protected",
                                    crate::ast::Visibility::Public => "public",
                                };
                                return Err(format!(
                                    "Cannot modify {} property {}::${}",
                                    vis_str, resolved_class, prop_name
                                ));
                            }
                        }
                    }
                }

                // Need mutable access to update static property
                let class_def = self
                    .classes
                    .get_mut(&resolved_class)
                    .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;
                Arc::make_mut(class_def)
                    .static_properties
                    .insert(prop_name, value.clone());

                // Push value back (assignment returns the assigned value)
                self.stack.push(value);
            }

            Opcode::LoadThis => {
                // $this is stored in slot 0 for instance methods
                let frame = self.current_frame();
                let this = frame
                    .locals
                    .first()
                    .cloned()
                    .ok_or("No $this available in current context")?;
                self.stack.push(this);
            }

            Opcode::InstanceOf(class_idx) => {
                let class_name =
                    Self::normalize_class_name(self.current_frame().get_string(class_idx));
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
                    _ => return Err("__clone method called on non-object".to_string()),
                }
            }

            Opcode::LoadEnumCase(enum_idx, case_idx) => {
                let enum_name =
                    Self::normalize_class_name(self.current_frame().get_string(enum_idx));
                let case_name = self.current_frame().get_string(case_idx).to_string();

                // Look up the enum definition
                let enum_def = self
                    .enums
                    .get(&enum_name)
                    .ok_or_else(|| format!("Enum '{}' not found", enum_name))?
                    .clone();

                // Check if the case exists
                let backing_value = enum_def
                    .cases
                    .get(&case_name)
                    .ok_or_else(|| {
                        format!("Undefined case '{}' for enum '{}'", case_name, enum_name)
                    })?
                    .clone()
                    .map(Box::new);

                // Create the enum case value
                self.stack.push(Value::EnumCase {
                    enum_name,
                    case_name,
                    backing_value,
                });
            }

            Opcode::CallConstructor(arg_count) => {
                // Pop arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.stack.pop().ok_or("Stack underflow")?);
                }
                args.reverse();

                // Pop object (it's below the args on stack)
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        let class_name = instance.class_name.clone();

                        // Find constructor in inheritance chain
                        if let Some(constructor) =
                            self.find_method_in_chain(&class_name, "__construct")
                        {
                            let constructor = constructor.clone();

                            // Create call frame for constructor with $this as first local
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(constructor, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);

                            // Set up parameter locals (starting from slot 1) with type coercion
                            for (i, arg) in args.into_iter().enumerate() {
                                if i + 1 < frame.locals.len() {
                                    let coerced_arg = if i < frame.function.param_types.len() {
                                        if let Some(ref type_hint) = frame.function.param_types[i] {
                                            if !self.requires_strict_type_check(type_hint) {
                                                // First check if the original value CAN be coerced
                                                if !self.value_matches_type(&arg, type_hint) {
                                                    let type_name =
                                                        self.format_type_hint(type_hint);
                                                    let given_type = self.get_value_type_name(&arg);
                                                    return Err(format!(
                                                        "must be of type {}, {} given",
                                                        type_name, given_type
                                                    ));
                                                }
                                                // Then coerce
                                                self.coerce_value_to_type(arg, type_hint)
                                            } else {
                                                // Strict mode - validate without coercion
                                                if !self.value_matches_type_strict(&arg, type_hint)
                                                {
                                                    let type_name =
                                                        self.format_type_hint(type_hint);
                                                    let given_type = self.get_value_type_name(&arg);
                                                    return Err(format!(
                                                        "must be of type {}, {} given",
                                                        type_name, given_type
                                                    ));
                                                }
                                                arg
                                            }
                                        } else {
                                            arg
                                        }
                                    } else {
                                        arg
                                    };
                                    frame.locals[i + 1] = coerced_arg;
                                }
                            }

                            // Mark this as a constructor frame
                            frame.is_constructor = true;

                            self.frames.push(frame);
                        } else {
                            // No constructor, just push the object back
                            self.stack.push(Value::Object(instance));
                        }
                    }
                    _ => return Err("Cannot call constructor on non-object".to_string()),
                }
            }

            Opcode::CallConstructorNamed => {
                // Pop args array
                let args_array = self.stack.pop().ok_or("Stack underflow")?;
                // Pop object (it's below the args on stack)
                let object = self.stack.pop().ok_or("Stack underflow")?;

                match object {
                    Value::Object(instance) => {
                        let class_name = instance.class_name.clone();

                        // Find constructor in inheritance chain
                        if let Some(constructor) =
                            self.find_method_in_chain(&class_name, "__construct")
                        {
                            let constructor = constructor.clone();

                            // Extract arguments from array (mixed positional and named)
                            let args_map = if let Value::Array(arr) = args_array {
                                arr
                            } else {
                                return Err("Named constructor args must be an array".to_string());
                            };

                            // Separate positional (integer keys) from named (string keys)
                            let mut positional = Vec::new();
                            let mut named = std::collections::HashMap::new();

                            for (key, value) in args_map {
                                match key {
                                    ArrayKey::Integer(idx) => {
                                        // Positional arg - store with index
                                        positional.push((idx as usize, value));
                                    }
                                    ArrayKey::String(name) => {
                                        // Named arg
                                        named.insert(name, value);
                                    }
                                }
                            }

                            // Sort positional by index
                            positional.sort_by_key(|(idx, _)| *idx);

                            // Build final args array by matching to parameters
                            let param_count = constructor.param_count as usize;
                            let mut final_args = vec![Value::Null; param_count];

                            // First, fill in positional arguments
                            for (i, (_, value)) in positional.into_iter().enumerate() {
                                if i < param_count {
                                    final_args[i] = value;
                                }
                            }

                            // Then, fill in named arguments by matching parameter names
                            for (param_idx, param) in constructor.parameters.iter().enumerate() {
                                if let Some(value) = named.get(&param.name) {
                                    if param_idx < param_count {
                                        final_args[param_idx] = value.clone();
                                    }
                                }
                            }

                            // Create call frame for constructor with $this as first local
                            let stack_base = self.stack.len();
                            let mut frame = CallFrame::new(constructor, stack_base);

                            // Set $this (slot 0)
                            frame.locals[0] = Value::Object(instance);

                            // Set up parameter locals (starting from slot 1)
                            for (i, arg) in final_args.into_iter().enumerate() {
                                if i + 1 < frame.locals.len() {
                                    frame.locals[i + 1] = arg;
                                }
                            }

                            // Mark this as a constructor frame
                            frame.is_constructor = true;

                            self.frames.push(frame);
                        } else {
                            // No constructor, just push the object back
                            self.stack.push(Value::Object(instance));
                        }
                    }
                    _ => return Err("Cannot call constructor on non-object".to_string()),
                }
            }

            // ==================== Exception Handling ====================
            Opcode::Throw => {
                let exception = self.stack.pop().ok_or("Stack underflow")?;

                let current_frame_depth = self.frames.len();
                let current_ip = self.current_frame().ip;

                // Search for exception handler (can be in current or ancestor frames)
                let mut handler_info: Option<(usize, usize, usize)> = None; // (catch_offset, frame_depth, handler_idx)

                // Look for a handler, searching from newest to oldest
                for (handler_idx, handler) in self.handlers.iter().enumerate().rev() {
                    // Skip handlers from deeper frames (they've been popped)
                    if handler.frame_depth > current_frame_depth {
                        continue;
                    }

                    let handler_is_active = if handler.frame_depth == current_frame_depth {
                        // Handler is in current frame - check IP range
                        current_ip >= handler.try_start as usize
                            && (handler.try_end == 0 || current_ip < handler.try_end as usize)
                    } else {
                        // Handler is in an ancestor frame - always active if frame exists
                        // (exception propagates up through function calls)
                        handler.try_end == 0 || handler.try_end > handler.try_start
                    };

                    if handler_is_active {
                        handler_info = Some((
                            handler.catch_offset as usize,
                            handler.frame_depth,
                            handler_idx,
                        ));
                        break;
                    }
                }

                if let Some((catch_offset, target_frame_depth, handler_idx)) = handler_info {
                    // Unwind frames until we reach the target frame
                    while self.frames.len() > target_frame_depth {
                        self.frames.pop();
                    }

                    // Disable the handler by setting try_end
                    if let Some(handler) = self.handlers.get_mut(handler_idx) {
                        if handler.try_end == 0 {
                            handler.try_end = current_ip as u32;
                        }
                    }

                    // Jump to catch block with exception on stack
                    self.stack.push(exception);
                    if let Some(frame) = self.frames.last_mut() {
                        frame.jump_to(catch_offset);
                    }
                } else {
                    // Format uncaught exception error
                    let error_msg = if let Value::Object(ref obj) = exception {
                        // Try to get the message property
                        if let Some(msg_value) = obj.properties.get("message") {
                            match msg_value {
                                Value::String(s) if !s.is_empty() => {
                                    format!("Uncaught {}: {}", obj.class_name, s)
                                }
                                _ => format!("Uncaught {}", obj.class_name),
                            }
                        } else {
                            format!("Uncaught {}", obj.class_name)
                        }
                    } else {
                        format!("Uncaught exception: {:?}", exception)
                    };
                    return Err(error_msg);
                }
            }

            Opcode::TryStart(catch_offset, finally_offset) => {
                // Register exception handler
                let try_start = self.current_frame().ip as u32;
                let frame_depth = self.frames.len();
                self.handlers.push(ExceptionHandler {
                    try_start,
                    try_end: 0, // Will be set by TryEnd
                    catch_offset,
                    catch_class: String::new(), // For now, catch all exceptions
                    catch_var: String::new(),
                    finally_offset,
                    stack_depth: self.stack.len(),
                    frame_depth,
                });
            }

            Opcode::TryEnd => {
                // Mark the end of try block in the most recent handler
                let current_ip = self.current_frame().ip as u32;
                if let Some(handler) = self.handlers.last_mut() {
                    handler.try_end = current_ip;
                }
                // Don't pop the handler here - it needs to remain active for exception handling
                // The handler will be cleaned up when we exit the function
            }

            Opcode::FinallyStart => {
                // Mark that we're inside a finally block
                // Nothing special needed here
            }

            Opcode::FinallyEnd => {
                // Check if there's a pending return to complete
                if self.pending_return.is_some() {
                    // Signal to the main loop that we need to complete the return
                    // The return value is stored in pending_return
                    return Err("__FINALLY_RETURN__".to_string());
                }
                // Otherwise, just continue normal execution
            }

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

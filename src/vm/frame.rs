//! Call frame management for the bytecode VM
//!
//! This module defines the call frame structure used to track
//! function execution state in the VM.

use crate::interpreter::Value;
use crate::vm::opcode::CompiledFunction;
use std::collections::HashMap;
use std::sync::Arc;

/// A call frame represents a single function invocation
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Reference to the compiled function being executed
    pub function: Arc<CompiledFunction>,
    /// Instruction pointer (index into bytecode)
    pub ip: usize,
    /// Base index in the value stack for this frame
    pub stack_base: usize,
    /// Local variables (fixed-size slots for fast access)
    pub locals: Vec<Value>,
    /// Saved global variables (for restoring after function call)
    pub saved_globals: Option<HashMap<String, Value>>,
    /// Current `$this` object (for methods)
    pub this: Option<crate::interpreter::ObjectInstance>,
    /// Called class name (for late static binding)
    pub called_class: Option<String>,
    /// Whether this is a constructor frame (returns $this on completion)
    pub is_constructor: bool,
}

impl CallFrame {
    /// Create a new call frame for a function
    pub fn new(function: Arc<CompiledFunction>, stack_base: usize) -> Self {
        let local_count = function.local_count as usize;
        Self {
            function,
            ip: 0,
            stack_base,
            locals: vec![Value::Null; local_count],
            saved_globals: None,
            this: None,
            called_class: None,
            is_constructor: false,
        }
    }

    /// Create a new call frame for a method
    pub fn new_method(
        function: Arc<CompiledFunction>,
        stack_base: usize,
        this: crate::interpreter::ObjectInstance,
        called_class: String,
    ) -> Self {
        let local_count = function.local_count as usize;
        Self {
            function,
            ip: 0,
            stack_base,
            locals: vec![Value::Null; local_count],
            saved_globals: None,
            this: Some(this),
            called_class: Some(called_class),
            is_constructor: false,
        }
    }

    /// Get the current instruction offset
    #[inline]
    pub fn current_ip(&self) -> usize {
        self.ip
    }

    /// Advance the instruction pointer and return the new position
    #[inline]
    pub fn advance(&mut self) -> usize {
        self.ip += 1;
        self.ip
    }

    /// Jump to a specific instruction offset
    #[inline]
    pub fn jump_to(&mut self, offset: usize) {
        self.ip = offset;
    }

    /// Get a local variable by slot index
    #[inline]
    pub fn get_local(&self, slot: u16) -> &Value {
        &self.locals[slot as usize]
    }

    /// Set a local variable by slot index
    #[inline]
    pub fn set_local(&mut self, slot: u16, value: Value) {
        self.locals[slot as usize] = value;
    }

    /// Get a string from the function's string pool
    #[inline]
    pub fn get_string(&self, index: u32) -> &str {
        &self.function.strings[index as usize]
    }

    /// Get a constant from the function's constant pool
    #[inline]
    pub fn get_constant(&self, index: u32) -> &crate::vm::opcode::Constant {
        &self.function.constants[index as usize]
    }
}

/// Loop context for tracking break/continue targets
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Instruction offset for continue
    pub continue_target: u32,
    /// Instruction offset for break
    pub break_target: u32,
    /// Stack depth at loop start (for proper cleanup)
    pub stack_depth: usize,
}

/// Exception handler for try/catch/finally
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    /// Start of the try block
    pub try_start: u32,
    /// End of the try block
    pub try_end: u32,
    /// Catch block offset (0 if no catch)
    pub catch_offset: u32,
    /// Exception class to catch (empty for catch-all)
    pub catch_class: String,
    /// Variable name to bind exception to
    pub catch_var: String,
    /// Finally block offset (0 if no finally)
    pub finally_offset: u32,
    /// Stack depth at try block start
    pub stack_depth: usize,
}

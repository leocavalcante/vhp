//! Bytecode Virtual Machine for VHP
//!
//! This module implements a stack-based bytecode VM that executes
//! compiled PHP bytecode. The VM is designed to be faster than
//! tree-walking interpretation for hot paths and repeated execution.

pub mod autoload;
pub mod builtins;
pub mod class;
pub mod class_registration;
pub mod compiled_types;
pub mod compiler;
pub mod execution;
pub mod frame;
pub mod methods;
pub mod objects;
pub mod opcode;
pub mod reflection;

mod helpers;
mod ops;
mod type_validation;

pub use helpers::clear_required_files;

use crate::runtime::Value;
use class::{CompiledClass, CompiledEnum, CompiledInterface, CompiledTrait};
use frame::{CallFrame, ExceptionHandler, LoopContext};
use opcode::{CompiledFunction, Opcode};
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
        class_registration::register_builtin_classes(&mut self.classes);
    }

    /// Execute a compiled function
    pub fn execute(&mut self, function: Arc<CompiledFunction>) -> Result<Value, String> {
        execution::execute_vm(self, function)
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
                ops::execute_load_const(self, constant)?;
            }

            // ==================== Variables ====================
            Opcode::LoadVar(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                ops::execute_load_var(self, name);
            }
            Opcode::StoreVar(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                ops::execute_store_var(self, name)?;
            }
            Opcode::LoadFast(slot) => ops::execute_load_fast(self, slot),
            Opcode::StoreFast(slot) => ops::execute_store_fast(self, slot)?,
            Opcode::LoadGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                ops::execute_load_global(self, name);
            }
            Opcode::StoreGlobal(idx) => {
                let name = self.current_frame().get_string(idx).to_string();
                ops::execute_store_global(self, name)?;
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
            Opcode::Eq => ops::execute_eq(self)?,
            Opcode::Ne => ops::execute_ne(self)?,
            Opcode::Identical => ops::execute_identical(self)?,
            Opcode::NotIdentical => ops::execute_not_identical(self)?,
            Opcode::Lt => ops::execute_lt(self)?,
            Opcode::Le => ops::execute_le(self)?,
            Opcode::Gt => ops::execute_gt(self)?,
            Opcode::Ge => ops::execute_ge(self)?,
            Opcode::Spaceship => ops::execute_spaceship(self)?,

            // ==================== Logical ====================
            Opcode::Not => ops::execute_not(self)?,
            Opcode::And => ops::execute_and(self)?,
            Opcode::Or => ops::execute_or(self)?,
            Opcode::Xor => ops::execute_xor(self)?,

            // ==================== Control Flow ====================
            Opcode::Jump(offset) => ops::execute_jump(self, offset),
            Opcode::JumpIfFalse(offset) => ops::execute_jump_if_false(self, offset)?,
            Opcode::JumpIfTrue(offset) => ops::execute_jump_if_true(self, offset)?,
            Opcode::JumpIfNull(offset) => ops::execute_jump_if_null(self, offset)?,
            Opcode::JumpIfNotNull(offset) => ops::execute_jump_if_not_null(self, offset)?,
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
            Opcode::ArrayCount => ops::execute_array_count(self)?,
            Opcode::ArrayGetKeyAt => ops::execute_array_get_key_at(self)?,
            Opcode::ArrayGetValueAt => ops::execute_array_get_value_at(self)?,
            Opcode::ToArray => ops::execute_to_array(self)?,

            // ==================== Stack Manipulation ====================
            Opcode::Pop => ops::execute_pop(self),
            Opcode::Dup => ops::execute_dup(self)?,
            Opcode::Swap => ops::execute_swap(self)?,

            // ==================== Type Operations ====================
            Opcode::Cast(cast_type) => ops::execute_cast(self, cast_type)?,

            // ==================== Null Coalescing ====================
            Opcode::NullCoalesce => ops::execute_null_coalesce(self)?,

            // ==================== Output ====================
            Opcode::Echo => ops::execute_echo(self)?,
            Opcode::Print => ops::execute_print(self)?,

            // ==================== Function Calls ====================
            Opcode::Call(name_idx, arg_count) => {
                let func_name = self.current_frame().get_string(name_idx).to_string();
                ops::execute_call(self, func_name, arg_count)?;
            }

            Opcode::CallSpread(name_idx) => {
                crate::vm::ops::execute_call_spread(self, name_idx)?;
            }

            Opcode::CallNamed(name_idx) => {
                crate::vm::ops::execute_call_named_args(self, name_idx)?;
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
                ops::execute_new_fiber(self)?;
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
                ops::execute_isset_property(self, prop_name)?;
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
                ops::execute_unset_var(self, var_name);
            }

            Opcode::UnsetArrayElement => {
                ops::execute_unset_array_element(self)?;
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
                ops::execute_instance_of(self, class_name)?;
            }

            Opcode::Clone => ops::execute_clone(self)?,

            Opcode::LoadEnumCase(enum_idx, case_idx) => {
                let enum_name =
                    Self::normalize_class_name(self.current_frame().get_string(enum_idx));
                let case_name = self.current_frame().get_string(case_idx).to_string();
                ops::execute_load_enum_case(self, enum_name, case_name)?
            }

            Opcode::CallConstructor(arg_count) => {
                ops::execute_call_constructor(self, arg_count)?;
            }

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
                ops::execute_create_closure(self, func_name, capture_count)?;
            }

            Opcode::CaptureVar(var_idx) => {
                let var_name = self.current_frame().get_string(var_idx).to_string();
                ops::execute_capture_var(self, var_name);
            }

            Opcode::CreateMethodClosure => {
                ops::execute_create_method_closure(self)?;
            }

            Opcode::CreateStaticMethodClosure => {
                ops::execute_create_static_method_closure(self)?;
            }

            // ==================== Array Operations ====================
            Opcode::ArrayUnpack => {
                ops::execute_array_unpack(self)?;
            }

            // ==================== Generator Methods ====================
            Opcode::GeneratorCurrent => {
                ops::execute_generator_current(self)?;
            }
            Opcode::GeneratorKey => {
                ops::execute_generator_key(self)?;
            }
            Opcode::GeneratorNext => {
                ops::execute_generator_next(self)?;
            }
            Opcode::GeneratorRewind => {
                ops::execute_generator_rewind(self)?;
            }
            Opcode::GeneratorValid => {
                ops::execute_generator_valid(self)?;
            }
            Opcode::SetCurrentFiber => {
                ops::execute_set_current_fiber(self)?;
            }
            Opcode::GetCurrentFiber => {
                ops::execute_get_current_fiber(self)?;
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
}

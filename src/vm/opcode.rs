//! Bytecode instruction set for VHP VM
//!
//! This module defines the complete instruction set for the bytecode VM.
//! The VM uses a stack-based architecture with ~70 core opcodes.

pub use super::compiled_types::{CompiledFunction, Constant};

/// Bytecode instruction type
///
/// Stack-based VM instructions. Most instructions operate on values
/// at the top of the operand stack.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Many opcode variants are defined for future use
pub enum Opcode {
    // ==================== Literals & Constants ====================
    /// Push null value onto stack
    PushNull,
    /// Push true onto stack
    PushTrue,
    /// Push false onto stack
    PushFalse,
    /// Push integer literal onto stack
    PushInt(i64),
    /// Push float literal onto stack
    PushFloat(f64),
    /// Push string from constant pool (by index)
    PushString(u32),
    /// Load constant from pool by index
    LoadConst(u32),
    // ==================== Variables ====================
    /// Load variable by name index (from string pool)
    LoadVar(u32),
    /// Store top of stack to variable by name index
    StoreVar(u32),
    /// Load local variable by slot index (fast path for known locals)
    LoadFast(u16),
    /// Store to local variable slot (fast path)
    StoreFast(u16),
    /// Load from global scope by name index
    LoadGlobal(u32),
    /// Store to global scope by name index
    StoreGlobal(u32),

    // ==================== Arithmetic ====================
    /// Add: pop two values, push sum
    Add,
    /// Subtract: pop two values, push difference
    Sub,
    /// Multiply: pop two values, push product
    Mul,
    /// Divide: pop two values, push quotient
    Div,
    /// Modulo: pop two values, push remainder
    Mod,
    /// Power: pop two values, push base^exponent
    Pow,
    /// Negate: negate top of stack
    Neg,

    // ==================== String Operations ====================
    /// String concatenation: pop two values, push concatenated string
    Concat,

    // ==================== Comparison ====================
    /// Equal (==): pop two values, push bool result
    Eq,
    /// Not equal (!=): pop two values, push bool result
    Ne,
    /// Identical (===): pop two values, push bool result
    Identical,
    /// Not identical (!==): pop two values, push bool result
    NotIdentical,
    /// Less than (<): pop two values, push bool result
    Lt,
    /// Less than or equal (<=): pop two values, push bool result
    Le,
    /// Greater than (>): pop two values, push bool result
    Gt,
    /// Greater than or equal (>=): pop two values, push bool result
    Ge,
    /// Spaceship (<=>): pop two values, push -1, 0, or 1
    Spaceship,

    // ==================== Logical ====================
    /// Logical NOT: pop one value, push negated bool
    Not,
    /// Logical AND: pop two values, push bool result
    And,
    /// Logical OR: pop two values, push bool result
    Or,
    /// Logical XOR: pop two values, push bool result
    Xor,

    // ==================== Bitwise ====================
    /// Bitwise AND: pop two values, push result
    BitwiseAnd,
    /// Bitwise OR: pop two values, push result
    BitwiseOr,
    /// Bitwise XOR: pop two values, push result
    BitwiseXor,
    /// Bitwise NOT: pop one value, push result
    BitwiseNot,
    /// Left shift: pop two values, push result
    ShiftLeft,
    /// Right shift: pop two values, push result
    ShiftRight,

    // ==================== Control Flow ====================
    /// Unconditional jump to instruction offset
    Jump(u32),
    /// Jump if top of stack is falsy (pops value)
    JumpIfFalse(u32),
    /// Jump if top of stack is truthy (pops value)
    JumpIfTrue(u32),
    /// Jump if top of stack is null (for ??), doesn't pop
    JumpIfNull(u32),
    /// Jump if top of stack is not null (for ??), doesn't pop
    JumpIfNotNull(u32),
    /// Call function: name index, arg count
    Call(u32, u8),
    /// Call function with spread: name index (stack: args_array -> result)
    CallSpread(u32),
    /// Call function with named arguments: name index (stack: args_assoc_array -> result)
    CallNamed(u32),
    /// Call built-in function: name index, arg count
    CallBuiltin(u32, u8),
    /// Call built-in function with spread: name index (stack: args_array -> result)
    CallBuiltinSpread(u32),
    /// Call built-in function with named arguments: name index (stack: args_assoc_array -> result)
    CallBuiltinNamed(u32),
    /// Call a callable value (closure, first-class callable): arg count (stack: callable, args... -> result)
    CallCallable(u8),
    /// Array merge for spread operator: merge second array into first (stack: array1, array2 -> merged_array)
    ArrayMerge,
    /// Return from function (with value from stack)
    Return,
    /// Return null from function
    ReturnNull,

    // ==================== Generators ====================
    /// Yield value from generator
    Yield,
    /// Yield from iterator
    YieldFrom,
    /// Generator::current() - get current yielded value
    GeneratorCurrent,
    /// Generator::key() - get current yielded key
    GeneratorKey,
    /// Generator::next() - advance to next yield
    GeneratorNext,
    /// Generator::rewind() - rewind to beginning
    GeneratorRewind,
    /// Generator::valid() - check if generator is still valid
    GeneratorValid,

    // ==================== Loop Control ====================
    /// Break out of loop
    Break,
    /// Continue to next iteration
    Continue,
    /// Set up loop context (for break/continue targets)
    LoopStart(u32, u32), // (continue_offset, break_offset)
    /// End loop context
    LoopEnd,

    // ==================== Arrays ====================
    /// Create new array with N elements on stack
    NewArray(u16),
    /// Push value to array (stack: array, value -> array)
    ArrayPush,
    /// Set array element (stack: array, key, value -> array)
    ArraySet,
    /// Get array element (stack: array, key -> value)
    ArrayGet,
    /// Append to array (stack: array, value -> array)
    ArrayAppend,
    /// Unpack/spread array onto stack
    ArrayUnpack,
    /// Get array length (optimized count())
    ArrayCount,
    /// Get key at iterator index (stack: array, index -> key)
    ArrayGetKeyAt,
    /// Get value at iterator index (stack: array, index -> value)
    ArrayGetValueAt,
    /// Convert iterable to array (handles arrays and generators)
    /// Stack: iterable -> array
    ToArray,

    // ==================== Objects ====================
    /// Create new object: class name index
    NewObject(u32),
    /// Create new Fiber with callback (stack: callback -> Fiber object)
    NewFiber,
    /// Load property: property name index (stack: object -> value)
    LoadProperty(u32),
    /// Store property: property name index (stack: object, value -> object)
    StoreProperty(u32),
    /// Store property on $this and update local slot 0: property name index (stack: value -> void)
    StoreThisProperty(u32),
    /// Store property in clone with - validates property exists (stack: object, value -> object)
    StoreCloneProperty(u32),
    /// Unset property: property name index (stack: object -> void)
    /// Calls __unset magic method if property doesn't exist or can't be unset
    UnsetProperty(u32),
    /// Unset property on a local variable: var slot, property name index
    /// Tracks source variable so __unset modifications persist
    UnsetPropertyOnLocal(u16, u32),
    /// Unset property on a global variable: var name index, property name index
    /// Tracks source variable so __unset modifications persist
    UnsetPropertyOnGlobal(u32, u32),
    /// Check if property is set: property name index (stack: object -> bool)
    /// Calls __isset magic method if property doesn't exist on object
    IssetProperty(u32),
    /// Check if property is set on local variable: var slot, property name index
    /// Tracks source so __isset can be called with proper context
    IssetPropertyOnLocal(u16, u32),
    /// Check if property is set on global variable: var name index, property name index
    /// Tracks source so __isset can be called with proper context
    IssetPropertyOnGlobal(u32, u32),
    /// Unset variable: variable name index (removes from global scope)
    UnsetVar(u32),
    /// Unset array element (stack: array, key -> void)
    UnsetArrayElement,
    /// Load static property: class name index, property name index
    LoadStaticProp(u32, u32),
    /// Store static property: class name index, property name index
    StoreStaticProp(u32, u32),
    /// Call method: method name index, arg count (stack: object, args... -> result)
    CallMethod(u32, u8),
    /// Call method on a local variable: var slot, method name index, arg count
    /// This tracks the source variable so $this modifications persist
    CallMethodOnLocal(u16, u32, u8),
    /// Call method on a global variable: var name index, method name index, arg count
    /// This tracks the source variable so $this modifications persist
    CallMethodOnGlobal(u32, u32, u8),
    /// Call static method: class name index, method name index, arg count
    CallStaticMethod(u32, u32, u8),
    /// Call static method with named arguments: class name index, method name index (stack: args_array -> result)
    CallStaticMethodNamed(u32, u32),
    /// Load $this onto stack
    LoadThis,
    /// instanceof check: class name index (stack: object -> bool)
    InstanceOf(u32),
    /// Get current Fiber for Fiber::getCurrent() (stack: -> Fiber|null)
    GetCurrentFiber,
    /// Set current Fiber (stack: Fiber|null ->)
    SetCurrentFiber,
    /// Clone object (stack: object -> cloned_object)
    Clone,
    /// Call constructor on object: arg count (stack: object, args... -> object)
    CallConstructor(u8),
    /// Call constructor with named arguments (stack: object, args_array -> object)
    CallConstructorNamed,
    /// Load enum case: enum name index, case name index
    LoadEnumCase(u32, u32),
    /// Enum::from() - look up case by backing value, throws if not found
    EnumFromValue(u32),
    /// Enum::tryFrom() - look up case by backing value, returns null if not found
    EnumTryFromValue(u32),

    // ==================== Stack Manipulation ====================
    /// Pop and discard top of stack
    Pop,
    /// Duplicate top of stack
    Dup,
    /// Swap top two stack values
    Swap,

    // ==================== Type Operations ====================
    /// Cast to type (stack: value -> casted_value)
    Cast(CastType),
    /// Check if value matches type (for type hints)
    TypeCheck(u32), // type name index

    // ==================== Null Coalescing ====================
    /// Null coalescing: if top is null, pop and use next, else keep top
    NullCoalesce,

    // ==================== Ternary ====================
    /// Ternary condition marker (used with jumps)
    Ternary,

    // ==================== Match Expression ====================
    /// Match expression start: number of arms
    MatchStart(u16),
    /// Match arm check (stack: subject, value -> bool)
    MatchArm,
    /// Match default arm
    MatchDefault,

    // ==================== Exception Handling ====================
    /// Set up try block: catch offset, finally offset (0 if none)
    TryStart(u32, u32),
    /// End try block
    TryEnd,
    /// Throw exception (stack: exception -> throws)
    Throw,
    /// Catch block: exception class name index, variable name index
    Catch(u32, u32),
    /// Finally block start
    FinallyStart,
    /// Finally block end
    FinallyEnd,

    // ==================== Closure ====================
    /// Create closure: function index, captured var count
    CreateClosure(u32, u8),
    /// Capture variable for closure
    CaptureVar(u32),

    // ==================== Increment/Decrement ====================
    /// Pre-increment (++$x)
    PreInc,
    /// Pre-decrement (--$x)
    PreDec,
    /// Post-increment ($x++)
    PostInc,
    /// Post-decrement ($x--)
    PostDec,

    // ==================== Utility ====================
    /// No operation
    Nop,
    /// Echo value from stack
    Echo,
    /// Print value from stack (returns 1)
    Print,
}

/// Cast type for Cast opcode
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)] // All variants defined for future use
pub enum CastType {
    Int,
    Float,
    String,
    Bool,
    Array,
    Object,
}

impl Opcode {
    /// Get the stack effect of this opcode (positive = pushes, negative = pops)
    #[allow(dead_code)]
    pub fn stack_effect(&self) -> i32 {
        match self {
            // Pushes: +1
            Opcode::PushNull
            | Opcode::PushTrue
            | Opcode::PushFalse
            | Opcode::PushInt(_)
            | Opcode::PushFloat(_)
            | Opcode::PushString(_)
            | Opcode::LoadConst(_)
            | Opcode::LoadVar(_)
            | Opcode::LoadFast(_)
            | Opcode::LoadGlobal(_)
            | Opcode::LoadThis
            | Opcode::GetCurrentFiber
            | Opcode::Dup => 1,

            // Pops 1, pushes 1: 0
            Opcode::Neg
            | Opcode::Not
            | Opcode::BitwiseNot
            | Opcode::Cast(_)
            | Opcode::TypeCheck(_)
            | Opcode::InstanceOf(_)
            | Opcode::Clone
            | Opcode::PreInc
            | Opcode::PreDec
            | Opcode::PostInc
            | Opcode::PostDec
            | Opcode::ArrayCount => 0,

            // Pops 2, pushes 1: -1
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Mod
            | Opcode::Pow
            | Opcode::Concat
            | Opcode::Eq
            | Opcode::Ne
            | Opcode::Identical
            | Opcode::NotIdentical
            | Opcode::Lt
            | Opcode::Le
            | Opcode::Gt
            | Opcode::Ge
            | Opcode::Spaceship
            | Opcode::And
            | Opcode::Or
            | Opcode::Xor
            | Opcode::BitwiseAnd
            | Opcode::BitwiseOr
            | Opcode::BitwiseXor
            | Opcode::ShiftLeft
            | Opcode::ShiftRight
            | Opcode::NullCoalesce
            | Opcode::ArrayGet
            | Opcode::ArrayPush
            | Opcode::ArrayGetKeyAt
            | Opcode::ArrayGetValueAt => -1,

            // Pops 1, pushes 1: 0
            Opcode::ToArray => 0,

            // Pops 3, pushes 1: -2
            Opcode::ArraySet => -2,

            // Pops 1, pushes 0: -1
            Opcode::Pop
            | Opcode::StoreVar(_)
            | Opcode::StoreFast(_)
            | Opcode::StoreGlobal(_)
            | Opcode::Return
            | Opcode::Echo
            | Opcode::JumpIfFalse(_)
            | Opcode::JumpIfTrue(_)
            | Opcode::Throw
            | Opcode::Yield => -1,

            // Special cases
            Opcode::ReturnNull | Opcode::YieldFrom => -1,
            Opcode::Jump(_)
            | Opcode::Nop
            | Opcode::LoopStart(_, _)
            | Opcode::LoopEnd
            | Opcode::TryStart(_, _)
            | Opcode::TryEnd
            | Opcode::Break
            | Opcode::Continue
            | Opcode::Swap => 0,

            // Function calls: variable effect (handled specially)
            Opcode::Call(_, n) | Opcode::CallBuiltin(_, n) => -(*n as i32),
            Opcode::CallSpread(_) | Opcode::CallBuiltinSpread(_) => 0, // pops array, pushes result
            Opcode::CallNamed(_) | Opcode::CallBuiltinNamed(_) => 0, // pops assoc array, pushes result
            Opcode::CallCallable(n) => -(*n as i32) - 1 + 1, // pops callable + args, pushes result
            Opcode::CallMethod(_, n) => -(*n as i32) - 1 + 1, // pops object + args, pushes result
            Opcode::CallMethodOnLocal(_, _, n) => -(*n as i32) + 1, // pops args only (loads from local), pushes result
            Opcode::CallMethodOnGlobal(_, _, n) => -(*n as i32) + 1, // pops args only (loads from global), pushes result
            Opcode::CallStaticMethod(_, _, n) => -(*n as i32) + 1,
            Opcode::CallStaticMethodNamed(_, _) => 0, // pops array, pushes result
            Opcode::CallConstructor(n) => -(*n as i32), // pops args, uses object in-place
            Opcode::CallConstructorNamed => -1,       // pops args array, uses object in-place

            // Object/Array/Generator operations grouped by stack effect
            // Push 1: NewObject, IssetProperty*, LoadStatic*, LoadEnum*
            Opcode::NewObject(_)
            | Opcode::IssetPropertyOnLocal(_, _)
            | Opcode::IssetPropertyOnGlobal(_, _)
            | Opcode::LoadStaticProp(_, _)
            | Opcode::LoadEnumCase(_, _)
            | Opcode::Catch(_, _) => 1,
            // Push 0: Most others
            Opcode::NewFiber
            | Opcode::LoadProperty(_)
            | Opcode::UnsetPropertyOnLocal(_, _)
            | Opcode::UnsetPropertyOnGlobal(_, _)
            | Opcode::StoreThisProperty(_)
            | Opcode::IssetProperty(_)
            | Opcode::UnsetVar(_)
            | Opcode::EnumFromValue(_)
            | Opcode::EnumTryFromValue(_)
            | Opcode::ArrayUnpack
            | Opcode::GeneratorCurrent
            | Opcode::GeneratorKey
            | Opcode::GeneratorNext
            | Opcode::GeneratorRewind
            | Opcode::GeneratorValid
            | Opcode::SetCurrentFiber
            | Opcode::JumpIfNull(_)
            | Opcode::JumpIfNotNull(_)
            | Opcode::Ternary
            | Opcode::MatchStart(_)
            | Opcode::MatchDefault
            | Opcode::FinallyStart
            | Opcode::FinallyEnd
            | Opcode::CaptureVar(_)
            | Opcode::Print => 0,
            // Pop 1: Store*, UnsetProperty, ArrayAppend/Merge, MatchArm
            Opcode::StoreProperty(_)
            | Opcode::StoreStaticProp(_, _)
            | Opcode::StoreCloneProperty(_)
            | Opcode::UnsetProperty(_)
            | Opcode::ArrayAppend
            | Opcode::ArrayMerge
            | Opcode::MatchArm => -1,
            // Pop 2: UnsetArrayElement
            Opcode::UnsetArrayElement => -2,
            // Special: NewArray, CreateClosure
            Opcode::NewArray(n) => 1 - (*n as i32) * 2,
            Opcode::CreateClosure(_, n) => 1 - (*n as i32),
        }
    }
}

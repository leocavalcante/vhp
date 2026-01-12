//! Bytecode instruction set for VHP VM
//!
//! This module defines the complete instruction set for the bytecode VM.
//! The VM uses a stack-based architecture with ~70 core opcodes.

/// Bytecode instruction type
///
/// Stack-based VM instructions. Most instructions operate on values
/// at the top of the operand stack.
#[derive(Debug, Clone, PartialEq)]
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

    // ==================== Objects ====================
    /// Create new object: class name index
    NewObject(u32),
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
    /// Clone object (stack: object -> cloned_object)
    Clone,
    /// Call constructor on object: arg count (stack: object, args... -> object)
    CallConstructor(u8),
    /// Call constructor with named arguments (stack: object, args_array -> object)
    CallConstructorNamed,
    /// Load enum case: enum name index, case name index
    LoadEnumCase(u32, u32),

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
            | Opcode::Throw => -1,

            // Special cases
            Opcode::ReturnNull
            | Opcode::Jump(_)
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

            // Object operations
            Opcode::NewObject(_) => 1,
            Opcode::LoadProperty(_) => 0,   // pops object, pushes value
            Opcode::StoreProperty(_) => -1, // pops object and value, pushes object
            Opcode::StoreThisProperty(_) => 0, // pops value, modifies $this in slot 0, pushes value back
            Opcode::StoreCloneProperty(_) => -1, // pops object and value, pushes modified object
            Opcode::UnsetProperty(_) => -1,    // pops object
            Opcode::UnsetVar(_) => 0,          // no stack effect
            Opcode::UnsetArrayElement => -2,   // pops array and key
            Opcode::LoadStaticProp(_, _) => 1,
            Opcode::StoreStaticProp(_, _) => -1,
            Opcode::LoadEnumCase(_, _) => 1, // pushes enum case value

            // Array
            Opcode::NewArray(n) => 1 - (*n as i32) * 2, // pops n key-value pairs, pushes array
            Opcode::ArrayAppend => -1,                  // pops array and value, pushes array
            Opcode::ArrayUnpack => 0,                   // varies at runtime
            Opcode::ArrayMerge => -1,                   // pops two arrays, pushes merged array

            // Null coalescing jumps
            Opcode::JumpIfNull(_) | Opcode::JumpIfNotNull(_) => 0, // doesn't pop

            // Print returns 1
            Opcode::Print => 0, // pops 1, pushes 1

            // Ternary
            Opcode::Ternary => 0,

            // Match
            Opcode::MatchStart(_) => 0,
            Opcode::MatchArm => -1,
            Opcode::MatchDefault => 0,

            // Exception handling
            Opcode::Catch(_, _) => 1, // pushes caught exception
            Opcode::FinallyStart | Opcode::FinallyEnd => 0,

            // Closures
            Opcode::CreateClosure(_, n) => 1 - (*n as i32), // pops captured vars, pushes closure
            Opcode::CaptureVar(_) => 0,
        }
    }
}

/// Compiled function representation
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    /// Function name
    pub name: String,
    /// Bytecode instructions
    pub bytecode: Vec<Opcode>,
    /// Constant pool
    pub constants: Vec<Constant>,
    /// String pool (for variable names, property names, etc.)
    pub strings: Vec<String>,
    /// Number of local variable slots
    pub local_count: u16,
    /// Local variable names (for debugging)
    pub local_names: Vec<String>,
    /// Parameter count
    pub param_count: u8,
    /// Required parameter count (parameters without defaults)
    pub required_param_count: u8,
    /// Is variadic
    pub is_variadic: bool,
    /// Return type (for validation)
    pub return_type: Option<crate::ast::TypeHint>,
    /// Parameter types for validation
    pub param_types: Vec<Option<crate::ast::TypeHint>>,
    /// Function parameters (for reflection)
    pub parameters: Vec<crate::ast::FunctionParam>,
    /// Function attributes (for reflection)
    pub attributes: Vec<crate::ast::Attribute>,
    /// Whether strict_types=1 was enabled when this function was compiled
    pub strict_types: bool,
}

impl CompiledFunction {
    /// Create a new empty compiled function
    pub fn new(name: String) -> Self {
        Self {
            name,
            bytecode: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            local_count: 0,
            local_names: Vec::new(),
            param_count: 0,
            required_param_count: 0,
            is_variadic: false,
            return_type: None,
            param_types: Vec::new(),
            parameters: Vec::new(),
            attributes: Vec::new(),
            strict_types: false,
        }
    }
}

/// Constant value in the constant pool
#[derive(Debug, Clone)]
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

/// Compiled function representation
#[derive(Debug, Clone)]
pub struct CompiledFunction {
    /// Function name
    pub name: String,
    /// Bytecode instructions
    pub bytecode: Vec<super::Opcode>,
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
    /// Is generator function (contains yield)
    pub is_generator: bool,
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
            is_generator: false,
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
#[allow(dead_code)] // All variants defined for future use
pub enum Constant {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

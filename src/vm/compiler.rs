//! Compiler module - converts AST to bytecode
//!
//! This module compiles PHP AST into bytecode for the VM to execute.

mod compiler_types;
mod definitions;
mod expr;
mod expr_helpers;
mod functions;
mod if_match;
mod loops;
mod stmt;
mod try_catch;

use crate::ast::{BinaryOp, Expr, FunctionParam, Method, Program, Stmt, UnaryOp};
use crate::vm::class::{CompiledClass, CompiledEnum, CompiledInterface, CompiledTrait};
use crate::vm::opcode::{CompiledFunction, Opcode};
use std::collections::HashMap;
use std::sync::Arc;

/// Result of compiling a program
pub struct CompilationResult {
    /// Main function bytecode
    pub main: Arc<CompiledFunction>,
    /// User-defined functions (name -> compiled function)
    pub functions: HashMap<String, Arc<CompiledFunction>>,
    /// Class definitions
    pub classes: HashMap<String, Arc<CompiledClass>>,
    /// Interface definitions
    pub interfaces: HashMap<String, Arc<CompiledInterface>>,
    /// Trait definitions
    pub traits: HashMap<String, Arc<CompiledTrait>>,
    /// Enum definitions
    pub enums: HashMap<String, Arc<CompiledEnum>>,
}

/// Compiler state for generating bytecode
#[allow(dead_code)] // break_targets and continue_targets fields not yet used
pub struct Compiler {
    /// Current function being compiled
    function: CompiledFunction,
    /// String interning table for deduplication
    string_table: HashMap<String, u32>,
    /// Local variable mapping (name -> slot)
    locals: HashMap<String, u16>,
    /// Next available local slot
    next_local: u16,
    /// Break target stack (for break statements)
    break_targets: Vec<usize>,
    /// Continue target stack (for continue statements)
    continue_targets: Vec<usize>,
    /// Compiled functions collected during compilation
    functions: HashMap<String, Arc<CompiledFunction>>,
    /// Compiled classes collected during compilation
    classes: HashMap<String, Arc<CompiledClass>>,
    /// Compiled interfaces collected during compilation
    interfaces: HashMap<String, Arc<CompiledInterface>>,
    /// Compiled traits collected during compilation
    traits: HashMap<String, Arc<CompiledTrait>>,
    /// Compiled enums collected during compilation
    enums: HashMap<String, Arc<CompiledEnum>>,
    /// Whether strict_types=1 is active for this compilation unit
    strict_types: bool,
    /// Current namespace (for prefixing class/function names)
    current_namespace: Option<String>,
    /// Use aliases: short name -> fully qualified name
    use_aliases: HashMap<String, String>,
}

impl Compiler {
    /// Create a new compiler for a function
    pub fn new(name: String) -> Self {
        Self {
            function: CompiledFunction::new(name),
            string_table: HashMap::new(),
            locals: HashMap::new(),
            next_local: 0,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            interfaces: HashMap::new(),
            traits: HashMap::new(),
            enums: HashMap::new(),
            strict_types: false,
            current_namespace: None,
            use_aliases: HashMap::new(),
        }
    }

    /// Compile a program to main function and all user-defined functions
    pub fn compile_program(mut self, program: &Program) -> Result<CompilationResult, String> {
        // Compile all statements
        for stmt in &program.statements {
            self.compile_stmt(stmt)?;
        }

        // Add implicit return null at end
        self.emit(Opcode::ReturnNull);

        Ok(CompilationResult {
            main: Arc::new(self.function),
            functions: self.functions,
            classes: self.classes,
            interfaces: self.interfaces,
            traits: self.traits,
            enums: self.enums,
        })
    }

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: &crate::ast::Stmt) -> Result<(), String> {
        self.compile_stmt_internal(stmt)
    }

    /// Compile an if statement with elseif and else branches
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        elseif_branches: &[(Expr, Vec<Stmt>)],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        self.compile_if_internal(condition, then_branch, elseif_branches, else_branch)
    }

    fn compile_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), String> {
        self.compile_while_internal(condition, body)
    }

    fn compile_do_while(&mut self, body: &[Stmt], condition: &Expr) -> Result<(), String> {
        self.compile_do_while_internal(body, condition)
    }

    fn compile_for(
        &mut self,
        init: &Option<Expr>,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
    ) -> Result<(), String> {
        self.compile_for_internal(init, condition, update, body)
    }

    fn compile_foreach(
        &mut self,
        array: &Expr,
        key: &Option<String>,
        value: &str,
        body: &[Stmt],
    ) -> Result<(), String> {
        self.compile_foreach_internal(array, key, value, body)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        self.compile_expr_internal(expr)
    }

    /// Compile a binary operation
    fn compile_binary_op(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<(), String> {
        self.compile_binary_op_internal(left, op, right)
    }

    /// Compile a unary operation
    fn compile_unary_op(&mut self, op: &UnaryOp, operand: &Expr) -> Result<(), String> {
        self.compile_unary_op_internal(op, operand)
    }

    /// Compile a ternary expression
    fn compile_ternary(
        &mut self,
        condition: &Expr,
        then_val: &Expr,
        else_val: &Expr,
    ) -> Result<(), String> {
        self.compile_ternary_internal(condition, then_val, else_val)
    }

    /// Compile a match expression
    fn compile_match(
        &mut self,
        expr: &Expr,
        arms: &[crate::ast::MatchArm],
        default: &Option<Box<Expr>>,
    ) -> Result<(), String> {
        self.compile_match_internal(expr, arms, default)
    }

    /// Compile an arrow function (fn($x) => expr)
    fn compile_arrow_function(
        &mut self,
        params: &[FunctionParam],
        body: &Expr,
    ) -> Result<(), String> {
        self.compile_arrow_function_internal(params, body)
    }

    /// Emit an opcode
    fn emit(&mut self, opcode: Opcode) -> usize {
        let offset = self.function.bytecode.len();
        self.function.bytecode.push(opcode);
        offset
    }

    /// Emit a jump instruction and return its offset for patching
    fn emit_jump(&mut self, opcode: Opcode) -> usize {
        self.emit(opcode)
    }

    /// Emit a loop jump (backward jump to loop start)
    fn emit_loop(&mut self, loop_start: usize) {
        self.emit(Opcode::Jump(loop_start as u32));
    }

    /// Patch a jump instruction to jump to the current position
    fn patch_jump(&mut self, offset: usize) {
        let target = self.function.bytecode.len() as u32;
        match &mut self.function.bytecode[offset] {
            Opcode::Jump(ref mut target_ref) => *target_ref = target,
            Opcode::JumpIfFalse(ref mut target_ref) => *target_ref = target,
            Opcode::JumpIfTrue(ref mut target_ref) => *target_ref = target,
            Opcode::JumpIfNull(ref mut target_ref) => *target_ref = target,
            Opcode::JumpIfNotNull(ref mut target_ref) => *target_ref = target,
            _ => panic!("Attempted to patch non-jump instruction"),
        }
    }

    /// Get current bytecode offset
    fn current_offset(&self) -> usize {
        self.function.bytecode.len()
    }

    /// Intern a string and return its index
    fn intern_string(&mut self, s: String) -> u32 {
        if let Some(&idx) = self.string_table.get(&s) {
            return idx;
        }

        let idx = self.function.strings.len() as u32;
        self.function.strings.push(s.clone());
        self.string_table.insert(s, idx);
        idx
    }

    /// Allocate a local variable slot
    fn allocate_local(&mut self, name: String) -> u16 {
        if let Some(&slot) = self.locals.get(&name) {
            return slot;
        }

        let slot = self.next_local;
        self.locals.insert(name.clone(), slot);
        self.next_local += 1;
        self.function.local_count = self.next_local;
        self.function.local_names.push(name);
        slot
    }

    /// Compile a function definition
    fn compile_function(
        &mut self,
        name: &str,
        params: &[FunctionParam],
        return_type: &Option<crate::ast::TypeHint>,
        body: &[Stmt],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        self.compile_function_internal(name, params, return_type, body, attributes)
    }

    fn compile_switch(
        &mut self,
        expr: &Expr,
        cases: &[crate::ast::SwitchCase],
        default: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        self.compile_switch_internal(expr, cases, default)
    }

    fn compile_try_catch(
        &mut self,
        try_body: &[Stmt],
        catch_clauses: &[crate::ast::CatchClause],
        finally_body: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        self.compile_try_catch_internal(try_body, catch_clauses, finally_body)
    }

    /// Compile a class definition
    #[allow(clippy::too_many_arguments)]
    fn compile_class(
        &mut self,
        name: &str,
        is_abstract: bool,
        is_final: bool,
        readonly: bool,
        parent: &Option<crate::ast::QualifiedName>,
        interfaces: &[crate::ast::QualifiedName],
        trait_uses: &[crate::ast::TraitUse],
        properties: &[crate::ast::Property],
        methods: &[Method],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        self.compile_class_internal(
            name,
            is_abstract,
            is_final,
            readonly,
            parent,
            interfaces,
            trait_uses,
            properties,
            methods,
            attributes,
        )
    }

    fn compile_interface(
        &mut self,
        name: &str,
        parents: &[crate::ast::QualifiedName],
        methods: &[crate::ast::InterfaceMethodSignature],
        constants: &[crate::ast::InterfaceConstant],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        self.compile_interface_internal(name, parents, methods, constants, attributes)
    }

    fn compile_trait(
        &mut self,
        name: &str,
        uses: &[String],
        properties: &[crate::ast::Property],
        methods: &[Method],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        self.compile_trait_internal(name, uses, properties, methods, attributes)
    }

    fn compile_enum(
        &mut self,
        name: &str,
        backing_type: &crate::ast::EnumBackingType,
        cases: &[crate::ast::EnumCase],
        methods: &[Method],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        self.compile_enum_internal(name, backing_type, cases, methods, attributes)
    }
}

//! Compiler module - converts AST to bytecode
//!
//! This module compiles PHP AST into bytecode for the VM to execute.

use crate::ast::{BinaryOp, Expr, FunctionParam, Method, Program, Stmt, UnaryOp};
use crate::vm::class::{
    CompiledClass, CompiledEnum, CompiledInterface, CompiledProperty, CompiledTrait,
};
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
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Echo(exprs) => {
                for expr in exprs {
                    self.compile_expr(expr)?;
                    self.emit(Opcode::Echo);
                }
            }
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.emit(Opcode::Pop); // Discard result
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr)?;
                    self.emit(Opcode::Return);
                } else {
                    self.emit(Opcode::ReturnNull);
                }
            }
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
                self.compile_if(condition, then_branch, elseif_branches, else_branch)?;
            }
            Stmt::While { condition, body } => {
                self.compile_while(condition, body)?;
            }
            Stmt::DoWhile { body, condition } => {
                self.compile_do_while(body, condition)?;
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                self.compile_for(init, condition, update, body)?;
            }
            Stmt::Foreach {
                array,
                key,
                value,
                body,
            } => {
                self.compile_foreach(array, key, value, body)?;
            }
            Stmt::Break => {
                self.emit(Opcode::Break);
            }
            Stmt::Continue => {
                self.emit(Opcode::Continue);
            }
            Stmt::Function {
                name,
                params,
                return_type,
                body,
                attributes,
            } => {
                self.compile_function(name, params, return_type, body, attributes)?;
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                self.compile_switch(expr, cases, default)?;
            }
            Stmt::Html(content) => {
                // Output raw HTML content
                let idx = self.intern_string(content.clone());
                self.emit(Opcode::PushString(idx));
                self.emit(Opcode::Echo);
            }
            Stmt::Declare { directives, body } => {
                // Handle declare directives
                for directive in directives {
                    match directive {
                        crate::ast::DeclareDirective::StrictTypes(enabled) => {
                            self.strict_types = *enabled;
                        }
                        _ => {
                            // Other directives like encoding are ignored for now
                        }
                    }
                }
                // Compile the body if present
                if let Some(stmts) = body {
                    for stmt in stmts {
                        self.compile_stmt(stmt)?;
                    }
                }
            }
            Stmt::Namespace { name, body } => {
                // Save the previous namespace and use aliases
                let prev_namespace = self.current_namespace.clone();
                let prev_use_aliases = self.use_aliases.clone();

                // Set current namespace (using the full qualified name)
                self.current_namespace = name.as_ref().map(|n| n.parts.join("\\"));

                // Clear use aliases when entering new namespace
                self.use_aliases.clear();

                // Compile namespace body
                match body {
                    crate::ast::NamespaceBody::Braced(stmts) => {
                        for stmt in stmts {
                            self.compile_stmt(stmt)?;
                        }
                        // Restore previous namespace and use aliases after braced block
                        self.current_namespace = prev_namespace;
                        self.use_aliases = prev_use_aliases;
                    }
                    crate::ast::NamespaceBody::Unbraced => {
                        // Rest of file is in namespace - don't restore
                        // But keep prev_use_aliases cleared
                    }
                }
            }
            Stmt::Use(use_clauses) => {
                // Register use aliases
                for clause in use_clauses {
                    let full_name = clause.name.parts.join("\\");
                    let alias = clause.alias.clone().unwrap_or_else(|| {
                        // Use last part of name as alias
                        clause.name.last().cloned().unwrap_or_default()
                    });
                    self.use_aliases.insert(alias, full_name);
                }
            }
            Stmt::GroupUse(group_use) => {
                // Register group use aliases (e.g., use Foo\{Bar, Baz})
                let prefix = group_use.prefix.parts.join("\\");
                for clause in &group_use.items {
                    let full_name = if prefix.is_empty() {
                        clause.name.parts.join("\\")
                    } else {
                        format!("{}\\{}", prefix, clause.name.parts.join("\\"))
                    };
                    let alias = clause.alias.clone().unwrap_or_else(|| {
                        // Use last part of name as alias
                        clause.name.last().cloned().unwrap_or_default()
                    });
                    self.use_aliases.insert(alias, full_name);
                }
            }
            Stmt::Throw(expr) => {
                self.compile_expr(expr)?;
                self.emit(Opcode::Throw);
            }
            Stmt::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
            } => {
                self.compile_try_catch(try_body, catch_clauses, finally_body)?;
            }
            Stmt::Class {
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
            } => {
                self.compile_class(
                    name,
                    *is_abstract,
                    *is_final,
                    *readonly,
                    parent,
                    interfaces,
                    trait_uses,
                    properties,
                    methods,
                    attributes,
                )?;
            }
            Stmt::Interface {
                name,
                parents,
                methods,
                constants,
                attributes,
            } => {
                self.compile_interface(name, parents, methods, constants, attributes)?;
            }
            Stmt::Trait {
                name,
                uses,
                properties,
                methods,
                attributes,
            } => {
                self.compile_trait(name, uses, properties, methods, attributes)?;
            }
            Stmt::Enum {
                name,
                backing_type,
                cases,
                methods,
                attributes,
            } => {
                self.compile_enum(name, backing_type, cases, methods, attributes)?;
            }
        }
        Ok(())
    }

    /// Compile an if statement with elseif and else branches
    fn compile_if(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        elseif_branches: &[(Expr, Vec<Stmt>)],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        // Compile condition
        self.compile_expr(condition)?;

        // Jump to first elseif/else if condition is false
        let then_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        // Compile then branch
        for stmt in then_branch {
            self.compile_stmt(stmt)?;
        }

        // Jump past all elseif/else branches
        let end_jump = self.emit_jump(Opcode::Jump(0));

        // Patch then_jump to point here
        self.patch_jump(then_jump);

        // Compile elseif branches
        let mut elseif_jumps = Vec::new();
        for (elseif_condition, elseif_body) in elseif_branches {
            self.compile_expr(elseif_condition)?;
            let elseif_jump = self.emit_jump(Opcode::JumpIfFalse(0));

            for stmt in elseif_body {
                self.compile_stmt(stmt)?;
            }

            elseif_jumps.push(self.emit_jump(Opcode::Jump(0)));
            self.patch_jump(elseif_jump);
        }

        // Compile else branch if present
        if let Some(else_stmts) = else_branch {
            for stmt in else_stmts {
                self.compile_stmt(stmt)?;
            }
        }

        // Patch all end jumps
        self.patch_jump(end_jump);
        for jump in elseif_jumps {
            self.patch_jump(jump);
        }

        Ok(())
    }

    /// Compile a while loop
    fn compile_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), String> {
        let loop_start = self.current_offset();

        // Compile condition
        self.compile_expr(condition)?;

        // Jump past loop if false
        let exit_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        // Emit LoopStart with placeholders for continue/break targets
        // continue_target = loop_start, break_target will be patched
        let loop_start_idx = self.emit(Opcode::LoopStart(loop_start as u32, 0));

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // LoopEnd
        self.emit(Opcode::LoopEnd);

        // Jump back to loop start
        self.emit_loop(loop_start);

        // Patch exit jump to here (after loop)
        let loop_end = self.current_offset();
        self.patch_jump(exit_jump);

        // Patch LoopStart's break target
        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    /// Compile a do-while loop
    fn compile_do_while(&mut self, body: &[Stmt], condition: &Expr) -> Result<(), String> {
        let loop_start = self.current_offset();

        // Emit LoopStart - break target will be patched
        let loop_start_idx = self.emit(Opcode::LoopStart(loop_start as u32, 0));

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // LoopEnd before condition check
        self.emit(Opcode::LoopEnd);

        // Compile condition
        self.compile_expr(condition)?;

        // Jump back to loop start if true
        self.emit(Opcode::JumpIfTrue(loop_start as u32));

        // Patch break target to current position
        let loop_end = self.current_offset();
        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    /// Compile a for loop
    fn compile_for(
        &mut self,
        init: &Option<Expr>,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
    ) -> Result<(), String> {
        // Compile init
        if let Some(init_expr) = init {
            self.compile_expr(init_expr)?;
            self.emit(Opcode::Pop); // Discard result
        }

        let loop_start = self.current_offset();

        // Compile condition (default to true if not present)
        let exit_jump = if let Some(cond_expr) = condition {
            self.compile_expr(cond_expr)?;
            Some(self.emit_jump(Opcode::JumpIfFalse(0)))
        } else {
            None
        };

        // Emit LoopStart with placeholder for continue_target (will be patched to update)
        // continue_target will be patched to point to the update expression
        let loop_start_idx = self.emit(Opcode::LoopStart(0, 0));

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // LoopEnd before update
        self.emit(Opcode::LoopEnd);

        // Record position of update for continue
        let update_offset = self.current_offset();

        // Compile update
        if let Some(update_expr) = update {
            self.compile_expr(update_expr)?;
            self.emit(Opcode::Pop); // Discard result
        }

        // Jump back to loop start (condition)
        self.emit_loop(loop_start);

        // Patch exit jump and get end position
        let loop_end = self.current_offset();
        if let Some(exit) = exit_jump {
            self.patch_jump(exit);
        }

        // Patch LoopStart's continue_target (to update) and break_target (to end)
        if let Opcode::LoopStart(ref mut continue_target, ref mut break_target) =
            self.function.bytecode[loop_start_idx]
        {
            *continue_target = update_offset as u32;
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    /// Compile a foreach loop
    fn compile_foreach(
        &mut self,
        array: &Expr,
        key: &Option<String>,
        value: &str,
        body: &[Stmt],
    ) -> Result<(), String> {
        // Compile the array expression and store in a local slot to avoid re-evaluation
        self.compile_expr(array)?;
        let array_slot = self.allocate_local("__foreach_array__".to_string());
        self.emit(Opcode::StoreFast(array_slot));

        // Create a local slot for the iterator index
        let iter_slot = self.allocate_local("__foreach_iter__".to_string());

        // Initialize iterator to 0
        self.emit(Opcode::PushInt(0));
        self.emit(Opcode::StoreFast(iter_slot));

        // Create local slots for key and value
        let key_slot = key
            .as_ref()
            .map(|key_name| self.allocate_local(key_name.clone()));
        let value_slot = self.allocate_local(value.to_string());

        // Loop start - where continue jumps to (increment and check)
        let loop_check = self.current_offset();

        // Load array and get length
        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::ArrayCount);

        // Load iterator
        self.emit(Opcode::LoadFast(iter_slot));

        // Compare: if iter >= length, exit
        self.emit(Opcode::Le); // length <= iter means exit (reversed operands)
        let exit_jump = self.emit_jump(Opcode::JumpIfTrue(0));

        // Get key at current index
        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::ArrayGetKeyAt);

        // Store key if needed, otherwise discard
        if let Some(slot) = key_slot {
            self.emit(Opcode::StoreFast(slot));
        } else {
            self.emit(Opcode::Pop);
        }

        // Get value at current index
        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::ArrayGetValueAt);

        // Store value
        self.emit(Opcode::StoreFast(value_slot));

        // Emit LoopStart - continue goes to increment, break exits
        let loop_start_idx = self.emit(Opcode::LoopStart(loop_check as u32, 0));

        // Compile body
        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // LoopEnd before increment
        self.emit(Opcode::LoopEnd);

        // Increment iterator
        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::PushInt(1));
        self.emit(Opcode::Add);
        self.emit(Opcode::StoreFast(iter_slot));

        // Jump back to condition check
        self.emit_loop(loop_check);

        // Patch exit jump to here
        let loop_end = self.current_offset();
        self.patch_jump(exit_jump);

        // Patch LoopStart's break target
        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    /// Compile an expression
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Null => {
                self.emit(Opcode::PushNull);
            }
            Expr::Bool(b) => {
                if *b {
                    self.emit(Opcode::PushTrue);
                } else {
                    self.emit(Opcode::PushFalse);
                }
            }
            Expr::Integer(n) => {
                self.emit(Opcode::PushInt(*n));
            }
            Expr::Float(f) => {
                self.emit(Opcode::PushFloat(*f));
            }
            Expr::String(s) => {
                let idx = self.intern_string(s.clone());
                self.emit(Opcode::PushString(idx));
            }
            Expr::Variable(name) => {
                // Check if it's a local variable
                if let Some(&slot) = self.locals.get(name) {
                    self.emit(Opcode::LoadFast(slot));
                } else {
                    let idx = self.intern_string(name.clone());
                    self.emit(Opcode::LoadVar(idx));
                }
            }
            Expr::Assign { var, op, value } => {
                use crate::ast::AssignOp;

                // For compound assignment, need to load current value first
                if *op != AssignOp::Assign {
                    if let Some(&slot) = self.locals.get(var) {
                        self.emit(Opcode::LoadFast(slot));
                    } else {
                        let idx = self.intern_string(var.clone());
                        self.emit(Opcode::LoadVar(idx));
                    }
                }

                self.compile_expr(value)?;

                // Apply compound operation if needed
                match op {
                    AssignOp::Assign => {}
                    AssignOp::AddAssign => {
                        self.emit(Opcode::Add);
                    }
                    AssignOp::SubAssign => {
                        self.emit(Opcode::Sub);
                    }
                    AssignOp::MulAssign => {
                        self.emit(Opcode::Mul);
                    }
                    AssignOp::DivAssign => {
                        self.emit(Opcode::Div);
                    }
                    AssignOp::ModAssign => {
                        self.emit(Opcode::Mod);
                    }
                    AssignOp::ConcatAssign => {
                        self.emit(Opcode::Concat);
                    }
                };

                // Check if we need to create a local slot
                if !self.locals.contains_key(var) {
                    let slot = self.next_local;
                    self.locals.insert(var.clone(), slot);
                    self.next_local += 1;
                    self.function.local_count = self.next_local;
                    self.function.local_names.push(var.clone());
                }

                if let Some(&slot) = self.locals.get(var) {
                    self.emit(Opcode::Dup); // Keep value on stack for assignment expression
                    self.emit(Opcode::StoreFast(slot));
                } else {
                    let idx = self.intern_string(var.clone());
                    self.emit(Opcode::Dup);
                    self.emit(Opcode::StoreVar(idx));
                }
            }
            Expr::Binary { left, op, right } => {
                self.compile_binary_op(left, op, right)?;
            }
            Expr::Unary { op, expr } => {
                self.compile_unary_op(op, expr)?;
            }
            Expr::Array(elements) => {
                // Compile all key-value pairs
                let count = elements.len();
                for elem in elements {
                    // Use auto-indexing if no key provided
                    if let Some(key_expr) = &elem.key {
                        self.compile_expr(key_expr)?;
                    } else {
                        // Auto-generate integer key
                        let idx = elements.iter().position(|e| std::ptr::eq(e, elem)).unwrap();
                        self.emit(Opcode::PushInt(idx as i64));
                    }
                    self.compile_expr(&elem.value)?;
                }
                self.emit(Opcode::NewArray(count as u16));
            }
            Expr::ArrayAccess { array, index } => {
                self.compile_expr(array)?;
                self.compile_expr(index)?;
                self.emit(Opcode::ArrayGet);
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.compile_ternary(condition, then_expr, else_expr)?;
            }
            Expr::FunctionCall { name, args } => {
                // Special handling for unset() - it's a language construct
                if name.to_lowercase() == "unset" {
                    // unset() can take multiple arguments
                    for arg in args {
                        match arg.value.as_ref() {
                            Expr::PropertyAccess { object, property } => {
                                // unset($obj->prop) - should call __unset if property doesn't exist
                                let prop_idx = self.intern_string(property.clone());

                                // Check if object is a variable to track source
                                if let Expr::Variable(var_name) = object.as_ref() {
                                    if let Some(&slot) = self.locals.get(var_name) {
                                        // Local variable - use tracking opcode
                                        self.emit(Opcode::UnsetPropertyOnLocal(slot, prop_idx));
                                    } else {
                                        // Global variable - use tracking opcode
                                        let var_idx = self.intern_string(var_name.clone());
                                        self.emit(Opcode::UnsetPropertyOnGlobal(var_idx, prop_idx));
                                    }
                                } else {
                                    // Complex expression - use regular UnsetProperty
                                    self.compile_expr(object)?;
                                    self.emit(Opcode::UnsetProperty(prop_idx));
                                }
                            }
                            Expr::Variable(var_name) => {
                                // unset($var) - remove variable from scope
                                if let Some(&slot) = self.locals.get(var_name) {
                                    // Local variable - set to null (PHP doesn't truly remove locals)
                                    self.emit(Opcode::PushNull);
                                    self.emit(Opcode::StoreFast(slot));
                                } else {
                                    // Global variable
                                    let idx = self.intern_string(var_name.clone());
                                    self.emit(Opcode::UnsetVar(idx));
                                }
                            }
                            Expr::ArrayAccess { array, index } => {
                                // unset($arr[$key]) - remove array element
                                self.compile_expr(array)?;
                                self.compile_expr(index)?;
                                self.emit(Opcode::UnsetArrayElement);
                            }
                            _ => {
                                return Err(format!("Cannot unset expression: {:?}", arg.value));
                            }
                        }
                    }
                    // unset() doesn't return a value, push null
                    self.emit(Opcode::PushNull);
                    return Ok(());
                }

                // Special handling for isset() - it's a language construct
                // isset($obj->prop) should call __isset magic method
                if name.to_lowercase() == "isset" && args.len() == 1 {
                    if let Expr::PropertyAccess { object, property } = args[0].value.as_ref() {
                        let prop_idx = self.intern_string(property.clone());

                        // Check if object is a variable to track source
                        if let Expr::Variable(var_name) = object.as_ref() {
                            if let Some(&slot) = self.locals.get(var_name) {
                                // Local variable - use tracking opcode
                                self.emit(Opcode::IssetPropertyOnLocal(slot, prop_idx));
                            } else {
                                // Global variable - use tracking opcode
                                let var_idx = self.intern_string(var_name.clone());
                                self.emit(Opcode::IssetPropertyOnGlobal(var_idx, prop_idx));
                            }
                        } else {
                            // Complex expression - use regular IssetProperty
                            self.compile_expr(object)?;
                            self.emit(Opcode::IssetProperty(prop_idx));
                        }
                        return Ok(());
                    }
                    // For other cases (variables, array access), use normal isset builtin
                }

                // Check what kind of arguments we have
                let has_spread = args
                    .iter()
                    .any(|arg| matches!(arg.value.as_ref(), Expr::Spread(_)));
                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_spread {
                    // Compile with spread operator support
                    // Strategy: Build an array with all arguments, then pass to CallSpread
                    // Start with empty array
                    self.emit(Opcode::NewArray(0));

                    for arg in args {
                        match arg.value.as_ref() {
                            Expr::Spread(inner) => {
                                // Compile the spread array
                                self.compile_expr(inner)?;
                                // Merge it into the argument array
                                self.emit(Opcode::ArrayMerge);
                            }
                            _ => {
                                // Regular argument - append to array
                                self.compile_expr(&arg.value)?;
                                self.emit(Opcode::ArrayAppend);
                            }
                        }
                    }

                    // Now we have an array with all arguments
                    // Emit CallSpread opcode with function name index
                    let name_idx = self.intern_string(name.clone());
                    self.emit(Opcode::CallSpread(name_idx));
                } else if has_named {
                    // Compile with named arguments support
                    // Build an array with positional args followed by named args
                    // Use array with integer and string keys

                    let mut total_pairs = 0;

                    // First, push all key-value pairs (positional as integers, named as strings)
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            // Named argument
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            // Positional argument - use integer index
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                        total_pairs += 1;
                    }

                    // Create array from pairs
                    self.emit(Opcode::NewArray(total_pairs as u16));

                    // Emit CallNamed opcode
                    let name_idx = self.intern_string(name.clone());
                    self.emit(Opcode::CallNamed(name_idx));
                } else {
                    // Compile arguments in order (no spread, no named)
                    for arg in args {
                        self.compile_expr(&arg.value)?;
                    }
                    // Emit call opcode with function name index and arg count
                    let name_idx = self.intern_string(name.clone());
                    self.emit(Opcode::Call(name_idx, args.len() as u8));
                }
            }
            Expr::ArrayAssign {
                array,
                index,
                op,
                value,
            } => {
                // Array assignment: $arr[$key] = $value or $obj->prop[$key] = $value
                // This is complex because we need to update the array in place
                use crate::ast::AssignOp;

                // Handle compound assignment (+=, -= etc)
                if *op != AssignOp::Assign {
                    // Need to get current value, apply operation, then store
                    return Err("Compound array assignment not yet implemented".to_string());
                }

                // Handle different base types
                match array.as_ref() {
                    Expr::Variable(var_name) => {
                        // Simple variable: $arr[$key] = $value
                        // Load the current array
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::LoadFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::LoadVar(idx));
                        }

                        // Compile the key (or null for append)
                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        // Compile the value
                        self.compile_expr(value)?;

                        // Set or append to array
                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            // For append ($arr[] = value):
                            // Stack is: array, null, value
                            // We need: array, value for ArrayAppend
                            self.emit(Opcode::Swap); // array, value, null
                            self.emit(Opcode::Pop); // array, value
                            self.emit(Opcode::ArrayAppend);
                        }

                        // Store the updated array back
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::Dup); // Keep result on stack
                            self.emit(Opcode::StoreFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::Dup);
                            self.emit(Opcode::StoreVar(idx));
                        }
                    }
                    Expr::PropertyAccess { object, property } => {
                        // Property array access: $obj->prop[$key] = $value
                        // We need to: load prop, set array element, store prop back

                        // Check if this is $this->prop[$key] = value
                        let is_this = matches!(object.as_ref(), Expr::This);

                        // Load the property (which is an array)
                        self.compile_expr(object)?;
                        let prop_idx = self.intern_string(property.clone());
                        self.emit(Opcode::LoadProperty(prop_idx));

                        // Compile the key (or null for append)
                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        // Compile the value
                        self.compile_expr(value)?;

                        // Set in array
                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            // Stack: array, null, value -> need array, value
                            self.emit(Opcode::Swap); // array, value, null
                            self.emit(Opcode::Pop); // array, value
                            self.emit(Opcode::ArrayAppend);
                        }

                        // Now we have the modified array on stack
                        // We need to store it back to the property
                        if is_this {
                            // For $this->prop[$key] = value, use StoreThisProperty
                            // which updates slot 0 and pushes the value back
                            self.emit(Opcode::StoreThisProperty(prop_idx));
                        } else {
                            // Load object again, swap with array, then store property
                            self.compile_expr(object)?;
                            self.emit(Opcode::Swap); // array, object -> object, array
                            self.emit(Opcode::StoreProperty(prop_idx));

                            // For variables, also store back to the variable if it's a simple var
                            if let Expr::Variable(var_name) = object.as_ref() {
                                if let Some(&slot) = self.locals.get(var_name) {
                                    self.emit(Opcode::StoreFast(slot));
                                } else {
                                    let idx = self.intern_string(var_name.clone());
                                    self.emit(Opcode::StoreVar(idx));
                                }
                            }
                        }
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        // Static property array access: Class::$prop[$key] = $value
                        // Load the static property (which should be an array)
                        let class_idx = self.intern_string(class.clone());
                        let prop_idx = self.intern_string(property.clone());
                        self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                        // Compile the key (or null for append)
                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        // Compile the value
                        self.compile_expr(value)?;

                        // Set in array
                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            // Stack: array, null, value -> need array, value
                            self.emit(Opcode::Swap); // array, value, null
                            self.emit(Opcode::Pop); // array, value
                            self.emit(Opcode::ArrayAppend);
                        }

                        // Store the modified array back to the static property
                        self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                    }
                    _ => return Err("Complex array assignment not yet implemented".to_string()),
                }
            }
            Expr::Grouped(inner) => {
                self.compile_expr(inner)?;
            }
            // OOP Expressions
            Expr::New { class_name, args } => {
                // Qualify the class name with current namespace
                let qualified_name = self.qualify_class_name(class_name);
                // Emit new object opcode first (creates object with defaults)
                let class_idx = self.intern_string(qualified_name);
                self.emit(Opcode::NewObject(class_idx));

                // Check if we have named arguments
                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_named {
                    // Build associative array with positional and named args
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            // Named argument
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            // Positional argument - use integer index
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                    }
                    // Create array from pairs
                    self.emit(Opcode::NewArray(args.len() as u16));
                    // Emit CallConstructorNamed opcode
                    self.emit(Opcode::CallConstructorNamed);
                } else {
                    // Regular positional arguments
                    for arg in args {
                        self.compile_expr(&arg.value)?;
                    }
                    // Emit constructor call
                    self.emit(Opcode::CallConstructor(args.len() as u8));
                }
            }
            Expr::PropertyAccess { object, property } => {
                self.compile_expr(object)?;
                let prop_idx = self.intern_string(property.clone());
                self.emit(Opcode::LoadProperty(prop_idx));
            }
            Expr::PropertyAssign {
                object,
                property,
                value,
            } => {
                // Check if we're assigning to $this->property
                if matches!(object.as_ref(), Expr::This) {
                    // Use optimized StoreThisProperty that updates slot 0
                    self.compile_expr(value)?;
                    let prop_idx = self.intern_string(property.clone());
                    self.emit(Opcode::StoreThisProperty(prop_idx));
                } else if let Expr::Variable(var_name) = object.as_ref() {
                    // Assigning to a variable's property: $var->prop = value
                    // We need to: load var, store property, store back to var
                    self.compile_expr(object)?;
                    self.compile_expr(value)?;
                    let prop_idx = self.intern_string(property.clone());
                    self.emit(Opcode::StoreProperty(prop_idx));
                    // Store the modified object back to the variable
                    if let Some(&slot) = self.locals.get(var_name) {
                        self.emit(Opcode::StoreFast(slot));
                    } else {
                        let var_idx = self.intern_string(var_name.clone());
                        self.emit(Opcode::StoreVar(var_idx));
                    }
                } else {
                    self.compile_expr(object)?;
                    self.compile_expr(value)?;
                    let prop_idx = self.intern_string(property.clone());
                    self.emit(Opcode::StoreProperty(prop_idx));
                }
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let method_idx = self.intern_string(method.clone());

                // Check if we're calling on a variable - use special opcode to track source
                match object.as_ref() {
                    Expr::Variable(var_name) => {
                        // Compile arguments
                        for arg in args {
                            self.compile_expr(&arg.value)?;
                        }

                        if let Some(&slot) = self.locals.get(var_name) {
                            // Local variable - use CallMethodOnLocal
                            self.emit(Opcode::CallMethodOnLocal(
                                slot,
                                method_idx,
                                args.len() as u8,
                            ));
                        } else {
                            // Global variable - use CallMethodOnGlobal
                            let var_idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::CallMethodOnGlobal(
                                var_idx,
                                method_idx,
                                args.len() as u8,
                            ));
                        }
                    }
                    _ => {
                        // Non-variable object - compile object first, then args
                        self.compile_expr(object)?;
                        for arg in args {
                            self.compile_expr(&arg.value)?;
                        }
                        self.emit(Opcode::CallMethod(method_idx, args.len() as u8));
                    }
                }
            }
            Expr::StaticMethodCall {
                class_name,
                method,
                args,
            } => {
                // Check if we have named arguments
                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_named {
                    // Build associative array with positional and named args
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            // Named argument
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            // Positional argument - use integer index
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                    }
                    // Create array from pairs
                    self.emit(Opcode::NewArray(args.len() as u16));
                    let class_idx = self.intern_string(class_name.clone());
                    let method_idx = self.intern_string(method.clone());
                    self.emit(Opcode::CallStaticMethodNamed(class_idx, method_idx));
                } else {
                    // Regular positional arguments
                    for arg in args {
                        self.compile_expr(&arg.value)?;
                    }
                    let class_idx = self.intern_string(class_name.clone());
                    let method_idx = self.intern_string(method.clone());
                    self.emit(Opcode::CallStaticMethod(
                        class_idx,
                        method_idx,
                        args.len() as u8,
                    ));
                }
            }
            Expr::StaticPropertyAccess { class, property } => {
                let class_idx = self.intern_string(class.clone());
                let prop_idx = self.intern_string(property.clone());
                self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));
            }
            Expr::StaticPropertyAssign {
                class,
                property,
                value,
            } => {
                self.compile_expr(value)?;
                let class_idx = self.intern_string(class.clone());
                let prop_idx = self.intern_string(property.clone());
                self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
            }
            Expr::This => {
                self.emit(Opcode::LoadThis);
            }
            Expr::Clone { object } => {
                self.compile_expr(object)?;
                self.emit(Opcode::Clone);
            }
            Expr::Match {
                expr,
                arms,
                default,
            } => {
                self.compile_match(expr, arms, default)?;
            }
            Expr::EnumCase {
                enum_name,
                case_name,
            } => {
                // Load enum case as a proper enum value
                let enum_idx = self.intern_string(enum_name.clone());
                let case_idx = self.intern_string(case_name.clone());
                self.emit(Opcode::LoadEnumCase(enum_idx, case_idx));
            }
            Expr::ArrowFunction { params, body } => {
                self.compile_arrow_function(params, body)?;
            }
            Expr::Throw(inner) => {
                self.compile_expr(inner)?;
                self.emit(Opcode::Throw);
            }
            Expr::Yield { key, value } => {
                // Yield: yield $key => $value or yield $value
                if let Some(k) = key {
                    // Compile key
                    self.compile_expr(k)?;
                }
                if let Some(v) = value {
                    // Compile value
                    self.compile_expr(v)?;
                }
                self.emit(Opcode::Yield);
            }
            Expr::YieldFrom(inner) => {
                // yield from $iterable
                self.compile_expr(inner)?;
                self.emit(Opcode::YieldFrom);
            }
            Expr::Spread(inner) => {
                self.compile_expr(inner)?;
                self.emit(Opcode::ArrayUnpack);
            }
            Expr::CallableCall { callable, args } => {
                // Variable function call: $func() or expression()
                // First compile all args, then the callable, then emit CallCallable
                for arg in args {
                    self.compile_expr(&arg.value)?;
                }
                self.compile_expr(callable)?;
                self.emit(Opcode::CallCallable(args.len() as u8));
            }
            Expr::CallableFromFunction(name) => {
                // First-class callable syntax: strlen(...)
                let name_idx = self.intern_string(name.clone());
                self.emit(Opcode::PushString(name_idx));
            }
            Expr::CallableFromMethod { object, method } => {
                // $obj->method(...) - First-class callable from method
                self.compile_expr(object)?;
                let method_idx = self.intern_string(method.clone());
                self.emit(Opcode::PushString(method_idx));
            }
            Expr::CallableFromStaticMethod { class, method } => {
                // Class::method(...) - First-class callable from static method
                let class_idx = self.intern_string(class.clone());
                let method_idx = self.intern_string(method.clone());
                self.emit(Opcode::PushString(class_idx));
                self.emit(Opcode::PushString(method_idx));
            }
            Expr::NewAnonymousClass {
                constructor_args,
                parent,
                interfaces: _,
                traits: _,
                properties,
                methods,
            } => {
                // Create anonymous class name
                let anon_name = format!("__anon_class_{}", self.classes.len());

                // Build a simple anonymous class with properties and methods
                let mut anon_class = CompiledClass::new(anon_name.clone());

                // Set parent if specified
                anon_class.parent = parent.clone();

                // Add properties
                for prop in properties {
                    let compiled_prop = CompiledProperty::from_ast(prop, false);
                    anon_class.properties.push(compiled_prop);
                }

                // Compile methods
                for method in methods {
                    let method_name = format!("{}::{}", anon_name, method.name);
                    let mut method_compiler = Compiler::new(method_name.clone());

                    if !method.is_static {
                        method_compiler.locals.insert("this".to_string(), 0);
                        method_compiler
                            .function
                            .local_names
                            .push("this".to_string());
                        method_compiler.next_local = 1;
                    }

                    let param_start = method_compiler.next_local;
                    for (i, param) in method.params.iter().enumerate() {
                        let slot = param_start + i as u16;
                        method_compiler.locals.insert(param.name.clone(), slot);
                        method_compiler
                            .function
                            .local_names
                            .push(param.name.clone());
                    }
                    method_compiler.next_local = param_start + method.params.len() as u16;
                    method_compiler.function.local_count = method_compiler.next_local;
                    method_compiler.function.param_count = method.params.len() as u8;
                    method_compiler.function.required_param_count = method
                        .params
                        .iter()
                        .filter(|p| p.default.is_none() && !p.is_variadic)
                        .count()
                        as u8;

                    // Store parameters and attributes for reflection
                    method_compiler.function.parameters = method.params.clone();
                    method_compiler.function.attributes = method.attributes.clone();

                    // Store parameter types for validation
                    for param in &method.params {
                        method_compiler
                            .function
                            .param_types
                            .push(param.type_hint.clone());
                    }

                    for stmt in &method.body {
                        method_compiler.compile_stmt(stmt)?;
                    }
                    method_compiler.emit(Opcode::ReturnNull);

                    // Merge any nested functions/closures from the method compiler
                    for (inner_name, inner_func) in method_compiler.functions.drain() {
                        self.functions.insert(inner_name, inner_func);
                    }

                    let compiled = Arc::new(method_compiler.function);
                    anon_class.methods.insert(method.name.clone(), compiled);
                }

                // Store the anonymous class
                self.classes.insert(anon_name.clone(), Arc::new(anon_class));

                // Emit new object opcode first (creates object with defaults)
                let class_idx = self.intern_string(anon_name);
                self.emit(Opcode::NewObject(class_idx));

                // Compile constructor args (after object creation)
                let arg_count = constructor_args.len();
                for arg in constructor_args {
                    self.compile_expr(&arg.value)?;
                }

                // Call constructor
                self.emit(Opcode::CallConstructor(arg_count as u8));
            }
            Expr::NewFiber { callback } => {
                // Compile callback expression
                self.compile_expr(callback)?;
                // Emit NewFiber opcode
                self.emit(Opcode::NewFiber);
            }
            Expr::FiberSuspend { value } => {
                // Fiber::suspend() - compile value or null
                if let Some(v) = value {
                    self.compile_expr(v)?;
                } else {
                    self.emit(Opcode::PushNull);
                }
            }
            Expr::FiberGetCurrent => {
                // Fiber::getCurrent() - not supported in VM yet
                self.emit(Opcode::PushNull);
            }
            Expr::CloneWith {
                object,
                modifications,
            } => {
                // Clone with modifications (PHP 8.4)
                self.compile_expr(object)?;
                self.emit(Opcode::Clone);

                // Apply modifications - each StoreCloneProperty validates property exists
                for modification in modifications {
                    // Compile the new value (object is already on stack from Clone or previous StoreCloneProperty)
                    self.compile_expr(&modification.value)?;
                    // Store property - pops object and value, pushes modified object
                    let prop_idx = self.intern_string(modification.property.clone());
                    self.emit(Opcode::StoreCloneProperty(prop_idx));
                }
                // The cloned (and modified) object is now on stack
            }
            Expr::Placeholder => {
                // Pipe placeholder - not yet supported
                return Err("Pipe placeholder not yet implemented".to_string());
            }
        }
        Ok(())
    }

    /// Compile a binary operation
    fn compile_binary_op(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<(), String> {
        // For short-circuit operators, handle specially
        match op {
            BinaryOp::And => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let short_circuit = self.emit_jump(Opcode::JumpIfFalse(0));
                self.emit(Opcode::Pop); // Pop the duplicate
                self.compile_expr(right)?;
                self.patch_jump(short_circuit);
                return Ok(());
            }
            BinaryOp::Or => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let short_circuit = self.emit_jump(Opcode::JumpIfTrue(0));
                self.emit(Opcode::Pop); // Pop the duplicate
                self.compile_expr(right)?;
                self.patch_jump(short_circuit);
                return Ok(());
            }
            BinaryOp::NullCoalesce => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let jump_if_not_null = self.emit_jump(Opcode::JumpIfNotNull(0));
                self.emit(Opcode::Pop); // Pop the null value
                self.compile_expr(right)?;
                self.patch_jump(jump_if_not_null);
                return Ok(());
            }
            BinaryOp::Pipe => {
                // Pipe operator: $left |> func(...)
                // The right side should be a function call (or first-class callable)
                // We pass $left as the first argument (or at placeholder position)

                // Right side should be a call or first-class callable
                match right {
                    Expr::FunctionCall { name, args } => {
                        // Find placeholder position
                        let placeholder_pos = args
                            .iter()
                            .position(|arg| matches!(&*arg.value, Expr::Placeholder));

                        if let Some(pos) = placeholder_pos {
                            // Placeholder found: insert piped value at that position
                            for (i, arg) in args.iter().enumerate() {
                                if i == pos {
                                    // Compile the left (piped) value at placeholder position
                                    self.compile_expr(left)?;
                                } else {
                                    self.compile_expr(&arg.value)?;
                                }
                            }
                            // Call with same arg count as original
                            let func_idx = self.intern_string(name.clone());
                            self.emit(Opcode::Call(func_idx, args.len() as u8));
                        } else {
                            // No placeholder: insert piped value as first argument
                            self.compile_expr(left)?;
                            // Push additional args after the piped value
                            for arg in args {
                                self.compile_expr(&arg.value)?;
                            }
                            // Call with left as first arg (1 + args.len())
                            let func_idx = self.intern_string(name.clone());
                            self.emit(Opcode::Call(func_idx, (1 + args.len()) as u8));
                        }
                    }
                    Expr::CallableFromFunction(func_name) => {
                        // First-class callable: func(...)
                        // Check if this is a built-in function
                        use crate::vm::builtins;
                        let is_builtin = builtins::is_builtin(&func_name);
                        let func_idx = self.intern_string(func_name.clone());
                        self.compile_expr(left)?;
                        self.emit(if is_builtin {
                            Opcode::CallBuiltin(func_idx, 1) // 1 arg (the piped value)
                        } else {
                            Opcode::Call(func_idx, 1) // 1 arg (the piped value)
                        });
                    }
                    _ => {
                        return Err(
                            "Pipe operator right-hand side must be a function call".to_string()
                        )
                    }
                }
                return Ok(());
            }
            _ => {}
        }

        // For other binary ops, evaluate both sides
        self.compile_expr(left)?;
        self.compile_expr(right)?;

        // Emit the appropriate opcode
        match op {
            BinaryOp::Add => self.emit(Opcode::Add),
            BinaryOp::Sub => self.emit(Opcode::Sub),
            BinaryOp::Mul => self.emit(Opcode::Mul),
            BinaryOp::Div => self.emit(Opcode::Div),
            BinaryOp::Mod => self.emit(Opcode::Mod),
            BinaryOp::Pow => self.emit(Opcode::Pow),
            BinaryOp::Concat => self.emit(Opcode::Concat),
            BinaryOp::Equal => self.emit(Opcode::Eq),
            BinaryOp::NotEqual => self.emit(Opcode::Ne),
            BinaryOp::Identical => self.emit(Opcode::Identical),
            BinaryOp::NotIdentical => self.emit(Opcode::NotIdentical),
            BinaryOp::LessThan => self.emit(Opcode::Lt),
            BinaryOp::LessEqual => self.emit(Opcode::Le),
            BinaryOp::GreaterThan => self.emit(Opcode::Gt),
            BinaryOp::GreaterEqual => self.emit(Opcode::Ge),
            BinaryOp::Spaceship => self.emit(Opcode::Spaceship),
            BinaryOp::And => self.emit(Opcode::And), // Non-short-circuit fallback
            BinaryOp::Or => self.emit(Opcode::Or),   // Non-short-circuit fallback
            BinaryOp::Xor => self.emit(Opcode::Xor),
            BinaryOp::NullCoalesce => unreachable!("Handled above"),
            _ => return Err(format!("Binary operator not yet implemented: {:?}", op)),
        };

        Ok(())
    }

    /// Compile a unary operation
    fn compile_unary_op(&mut self, op: &UnaryOp, operand: &Expr) -> Result<(), String> {
        match op {
            UnaryOp::Not => {
                self.compile_expr(operand)?;
                self.emit(Opcode::Not);
            }
            UnaryOp::Neg => {
                self.compile_expr(operand)?;
                self.emit(Opcode::Neg);
            }
            UnaryOp::PreInc | UnaryOp::PreDec => {
                // ++$x or --$x: increment/decrement, then return new value
                match operand {
                    Expr::Variable(var_name) => {
                        // Load current value
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::LoadFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::LoadVar(idx));
                        }

                        // Add or subtract 1
                        self.emit(Opcode::PushInt(1));
                        if matches!(op, UnaryOp::PreInc) {
                            self.emit(Opcode::Add);
                        } else {
                            self.emit(Opcode::Sub);
                        }

                        // Dup for return value
                        self.emit(Opcode::Dup);

                        // Store back
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::StoreFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::StoreVar(idx));
                        }
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let class_idx = self.intern_string(class.clone());
                        let prop_idx = self.intern_string(property.clone());

                        // Load current value
                        self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                        // Add or subtract 1
                        self.emit(Opcode::PushInt(1));
                        if matches!(op, UnaryOp::PreInc) {
                            self.emit(Opcode::Add);
                        } else {
                            self.emit(Opcode::Sub);
                        }

                        // Dup for return value
                        self.emit(Opcode::Dup);

                        // Store back
                        self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                        // Pop the value StoreStaticProp pushed
                        self.emit(Opcode::Pop);
                    }
                    _ => return Err("Increment/decrement requires a variable".to_string()),
                }
            }
            UnaryOp::PostInc | UnaryOp::PostDec => {
                // $x++ or $x--: return current value, then increment/decrement
                match operand {
                    Expr::Variable(var_name) => {
                        // Load current value (this will be our return value)
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::LoadFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::LoadVar(idx));
                        }

                        // Dup for calculation
                        self.emit(Opcode::Dup);

                        // Add or subtract 1
                        self.emit(Opcode::PushInt(1));
                        if matches!(op, UnaryOp::PostInc) {
                            self.emit(Opcode::Add);
                        } else {
                            self.emit(Opcode::Sub);
                        }

                        // Store the new value back
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::StoreFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::StoreVar(idx));
                        }
                        // Pop the value StoreFast pushed (we want the original value)
                        self.emit(Opcode::Pop);
                        // The old value is still on the stack
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let class_idx = self.intern_string(class.clone());
                        let prop_idx = self.intern_string(property.clone());

                        // Load current value (this will be our return value)
                        self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                        // Dup for calculation
                        self.emit(Opcode::Dup);

                        // Add or subtract 1
                        self.emit(Opcode::PushInt(1));
                        if matches!(op, UnaryOp::PostInc) {
                            self.emit(Opcode::Add);
                        } else {
                            self.emit(Opcode::Sub);
                        }

                        // Store the new value back
                        self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                        // Pop the value StoreStaticProp pushed (we want the original value)
                        self.emit(Opcode::Pop);
                        // The old value is still on the stack
                    }
                    _ => return Err("Increment/decrement requires a variable".to_string()),
                }
            }
        };

        Ok(())
    }

    /// Compile a ternary expression
    fn compile_ternary(
        &mut self,
        condition: &Expr,
        then_val: &Expr,
        else_val: &Expr,
    ) -> Result<(), String> {
        // Compile condition
        self.compile_expr(condition)?;

        // Jump to else if false
        let else_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        // Compile then value
        self.compile_expr(then_val)?;

        // Jump past else
        let end_jump = self.emit_jump(Opcode::Jump(0));

        // Patch else jump
        self.patch_jump(else_jump);

        // Compile else value
        self.compile_expr(else_val)?;

        // Patch end jump
        self.patch_jump(end_jump);

        Ok(())
    }

    /// Compile a match expression
    fn compile_match(
        &mut self,
        expr: &Expr,
        arms: &[crate::ast::MatchArm],
        default: &Option<Box<Expr>>,
    ) -> Result<(), String> {
        // Compile the subject expression and store it
        self.compile_expr(expr)?;
        let subject_slot = self.allocate_local("__match_subject__".to_string());
        self.emit(Opcode::StoreFast(subject_slot));

        let mut end_jumps = Vec::new();

        // For each arm, check conditions and emit result
        for arm in arms {
            let mut arm_jumps = Vec::new();

            // Check each condition in this arm (can have multiple: 1, 2, 3 =>)
            for condition in &arm.conditions {
                self.emit(Opcode::LoadFast(subject_slot));
                self.compile_expr(condition)?;
                self.emit(Opcode::Identical); // Match uses strict comparison
                arm_jumps.push(self.emit_jump(Opcode::JumpIfTrue(0)));
            }

            // If none matched, jump to next arm
            let skip_arm = self.emit_jump(Opcode::Jump(0));

            // Patch arm condition jumps to result
            for jump in arm_jumps {
                self.patch_jump(jump);
            }

            // Compile arm result
            self.compile_expr(&arm.result)?;

            // Jump to end
            end_jumps.push(self.emit_jump(Opcode::Jump(0)));

            // Patch skip to here
            self.patch_jump(skip_arm);
        }

        // Compile default if present, otherwise throw UnhandledMatchError
        if let Some(default_expr) = default {
            self.compile_expr(default_expr)?;
        } else {
            // Throw UnhandledMatchError exception
            // Create new UnhandledMatchError("Unhandled match value")
            let class_idx = self.intern_string("UnhandledMatchError".to_string());
            self.emit(Opcode::NewObject(class_idx));
            let msg_idx = self.intern_string("Unhandled match value".to_string());
            self.emit(Opcode::PushString(msg_idx));
            self.emit(Opcode::CallConstructor(1));
            self.emit(Opcode::Throw);
        }

        // Patch all end jumps
        for jump in end_jumps {
            self.patch_jump(jump);
        }

        Ok(())
    }

    /// Compile an arrow function (fn($x) => expr)
    fn compile_arrow_function(
        &mut self,
        params: &[FunctionParam],
        body: &Expr,
    ) -> Result<(), String> {
        // Create a unique name for the arrow function using a global counter
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ARROW_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = ARROW_COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!("__arrow_{}", id);

        // Find all variables used in the body that are NOT parameters
        let param_names: std::collections::HashSet<_> =
            params.iter().map(|p| p.name.as_str()).collect();
        let mut captured_vars = Vec::new();
        self.find_captured_vars(body, &param_names, &mut captured_vars);

        // Emit CaptureVar for each captured variable
        for var_name in &captured_vars {
            let var_idx = self.intern_string(var_name.clone());
            self.emit(Opcode::CaptureVar(var_idx));
        }

        // Create a new compiler for the closure
        let mut closure_compiler = Compiler::new(name.clone());

        // First N locals are captured variables
        for (i, var_name) in captured_vars.iter().enumerate() {
            closure_compiler.locals.insert(var_name.clone(), i as u16);
            closure_compiler.function.local_names.push(var_name.clone());
        }
        closure_compiler.next_local = captured_vars.len() as u16;

        // Set up parameters as local variables (after captured vars)
        for param in params.iter() {
            let slot = closure_compiler.next_local;
            closure_compiler.locals.insert(param.name.clone(), slot);
            closure_compiler
                .function
                .local_names
                .push(param.name.clone());
            closure_compiler.next_local += 1;
        }

        closure_compiler.function.local_count = closure_compiler.next_local;
        closure_compiler.function.param_count = params.len() as u8;
        closure_compiler.function.required_param_count = params
            .iter()
            .filter(|p| p.default.is_none() && !p.is_variadic)
            .count() as u8;
        closure_compiler.function.parameters = params.to_vec();

        // Emit default value initialization for parameters with defaults
        // The parameter slots start after captured variables
        let captured_count = captured_vars.len();
        for (i, param) in params.iter().enumerate() {
            if let Some(default_expr) = &param.default {
                let slot = (captured_count + i) as u16;
                // Load param value
                closure_compiler.emit(Opcode::LoadFast(slot));
                // If not null, skip default assignment
                let skip_jump = closure_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                // Pop the null value from the check
                closure_compiler.emit(Opcode::Pop);
                // Evaluate and store default
                closure_compiler.compile_expr(default_expr)?;
                closure_compiler.emit(Opcode::StoreFast(slot));
                // Jump past the Pop that handles the not-null case
                let end_jump = closure_compiler.emit_jump(Opcode::Jump(0));
                // Patch skip_jump to here - this is where not-null lands
                closure_compiler.patch_jump(skip_jump);
                // Pop the non-null value we checked
                closure_compiler.emit(Opcode::Pop);
                // Patch end_jump to here
                closure_compiler.patch_jump(end_jump);
            }
        }

        // Arrow functions return their body expression
        closure_compiler.compile_expr(body)?;
        closure_compiler.emit(Opcode::Return);

        // Merge any nested functions/closures from the inner compiler
        for (inner_name, inner_func) in closure_compiler.functions.drain() {
            self.functions.insert(inner_name, inner_func);
        }

        // Store the compiled closure
        let compiled = Arc::new(closure_compiler.function);
        let func_idx = self.intern_string(name.clone());
        self.functions.insert(name, compiled);

        // Emit closure creation opcode with captured var count
        self.emit(Opcode::CreateClosure(func_idx, captured_vars.len() as u8));

        Ok(())
    }

    /// Find all variables used in an expression that should be captured
    fn find_captured_vars(
        &self,
        expr: &Expr,
        param_names: &std::collections::HashSet<&str>,
        captured: &mut Vec<String>,
    ) {
        match expr {
            Expr::Variable(name) => {
                // If it's not a parameter and not already captured, capture it
                if !param_names.contains(name.as_str()) && !captured.contains(name) {
                    // Check if it exists in current scope
                    if self.locals.contains_key(name) || self.is_global_var(name) {
                        captured.push(name.clone());
                    }
                }
            }
            Expr::Binary { left, right, .. } => {
                self.find_captured_vars(left, param_names, captured);
                self.find_captured_vars(right, param_names, captured);
            }
            Expr::Unary { expr, .. } => {
                self.find_captured_vars(expr, param_names, captured);
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.find_captured_vars(condition, param_names, captured);
                self.find_captured_vars(then_expr, param_names, captured);
                self.find_captured_vars(else_expr, param_names, captured);
            }
            Expr::FunctionCall { args, .. } => {
                for arg in args {
                    self.find_captured_vars(&arg.value, param_names, captured);
                }
            }
            Expr::MethodCall { object, args, .. } => {
                self.find_captured_vars(object, param_names, captured);
                for arg in args {
                    self.find_captured_vars(&arg.value, param_names, captured);
                }
            }
            Expr::PropertyAccess { object, .. } => {
                self.find_captured_vars(object, param_names, captured);
            }
            Expr::ArrayAccess { array, index } => {
                self.find_captured_vars(array, param_names, captured);
                self.find_captured_vars(index, param_names, captured);
            }
            Expr::Array(elements) => {
                for elem in elements {
                    if let Some(ref key) = elem.key {
                        self.find_captured_vars(key, param_names, captured);
                    }
                    self.find_captured_vars(&elem.value, param_names, captured);
                }
            }
            Expr::Grouped(inner) => {
                self.find_captured_vars(inner, param_names, captured);
            }
            Expr::Spread(inner) => {
                self.find_captured_vars(inner, param_names, captured);
            }
            Expr::CallableCall { callable, args } => {
                self.find_captured_vars(callable, param_names, captured);
                for arg in args {
                    self.find_captured_vars(&arg.value, param_names, captured);
                }
            }
            Expr::CallableFromMethod { object, .. } => {
                self.find_captured_vars(object, param_names, captured);
            }
            Expr::Assign { value, .. } => {
                self.find_captured_vars(value, param_names, captured);
            }
            Expr::ArrayAssign {
                array,
                index,
                value,
                ..
            } => {
                self.find_captured_vars(array, param_names, captured);
                if let Some(ref idx) = index {
                    self.find_captured_vars(idx, param_names, captured);
                }
                self.find_captured_vars(value, param_names, captured);
            }
            Expr::PropertyAssign { object, value, .. } => {
                self.find_captured_vars(object, param_names, captured);
                self.find_captured_vars(value, param_names, captured);
            }
            Expr::StaticPropertyAssign { value, .. } => {
                self.find_captured_vars(value, param_names, captured);
            }
            Expr::New { args, .. } => {
                for arg in args {
                    self.find_captured_vars(&arg.value, param_names, captured);
                }
            }
            Expr::Throw(inner) => {
                self.find_captured_vars(inner, param_names, captured);
            }
            Expr::Match {
                expr,
                arms,
                default,
            } => {
                self.find_captured_vars(expr, param_names, captured);
                for arm in arms {
                    for condition in &arm.conditions {
                        self.find_captured_vars(condition, param_names, captured);
                    }
                    self.find_captured_vars(&arm.result, param_names, captured);
                }
                if let Some(ref def) = default {
                    self.find_captured_vars(def, param_names, captured);
                }
            }
            Expr::ArrowFunction {
                params: inner_params,
                body,
            } => {
                // Nested arrow function - we need to scan its body to find variables
                // that the inner might need from OUR enclosing scope.
                // We exclude:
                // - Our params (the outer arrow's params)
                // - The inner arrow's params
                // Variables found that aren't in either set need to be captured by us
                // so we can pass them to the inner when it's created.
                let mut combined_params: std::collections::HashSet<&str> = param_names.clone();
                for p in inner_params {
                    combined_params.insert(&p.name);
                }
                self.find_captured_vars(body, &combined_params, captured);
            }
            Expr::Clone { object } => {
                self.find_captured_vars(object, param_names, captured);
            }
            Expr::CloneWith {
                object,
                modifications,
            } => {
                self.find_captured_vars(object, param_names, captured);
                for modification in modifications {
                    self.find_captured_vars(&modification.value, param_names, captured);
                }
            }
            // Literals and other expressions that don't contain variables
            _ => {}
        }
    }

    fn is_global_var(&self, name: &str) -> bool {
        // For now, consider all non-local variables as potentially global
        // In a more sophisticated implementation, we'd track actual global scope
        !self.locals.contains_key(name)
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

    /// Patch a jump instruction to jump to a specific target
    #[allow(dead_code)]
    fn patch_jump_at(&mut self, offset: usize, target: usize) {
        match &mut self.function.bytecode[offset] {
            Opcode::Jump(ref mut target_ref) => *target_ref = target as u32,
            Opcode::JumpIfFalse(ref mut target_ref) => *target_ref = target as u32,
            Opcode::JumpIfTrue(ref mut target_ref) => *target_ref = target as u32,
            Opcode::JumpIfNull(ref mut target_ref) => *target_ref = target as u32,
            Opcode::JumpIfNotNull(ref mut target_ref) => *target_ref = target as u32,
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
        // Create a new compiler for the function
        let mut func_compiler = Compiler::new(name.to_string());

        // Inherit strict_types from parent compiler
        func_compiler.function.strict_types = self.strict_types;

        // Store parameters and attributes for reflection
        func_compiler.function.parameters = params.to_vec();
        func_compiler.function.attributes = attributes.to_vec();

        // Set up parameters as local variables
        for (i, param) in params.iter().enumerate() {
            func_compiler.locals.insert(param.name.clone(), i as u16);
            func_compiler.function.local_names.push(param.name.clone());
        }
        func_compiler.next_local = params.len() as u16;
        func_compiler.function.local_count = params.len() as u16;
        func_compiler.function.param_count = params.len() as u8;
        func_compiler.function.required_param_count = params
            .iter()
            .filter(|p| p.default.is_none() && !p.is_variadic)
            .count() as u8;
        func_compiler.function.return_type = return_type.clone();
        func_compiler.function.is_variadic = params.iter().any(|p| p.is_variadic);

        // Store parameter types for validation
        for param in params {
            func_compiler
                .function
                .param_types
                .push(param.type_hint.clone());
        }

        // Emit default value initialization for parameters with defaults
        for (i, param) in params.iter().enumerate() {
            if let Some(default_expr) = &param.default {
                let slot = i as u16;
                // Load param value
                func_compiler.emit(Opcode::LoadFast(slot));
                // If not null, skip default assignment
                let skip_jump = func_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                // Pop the null value from the check
                func_compiler.emit(Opcode::Pop);
                // Evaluate and store default
                func_compiler.compile_expr(default_expr)?;
                func_compiler.emit(Opcode::StoreFast(slot));
                // Jump past the Pop that handles the not-null case
                let end_jump = func_compiler.emit_jump(Opcode::Jump(0));
                // Patch skip_jump to here - this is where not-null lands
                func_compiler.patch_jump(skip_jump);
                // Pop the non-null value we checked
                func_compiler.emit(Opcode::Pop);
                // Patch end_jump to here
                func_compiler.patch_jump(end_jump);
            }
        }

        // Compile function body
        for stmt in body {
            func_compiler.compile_stmt(stmt)?;
        }

        // Add implicit return null if no return statement
        func_compiler.emit(Opcode::ReturnNull);

        // Merge any nested functions/closures from the inner compiler
        for (inner_name, inner_func) in func_compiler.functions.drain() {
            self.functions.insert(inner_name, inner_func);
        }

        // Store compiled function
        let compiled = Arc::new(func_compiler.function);
        self.functions.insert(name.to_string(), compiled);

        Ok(())
    }

    /// Compile a switch statement
    fn compile_switch(
        &mut self,
        expr: &Expr,
        cases: &[crate::ast::SwitchCase],
        default: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        // Compile the switch expression once and store in a local
        self.compile_expr(expr)?;
        let switch_slot = self.allocate_local("__switch_expr__".to_string());
        self.emit(Opcode::StoreFast(switch_slot));

        // Emit LoopStart for break support BEFORE comparisons
        // (continue_target doesn't apply to switch, use 0)
        let loop_start_idx = self.emit(Opcode::LoopStart(0, 0));

        // Track jump locations for case bodies
        let mut case_jumps: Vec<usize> = Vec::new();

        // For each case, emit comparison and conditional jump
        for case in cases {
            // Load switch expression
            self.emit(Opcode::LoadFast(switch_slot));
            // Compile case value
            self.compile_expr(&case.value)?;
            // Compare
            self.emit(Opcode::Eq);
            // Jump to this case's body if equal
            case_jumps.push(self.emit_jump(Opcode::JumpIfTrue(0)));
        }

        // Jump to default if no case matched
        let default_jump = self.emit_jump(Opcode::Jump(0));

        // Emit case bodies
        for (i, case) in cases.iter().enumerate() {
            // Patch the jump to this case
            self.patch_jump(case_jumps[i]);

            // Compile case body
            for stmt in &case.body {
                self.compile_stmt(stmt)?;
            }

            // Don't emit jump at end - fall through to next case or break will handle it
        }

        // Emit default body
        self.patch_jump(default_jump);
        if let Some(default_body) = default {
            for stmt in default_body {
                self.compile_stmt(stmt)?;
            }
        }

        // LoopEnd
        self.emit(Opcode::LoopEnd);

        // Patch LoopStart's break target to here
        let end_offset = self.current_offset();
        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = end_offset as u32;
        }

        Ok(())
    }

    /// Compile a try/catch/finally statement
    fn compile_try_catch(
        &mut self,
        try_body: &[Stmt],
        catch_clauses: &[crate::ast::CatchClause],
        finally_body: &Option<Vec<Stmt>>,
    ) -> Result<(), String> {
        // For now, just compile the try body without exception handling
        // Full exception support requires VM changes

        // Emit try start with placeholder offsets
        let try_start = self.emit_jump(Opcode::TryStart(0, 0));

        // Compile try body
        for stmt in try_body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Opcode::TryEnd);

        // Jump past catch blocks AND finally to end
        let skip_catch = self.emit_jump(Opcode::Jump(0));

        // Patch try start with catch offset
        let catch_offset = self.current_offset() as u32;
        if let Opcode::TryStart(ref mut c, _) = self.function.bytecode[try_start] {
            *c = catch_offset;
        }

        // Compile catch clauses
        let mut end_catch_jumps = Vec::new();
        for (i, catch) in catch_clauses.iter().enumerate() {
            // Allocate local for exception variable
            let var_slot = self.allocate_local(catch.variable.clone());

            // The caught exception will be on the stack
            self.emit(Opcode::StoreFast(var_slot));

            // Compile catch body
            for stmt in &catch.body {
                self.compile_stmt(stmt)?;
            }

            // After catch body executes, jump to finally (or end if no finally)
            // Don't emit jump for the last catch clause
            if i < catch_clauses.len() - 1 {
                let jump_to_end = self.emit_jump(Opcode::Jump(0));
                end_catch_jumps.push(jump_to_end);
            }
        }

        // Patch skip_catch and all end_catch jumps to point here (before finally)
        self.patch_jump(skip_catch);
        for jump in end_catch_jumps {
            self.patch_jump(jump);
        }

        // Compile finally body if present
        if let Some(finally) = finally_body {
            // Patch TryStart with finally offset BEFORE emitting FinallyStart
            let finally_offset = self.current_offset() as u32;
            if let Opcode::TryStart(_, ref mut f) = self.function.bytecode[try_start] {
                *f = finally_offset;
            }

            self.emit(Opcode::FinallyStart);
            for stmt in finally {
                self.compile_stmt(stmt)?;
            }
            self.emit(Opcode::FinallyEnd);
        }

        Ok(())
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
        // Fully qualify the class name with current namespace
        let qualified_name = if let Some(ref ns) = self.current_namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        };

        // Resolve parent class name through use aliases
        let resolved_parent = parent.as_ref().map(|p| self.resolve_qualified_name(p));

        // Check if parent class exists and is not final
        if let Some(ref parent_name) = resolved_parent {
            // Check if parent class exists (allow built-in classes)
            let is_builtin = matches!(
                parent_name.as_str(),
                "Exception"
                    | "Error"
                    | "TypeError"
                    | "InvalidArgumentException"
                    | "UnhandledMatchError"
            );

            if let Some(parent_class) = self.classes.get(parent_name) {
                if parent_class.is_final {
                    return Err(format!("cannot extend final class {}", parent_name));
                }
            } else if !is_builtin {
                return Err(format!("Parent class '{}' not found", parent_name));
            }
        }

        // Resolve interface names through use aliases
        let resolved_interfaces: Vec<String> = interfaces
            .iter()
            .map(|i| self.resolve_qualified_name(i))
            .collect();

        let mut compiled_class = CompiledClass::new(qualified_name.clone());
        compiled_class.is_abstract = is_abstract;
        compiled_class.is_final = is_final;
        compiled_class.readonly = readonly;
        compiled_class.parent = resolved_parent;
        compiled_class.interfaces = resolved_interfaces.clone();
        compiled_class.traits = trait_uses.iter().flat_map(|t| t.traits.clone()).collect();
        compiled_class.attributes = attributes.to_vec();

        // Verify interfaces exist
        for iface_name in &resolved_interfaces {
            if !self.interfaces.contains_key(iface_name) {
                return Err(format!("Interface '{}' not found", iface_name));
            }
        }

        // Compile properties
        for prop in properties {
            let mut compiled_prop = CompiledProperty::from_ast(prop, readonly);

            // Compile property hooks (PHP 8.4)
            for hook in &prop.hooks {
                let hook_method_name = match hook.hook_type {
                    crate::ast::PropertyHookType::Get => {
                        format!("{}::__prop_get_{}", qualified_name, prop.name)
                    }
                    crate::ast::PropertyHookType::Set => {
                        format!("{}::__prop_set_{}", qualified_name, prop.name)
                    }
                };

                let mut hook_compiler = Compiler::new(hook_method_name.clone());

                // Add $this as first local
                hook_compiler.locals.insert("this".to_string(), 0);
                hook_compiler.function.local_names.push("this".to_string());
                hook_compiler.next_local = 1;

                // For set hooks, add $value parameter
                if matches!(hook.hook_type, crate::ast::PropertyHookType::Set) {
                    hook_compiler.locals.insert("value".to_string(), 1);
                    hook_compiler.function.local_names.push("value".to_string());
                    hook_compiler.next_local = 2;
                    hook_compiler.function.param_count = 1;
                    hook_compiler.function.required_param_count = 1;
                }

                hook_compiler.function.local_count = hook_compiler.next_local;

                // Compile hook body
                match &hook.body {
                    crate::ast::PropertyHookBody::Expression(expr) => {
                        // For get hooks: return the expression
                        // For set hooks: evaluate expression (for side effects)
                        hook_compiler.compile_expr(expr)?;
                        if matches!(hook.hook_type, crate::ast::PropertyHookType::Get) {
                            hook_compiler.emit(Opcode::Return);
                        } else {
                            hook_compiler.emit(Opcode::Pop);
                            hook_compiler.emit(Opcode::ReturnNull);
                        }
                    }
                    crate::ast::PropertyHookBody::Block(stmts) => {
                        for stmt in stmts {
                            hook_compiler.compile_stmt(stmt)?;
                        }
                        hook_compiler.emit(Opcode::ReturnNull);
                    }
                }

                // Merge any nested functions from the hook compiler
                for (inner_name, inner_func) in hook_compiler.functions.drain() {
                    self.functions.insert(inner_name, inner_func);
                }

                let compiled_hook = Arc::new(hook_compiler.function);

                // Store hook method in class and update property reference
                match hook.hook_type {
                    crate::ast::PropertyHookType::Get => {
                        compiled_prop.get_hook = Some(format!("__prop_get_{}", prop.name));
                        compiled_class
                            .methods
                            .insert(format!("__prop_get_{}", prop.name), compiled_hook);
                    }
                    crate::ast::PropertyHookType::Set => {
                        compiled_prop.set_hook = Some(format!("__prop_set_{}", prop.name));
                        compiled_class
                            .methods
                            .insert(format!("__prop_set_{}", prop.name), compiled_hook);
                    }
                }
            }

            if prop.is_static {
                // Static properties go in a different map
                // Use the compiled default value if available
                let default_value = compiled_prop
                    .default
                    .clone()
                    .unwrap_or(crate::runtime::Value::Null);
                compiled_class
                    .static_properties
                    .insert(prop.name.clone(), default_value);
                // Track if static property is readonly
                if prop.readonly || readonly {
                    compiled_class
                        .readonly_static_properties
                        .insert(prop.name.clone());
                }
            }
            compiled_class.properties.push(compiled_prop);
        }

        // Add constructor-promoted properties
        // These are parameters with visibility modifiers that become properties
        for method in methods {
            if method.name == "__construct" {
                for param in &method.params {
                    if param.visibility.is_some() {
                        // This is a promoted property
                        let promoted_prop = CompiledProperty {
                            name: param.name.clone(),
                            visibility: param.visibility.unwrap(),
                            write_visibility: None,
                            default: None, // Value comes from constructor argument
                            readonly: param.readonly || readonly, // Inherit readonly from param or class
                            is_static: false,
                            type_hint: None,
                            attributes: param.attributes.clone(),
                            get_hook: None,
                            set_hook: None,
                        };
                        compiled_class.properties.push(promoted_prop);
                    }
                }
            }
        }

        // Add properties from used traits
        for trait_name in &compiled_class.traits {
            if let Some(trait_def) = self.traits.get(trait_name) {
                for trait_prop in &trait_def.properties {
                    // Add trait property to class if not already defined
                    if !compiled_class
                        .properties
                        .iter()
                        .any(|p| p.name == trait_prop.name)
                    {
                        compiled_class.properties.push(trait_prop.clone());
                    }
                }
            }
        }

        // Check for trait method conflicts
        // Build a map of method names to the traits that define them
        let mut trait_methods: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for trait_name in &compiled_class.traits {
            if let Some(trait_def) = self.traits.get(trait_name) {
                for method_name in trait_def.methods.keys() {
                    trait_methods
                        .entry(method_name.clone())
                        .or_default()
                        .push(trait_name.clone());
                }
            }
        }

        // Check for conflicts (methods defined in multiple traits)
        for (method_name, defining_traits) in &trait_methods {
            if defining_traits.len() > 1 {
                // Check if the class itself defines this method (which resolves the conflict)
                let class_defines_method = methods.iter().any(|m| &m.name == method_name);
                if !class_defines_method {
                    // Unresolved conflict
                    return Err(format!(
                        "Trait method conflict: {} is defined in multiple traits ({})",
                        method_name,
                        defining_traits.join(", ")
                    ));
                }
            }
        }

        // Compile methods
        for method in methods {
            // Check if method has #[\Override] attribute
            let has_override_attr = method
                .attributes
                .iter()
                .any(|attr| attr.name == "Override" || attr.name == "\\Override");

            // If method has #[\Override], verify parent/interface/trait method exists
            if has_override_attr {
                let mut found_parent_method = false;

                // Check parent class chain
                let mut current_parent = compiled_class.parent.clone();
                while let Some(parent_name) = current_parent {
                    if let Some(parent_class) = self.classes.get(&parent_name) {
                        if parent_class.methods.contains_key(&method.name)
                            || parent_class.static_methods.contains_key(&method.name)
                        {
                            found_parent_method = true;
                            break;
                        }
                        current_parent = parent_class.parent.clone();
                    } else {
                        break;
                    }
                }

                // Check implemented interfaces
                if !found_parent_method {
                    for iface_name in &compiled_class.interfaces {
                        if let Some(iface_def) = self.interfaces.get(iface_name) {
                            if iface_def
                                .method_signatures
                                .iter()
                                .any(|(name, _)| name == &method.name)
                            {
                                found_parent_method = true;
                                break;
                            }
                        }
                    }
                }

                // Check used traits
                if !found_parent_method {
                    for trait_name in &compiled_class.traits {
                        if let Some(trait_def) = self.traits.get(trait_name) {
                            if trait_def.methods.contains_key(&method.name) {
                                found_parent_method = true;
                                break;
                            }
                        }
                    }
                }

                if !found_parent_method {
                    return Err(format!(
                        "{}::{} has #[\\Override] attribute, but no matching parent method exists",
                        name, method.name
                    ));
                }
            }

            // Check if parent class has a final method with the same name
            if let Some(parent_name) = compiled_class.parent.as_ref() {
                if let Some(parent_class) = self.classes.get(parent_name) {
                    // Check both regular and static methods
                    let has_method = parent_class.methods.contains_key(&method.name)
                        || parent_class.static_methods.contains_key(&method.name);
                    if has_method
                        && parent_class
                            .method_finals
                            .get(&method.name)
                            .copied()
                            .unwrap_or(false)
                    {
                        return Err(format!(
                            "Cannot override final method {}::{}",
                            parent_name, method.name
                        ));
                    }
                }
            }

            let method_name = format!("{}::{}", qualified_name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            // Add $this as first local for non-static methods
            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            // Set up parameters as local variables
            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();
            method_compiler.function.is_variadic = method.params.iter().any(|p| p.is_variadic);

            // Store parameters and attributes for reflection
            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            // Store parameter types for validation
            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            // Emit default value initialization for parameters with defaults
            // Check if param is Null and set to default if so
            for (i, param) in method.params.iter().enumerate() {
                if let Some(default_expr) = &param.default {
                    let slot = param_start + i as u16;
                    // Load param value
                    method_compiler.emit(Opcode::LoadFast(slot));
                    // If not null, skip default assignment
                    let skip_jump = method_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                    // Pop the null value from the check
                    method_compiler.emit(Opcode::Pop);
                    // Evaluate and store default
                    method_compiler.compile_expr(default_expr)?;
                    method_compiler.emit(Opcode::StoreFast(slot));
                    // Jump past the Pop that handles the not-null case
                    let end_jump = method_compiler.emit_jump(Opcode::Jump(0));
                    // Patch skip_jump to here - this is where not-null lands
                    method_compiler.patch_jump(skip_jump);
                    // Pop the non-null value we checked
                    method_compiler.emit(Opcode::Pop);
                    // Patch end_jump to here
                    method_compiler.patch_jump(end_jump);
                }
            }

            // For __construct, emit constructor property promotion assignments
            // Parameters with visibility modifiers get assigned to $this->name
            if method.name == "__construct" && !method.is_static {
                for (i, param) in method.params.iter().enumerate() {
                    if param.visibility.is_some() {
                        // Load the parameter value
                        let slot = param_start + i as u16;
                        method_compiler.emit(Opcode::LoadFast(slot));
                        // Store to property with same name as parameter using StoreThisProperty
                        let prop_idx = method_compiler.intern_string(param.name.clone());
                        method_compiler.emit(Opcode::StoreThisProperty(prop_idx));
                    }
                }
            }

            // Compile method body
            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            // Add implicit return null
            method_compiler.emit(Opcode::ReturnNull);

            // Merge any nested functions/closures from the method compiler
            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_class
                .method_visibility
                .insert(method.name.clone(), method.visibility);
            compiled_class
                .method_finals
                .insert(method.name.clone(), method.is_final);
            compiled_class
                .method_abstracts
                .insert(method.name.clone(), method.is_abstract);

            if method.is_static {
                compiled_class
                    .static_methods
                    .insert(method.name.clone(), compiled);
            } else {
                compiled_class.methods.insert(method.name.clone(), compiled);
            }
        }

        // Verify abstract method implementations from parent (only for non-abstract classes)
        if !is_abstract {
            if let Some(parent_name) = &compiled_class.parent {
                if let Some(parent_class) = self.classes.get(parent_name) {
                    for (method_name, is_abs) in &parent_class.method_abstracts {
                        if *is_abs {
                            let has_method = compiled_class.methods.contains_key(method_name)
                                || compiled_class.static_methods.contains_key(method_name);
                            if !has_method {
                                return Err(format!("Class '{}' must implement abstract method '{}' from class '{}'",
                                    name, method_name, parent_name));
                            }
                        }
                    }
                }
            }
        }

        // Verify interface method implementations (only for non-abstract classes)
        if !is_abstract {
            for interface in interfaces {
                if let Some(iface_name) = interface.last() {
                    if let Some(iface_def) = self.interfaces.get(iface_name) {
                        for (method_name, _param_count) in &iface_def.method_signatures {
                            let has_method = compiled_class.methods.contains_key(method_name)
                                || compiled_class.static_methods.contains_key(method_name);
                            if !has_method {
                                return Err(format!(
                                    "Class '{}' does not implement method '{}' from interface '{}'",
                                    name, method_name, iface_name
                                ));
                            }
                        }
                    }
                }
            }
        }

        self.classes
            .insert(qualified_name, Arc::new(compiled_class));
        Ok(())
    }

    /// Compile an interface definition
    fn compile_interface(
        &mut self,
        name: &str,
        parents: &[crate::ast::QualifiedName],
        methods: &[crate::ast::InterfaceMethodSignature],
        constants: &[crate::ast::InterfaceConstant],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        // Qualify the interface name with current namespace
        let qualified_name = if let Some(ref ns) = self.current_namespace {
            format!("{}\\{}", ns, name)
        } else {
            name.to_string()
        };

        let mut compiled_interface = CompiledInterface::new(qualified_name.clone());
        // Resolve parent interface names through use aliases
        compiled_interface.parents = parents
            .iter()
            .map(|p| self.resolve_qualified_name(p))
            .collect();
        compiled_interface.attributes = attributes.to_vec();

        // Store method signatures (name, param_count)
        for method in methods {
            compiled_interface
                .method_signatures
                .push((method.name.clone(), method.params.len() as u8));
        }

        // Store constants (would need to evaluate at compile time)
        for constant in constants {
            // For now, we'll skip constant evaluation - needs expression evaluation
            let _ = constant;
        }

        self.interfaces
            .insert(qualified_name, Arc::new(compiled_interface));
        Ok(())
    }

    /// Compile a trait definition
    fn compile_trait(
        &mut self,
        name: &str,
        uses: &[String],
        properties: &[crate::ast::Property],
        methods: &[Method],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        let mut compiled_trait = CompiledTrait::new(name.to_string());
        compiled_trait.uses = uses.to_vec();
        compiled_trait.attributes = attributes.to_vec();

        // Compile properties
        for prop in properties {
            let compiled_prop = CompiledProperty::from_ast(prop, false);
            compiled_trait.properties.push(compiled_prop);
        }

        // Compile methods (similar to class methods)
        for method in methods {
            let method_name = format!("{}::{}", name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            // Add $this as first local
            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            // Set up parameters
            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();

            // Store parameters and attributes for reflection
            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            // Store parameter types for validation
            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            // Compile method body
            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            method_compiler.emit(Opcode::ReturnNull);

            // Merge any nested functions/closures from the method compiler
            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_trait.methods.insert(method.name.clone(), compiled);
        }

        self.traits
            .insert(name.to_string(), Arc::new(compiled_trait));
        Ok(())
    }

    /// Compile an enum definition
    fn compile_enum(
        &mut self,
        name: &str,
        backing_type: &crate::ast::EnumBackingType,
        cases: &[crate::ast::EnumCase],
        methods: &[Method],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        let mut compiled_enum = CompiledEnum::new(name.to_string(), *backing_type);
        compiled_enum.attributes = attributes.to_vec();

        // Store enum cases and check for duplicate backing values
        let mut seen_values: std::collections::HashSet<String> = std::collections::HashSet::new();
        for case in cases {
            // Evaluate the backing value expression if present
            let backing_value = if let Some(expr) = &case.value {
                // Handle simple literal values at compile time
                match expr {
                    Expr::Integer(n) => Some(crate::runtime::Value::Integer(*n)),
                    Expr::Float(n) => Some(crate::runtime::Value::Float(*n)),
                    Expr::String(s) => Some(crate::runtime::Value::String(s.clone())),
                    _ => None, // Complex expressions not supported yet
                }
            } else {
                None
            };

            // Check backing value type matches declared backing type
            if let Some(ref val) = backing_value {
                use crate::ast::EnumBackingType;
                let type_matches = matches!(
                    (backing_type, val),
                    (EnumBackingType::Int, crate::runtime::Value::Integer(_))
                        | (EnumBackingType::String, crate::runtime::Value::String(_))
                        | (EnumBackingType::None, _)
                );
                if !type_matches {
                    let expected_type = match backing_type {
                        EnumBackingType::Int => "int",
                        EnumBackingType::String => "string",
                        EnumBackingType::None => "none",
                    };
                    return Err(format!(
                        "Enum case '{}::{}' must have {} backing value",
                        name, case.name, expected_type
                    ));
                }

                // Check for duplicate backing values in backed enums
                let val_str = format!("{:?}", val);
                if !seen_values.insert(val_str) {
                    return Err("Duplicate case value in backed enum".to_string());
                }
            }

            compiled_enum.cases.insert(case.name.clone(), backing_value);
            compiled_enum.case_order.push(case.name.clone()); // Preserve insertion order
        }

        // Compile methods
        for method in methods {
            let method_name = format!("{}::{}", name, method.name);
            let mut method_compiler = Compiler::new(method_name.clone());

            // Add $this as first local
            if !method.is_static {
                method_compiler.locals.insert("this".to_string(), 0);
                method_compiler
                    .function
                    .local_names
                    .push("this".to_string());
                method_compiler.next_local = 1;
            }

            let param_start = method_compiler.next_local;
            for (i, param) in method.params.iter().enumerate() {
                let slot = param_start + i as u16;
                method_compiler.locals.insert(param.name.clone(), slot);
                method_compiler
                    .function
                    .local_names
                    .push(param.name.clone());
            }
            method_compiler.next_local = param_start + method.params.len() as u16;
            method_compiler.function.local_count = method_compiler.next_local;
            method_compiler.function.param_count = method.params.len() as u8;
            method_compiler.function.required_param_count = method
                .params
                .iter()
                .filter(|p| p.default.is_none() && !p.is_variadic)
                .count() as u8;
            method_compiler.function.return_type = method.return_type.clone();

            // Store parameters and attributes for reflection
            method_compiler.function.parameters = method.params.clone();
            method_compiler.function.attributes = method.attributes.clone();

            // Store parameter types for validation
            for param in &method.params {
                method_compiler
                    .function
                    .param_types
                    .push(param.type_hint.clone());
            }

            for stmt in &method.body {
                method_compiler.compile_stmt(stmt)?;
            }

            method_compiler.emit(Opcode::ReturnNull);

            // Merge any nested functions/closures from the method compiler
            for (inner_name, inner_func) in method_compiler.functions.drain() {
                self.functions.insert(inner_name, inner_func);
            }

            let compiled = Arc::new(method_compiler.function);
            compiled_enum.methods.insert(method.name.clone(), compiled);
        }

        self.enums.insert(name.to_string(), Arc::new(compiled_enum));
        Ok(())
    }

    /// Resolve a QualifiedName to a fully qualified class name string
    fn resolve_qualified_name(&self, qname: &crate::ast::QualifiedName) -> String {
        if qname.is_fully_qualified {
            // Already fully qualified - just join the parts
            qname.parts.join("\\")
        } else {
            // Join parts and use qualify_class_name for resolution
            let name = qname.parts.join("\\");
            self.qualify_class_name(&name)
        }
    }

    /// Qualify a class name with the current namespace if needed
    /// - If name starts with \, it's fully qualified (remove the \)
    /// - If name matches a use alias, return the aliased fully qualified name
    /// - If name contains \, it's a qualified name (resolve first segment as alias)
    /// - Otherwise, prefix with current namespace
    fn qualify_class_name(&self, name: &str) -> String {
        if let Some(stripped) = name.strip_prefix('\\') {
            // Fully qualified name - remove leading backslash
            stripped.to_string()
        } else if name.contains('\\') {
            // Qualified name like Foo\Bar - check if first segment is aliased
            let parts: Vec<&str> = name.splitn(2, '\\').collect();
            let first_segment = parts[0];
            let rest = parts.get(1).unwrap_or(&"");

            if let Some(aliased) = self.use_aliases.get(first_segment) {
                // First segment is aliased, combine with rest
                if rest.is_empty() {
                    aliased.clone()
                } else {
                    format!("{}\\{}", aliased, rest)
                }
            } else if let Some(ref ns) = self.current_namespace {
                // Not aliased - prefix with current namespace
                format!("{}\\{}", ns, name)
            } else {
                // No namespace - use as-is
                name.to_string()
            }
        } else if let Some(aliased) = self.use_aliases.get(name) {
            // Simple name matches a use alias
            aliased.clone()
        } else if let Some(ref ns) = self.current_namespace {
            // Relative name - prefix with current namespace
            format!("{}\\{}", ns, name)
        } else {
            // No namespace - use as-is
            name.to_string()
        }
    }
}

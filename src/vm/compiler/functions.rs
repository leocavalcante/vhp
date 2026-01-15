use super::Compiler;

use crate::ast::{Expr, FunctionParam, Stmt};
use crate::vm::opcode::Opcode;
use std::collections::HashSet;
use std::sync::Arc;

impl Compiler {
    pub(crate) fn compile_arrow_function_internal(
        &mut self,
        params: &[FunctionParam],
        body: &Expr,
    ) -> Result<(), String> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static ARROW_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = ARROW_COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!("__arrow_{}", id);

        let param_names: HashSet<_> = params.iter().map(|p| p.name.as_str()).collect();
        let mut captured_vars = Vec::new();
        self.find_captured_vars_internal(body, &param_names, &mut captured_vars);

        for var_name in &captured_vars {
            let var_idx = self.intern_string(var_name.clone());
            self.emit(Opcode::CaptureVar(var_idx));
        }

        let mut closure_compiler = Compiler::new(name.clone());

        for (i, var_name) in captured_vars.iter().enumerate() {
            closure_compiler.locals.insert(var_name.clone(), i as u16);
            closure_compiler.function.local_names.push(var_name.clone());
        }
        closure_compiler.next_local = captured_vars.len() as u16;

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

        let captured_count = captured_vars.len();
        for (i, param) in params.iter().enumerate() {
            if let Some(default_expr) = &param.default {
                let slot = (captured_count + i) as u16;
                closure_compiler.emit(Opcode::LoadFast(slot));
                let skip_jump = closure_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                closure_compiler.emit(Opcode::Pop);
                closure_compiler.compile_expr(default_expr)?;
                closure_compiler.emit(Opcode::StoreFast(slot));
                let end_jump = closure_compiler.emit_jump(Opcode::Jump(0));
                closure_compiler.patch_jump(skip_jump);
                closure_compiler.emit(Opcode::Pop);
                closure_compiler.patch_jump(end_jump);
            }
        }

        closure_compiler.compile_expr(body)?;
        closure_compiler.emit(Opcode::Return);

        for (inner_name, inner_func) in closure_compiler.functions.drain() {
            self.functions.insert(inner_name, inner_func);
        }

        let compiled = Arc::new(closure_compiler.function);
        let func_idx = self.intern_string(name.clone());
        self.functions.insert(name, compiled);

        self.emit(Opcode::CreateClosure(func_idx, captured_vars.len() as u8));

        Ok(())
    }

    pub(crate) fn find_captured_vars_internal(
        &self,
        expr: &Expr,
        param_names: &HashSet<&str>,
        captured: &mut Vec<String>,
    ) {
        match expr {
            Expr::Variable(name) => {
                if !param_names.contains(name.as_str())
                    && !captured.contains(name)
                    && (self.locals.contains_key(name) || self.is_global_var_internal(name))
                {
                    captured.push(name.clone());
                }
            }
            Expr::Binary { left, right, .. } => {
                self.find_captured_vars_internal(left, param_names, captured);
                self.find_captured_vars_internal(right, param_names, captured);
            }
            Expr::Unary { expr, .. } => {
                self.find_captured_vars_internal(expr, param_names, captured);
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.find_captured_vars_internal(condition, param_names, captured);
                self.find_captured_vars_internal(then_expr, param_names, captured);
                self.find_captured_vars_internal(else_expr, param_names, captured);
            }
            Expr::FunctionCall { args, .. } => {
                for arg in args {
                    self.find_captured_vars_internal(&arg.value, param_names, captured);
                }
            }
            Expr::MethodCall { object, args, .. } => {
                self.find_captured_vars_internal(object, param_names, captured);
                for arg in args {
                    self.find_captured_vars_internal(&arg.value, param_names, captured);
                }
            }
            Expr::PropertyAccess { object, .. } => {
                self.find_captured_vars_internal(object, param_names, captured);
            }
            Expr::ArrayAccess { array, index } => {
                self.find_captured_vars_internal(array, param_names, captured);
                self.find_captured_vars_internal(index, param_names, captured);
            }
            Expr::Array(elements) => {
                for elem in elements {
                    if let Some(ref key) = elem.key {
                        self.find_captured_vars_internal(key, param_names, captured);
                    }
                    self.find_captured_vars_internal(&elem.value, param_names, captured);
                }
            }
            Expr::Grouped(inner) => {
                self.find_captured_vars_internal(inner, param_names, captured);
            }
            Expr::Spread(inner) => {
                self.find_captured_vars_internal(inner, param_names, captured);
            }
            Expr::CallableCall { callable, args } => {
                self.find_captured_vars_internal(callable, param_names, captured);
                for arg in args {
                    self.find_captured_vars_internal(&arg.value, param_names, captured);
                }
            }
            Expr::CallableFromMethod { object, .. } => {
                self.find_captured_vars_internal(object, param_names, captured);
            }
            Expr::Assign { value, .. } => {
                self.find_captured_vars_internal(value, param_names, captured);
            }
            Expr::ArrayAssign {
                array,
                index,
                value,
                ..
            } => {
                self.find_captured_vars_internal(array, param_names, captured);
                if let Some(ref idx) = index {
                    self.find_captured_vars_internal(idx, param_names, captured);
                }
                self.find_captured_vars_internal(value, param_names, captured);
            }
            Expr::PropertyAssign { object, value, .. } => {
                self.find_captured_vars_internal(object, param_names, captured);
                self.find_captured_vars_internal(value, param_names, captured);
            }
            Expr::StaticPropertyAssign { value, .. } => {
                self.find_captured_vars_internal(value, param_names, captured);
            }
            Expr::New { args, .. } => {
                for arg in args {
                    self.find_captured_vars_internal(&arg.value, param_names, captured);
                }
            }
            Expr::Throw(inner) => {
                self.find_captured_vars_internal(inner, param_names, captured);
            }
            Expr::Match {
                expr,
                arms,
                default,
            } => {
                self.find_captured_vars_internal(expr, param_names, captured);
                for arm in arms {
                    for condition in &arm.conditions {
                        self.find_captured_vars_internal(condition, param_names, captured);
                    }
                    self.find_captured_vars_internal(&arm.result, param_names, captured);
                }
                if let Some(ref def) = default {
                    self.find_captured_vars_internal(def, param_names, captured);
                }
            }
            Expr::ArrowFunction {
                params: inner_params,
                body,
            } => {
                let mut combined_params: HashSet<&str> = param_names.clone();
                for p in inner_params {
                    combined_params.insert(&p.name);
                }
                self.find_captured_vars_internal(body, &combined_params, captured);
            }
            Expr::Clone { object } => {
                self.find_captured_vars_internal(object, param_names, captured);
            }
            Expr::CloneWith {
                object,
                modifications,
            } => {
                self.find_captured_vars_internal(object, param_names, captured);
                for modification in modifications {
                    self.find_captured_vars_internal(&modification.value, param_names, captured);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn is_global_var_internal(&self, name: &str) -> bool {
        !self.locals.contains_key(name)
    }

    pub(crate) fn compile_function_internal(
        &mut self,
        name: &str,
        params: &[FunctionParam],
        return_type: &Option<crate::ast::TypeHint>,
        body: &[Stmt],
        attributes: &[crate::ast::Attribute],
    ) -> Result<(), String> {
        let mut func_compiler = Compiler::new(name.to_string());

        func_compiler.function.strict_types = self.strict_types;

        // Copy namespace and use aliases from parent compiler
        func_compiler.current_namespace = self.current_namespace.clone();
        func_compiler.use_aliases = self.use_aliases.clone();

        func_compiler.function.parameters = params.to_vec();
        func_compiler.function.attributes = attributes.to_vec();

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
        func_compiler.function.return_type = return_type
            .as_ref()
            .map(|t| func_compiler.resolve_type_hint(t));
        func_compiler.function.is_variadic = params.iter().any(|p| p.is_variadic);

        for param in params {
            func_compiler
                .function
                .param_types
                .push(param.type_hint.clone());
        }

        for (i, param) in params.iter().enumerate() {
            if let Some(default_expr) = &param.default {
                let slot = i as u16;
                func_compiler.emit(Opcode::LoadFast(slot));
                let skip_jump = func_compiler.emit_jump(Opcode::JumpIfNotNull(0));
                func_compiler.emit(Opcode::Pop);
                func_compiler.compile_expr(default_expr)?;
                func_compiler.emit(Opcode::StoreFast(slot));
                let end_jump = func_compiler.emit_jump(Opcode::Jump(0));
                func_compiler.patch_jump(skip_jump);
                func_compiler.emit(Opcode::Pop);
                func_compiler.patch_jump(end_jump);
            }
        }

        for stmt in body {
            func_compiler.compile_stmt(stmt)?;
        }

        func_compiler.function.is_generator = func_compiler.contains_yield(body);

        func_compiler.emit(Opcode::ReturnNull);

        for (inner_name, inner_func) in func_compiler.functions.drain() {
            self.functions.insert(inner_name, inner_func);
        }

        let compiled = Arc::new(func_compiler.function);
        self.functions.insert(name.to_string(), compiled);

        Ok(())
    }

    fn contains_yield(&self, stmts: &[Stmt]) -> bool {
        self.contains_yield_in_stmts(stmts)
    }

    fn contains_yield_in_stmts(&self, stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            if self.contains_yield_in_stmt(stmt) {
                return true;
            }
        }
        false
    }

    fn contains_yield_in_stmt(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expression(expr) => {
                if let Expr::Yield { .. } = expr {
                    true
                } else if let Expr::YieldFrom(_inner) = expr {
                    // yield from itself makes this a generator
                    true
                } else {
                    false
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.contains_yield_in_stmts(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|b| self.contains_yield_in_stmts(b))
            }
            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => {
                self.contains_yield_in_stmts(body)
            }
            Stmt::For { body, .. } | Stmt::Foreach { body, .. } => {
                self.contains_yield_in_stmts(body)
            }
            Stmt::Switch { cases, default, .. } => {
                cases.iter().any(|c| self.contains_yield_in_stmts(&c.body))
                    || default
                        .as_ref()
                        .is_some_and(|d| self.contains_yield_in_stmts(d))
            }
            _ => false,
        }
    }

    #[allow(dead_code)]
    fn contains_yield_in_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Yield { .. } => true,
            Expr::YieldFrom(inner) => self.contains_yield_in_expr(inner),
            Expr::Binary { left, right, .. } => {
                self.contains_yield_in_expr(left) || self.contains_yield_in_expr(right)
            }
            Expr::Unary { expr, .. } => self.contains_yield_in_expr(expr),
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                self.contains_yield_in_expr(condition)
                    || self.contains_yield_in_expr(then_expr)
                    || self.contains_yield_in_expr(else_expr)
            }
            Expr::FunctionCall { args, .. } => {
                args.iter().any(|a| self.contains_yield_in_expr(&a.value))
            }
            Expr::MethodCall { object, args, .. } => {
                self.contains_yield_in_expr(object)
                    || args.iter().any(|a| self.contains_yield_in_expr(&a.value))
            }
            Expr::PropertyAccess { object, .. } => self.contains_yield_in_expr(object),
            Expr::ArrayAccess { array, index } => {
                self.contains_yield_in_expr(array) || self.contains_yield_in_expr(index)
            }
            Expr::Array(elements) => {
                let key_has_yield = elements.iter().any(|e| {
                    if let Some(ref k) = e.key {
                        self.contains_yield_in_expr(k)
                    } else {
                        false
                    }
                });
                let value_has_yield = elements
                    .iter()
                    .any(|e| self.contains_yield_in_expr(&e.value));
                key_has_yield || value_has_yield
            }
            Expr::Grouped(inner)
            | Expr::Spread(inner)
            | Expr::ArrowFunction { body: inner, .. } => self.contains_yield_in_expr(inner),
            Expr::CallableCall { callable, args } => {
                self.contains_yield_in_expr(callable)
                    || args.iter().any(|a| self.contains_yield_in_expr(&a.value))
            }
            Expr::CallableFromMethod { object, .. } => self.contains_yield_in_expr(object),
            Expr::New { args, .. } => args.iter().any(|a| self.contains_yield_in_expr(&a.value)),
            Expr::Throw(expr) => self.contains_yield_in_expr(expr),
            Expr::Match {
                expr,
                arms,
                default,
            } => {
                self.contains_yield_in_expr(expr)
                    || arms.iter().any(|arm| {
                        arm.conditions
                            .iter()
                            .any(|c| self.contains_yield_in_expr(c))
                            || self.contains_yield_in_expr(&arm.result)
                    })
                    || match default.as_ref() {
                        Some(d) => self.contains_yield_in_expr(d),
                        None => false,
                    }
            }
            _ => false,
        }
    }
}

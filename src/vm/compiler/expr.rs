use super::Compiler;

use crate::ast::Expr;
use crate::vm::opcode::Opcode;

impl Compiler {
    pub(crate) fn compile_expr_internal(&mut self, expr: &Expr) -> Result<(), String> {
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
            Expr::Heredoc(content) => {
                self.compile_heredoc(content)?;
            }
            Expr::Variable(name) => {
                if let Some(&slot) = self.locals.get(name) {
                    self.emit(Opcode::LoadFast(slot));
                } else {
                    let idx = self.intern_string(name.clone());
                    self.emit(Opcode::LoadVar(idx));
                }
            }
            Expr::Assign { var, op, value } => {
                self.compile_assign(var, op, value)?;
            }
            Expr::Binary { left, op, right } => {
                self.compile_binary_op(left, op, right)?;
            }
            Expr::Unary { op, expr } => {
                self.compile_unary_op(op, expr)?;
            }
            Expr::Array(elements) => {
                self.compile_array_literal(elements)?;
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
                self.compile_function_call(name, args)?;
            }
            Expr::ArrayAssign {
                array,
                index,
                op,
                value,
            } => {
                self.compile_array_assign(array, index, op, value)?;
            }
            Expr::Grouped(inner) => {
                self.compile_expr(inner)?;
            }
            Expr::New { class_name, args } => {
                self.compile_new_object(class_name, args)?;
            }
            Expr::PropertyAccess { object, property } => {
                self.compile_property_access(object, property)?;
            }
            Expr::PropertyAssign {
                object,
                property,
                value,
            } => {
                self.compile_property_assign(object, property, value)?;
            }
            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                self.compile_method_call(object, method, args)?;
            }
            Expr::StaticMethodCall {
                class_name,
                method,
                args,
            } => {
                self.compile_static_method_call(class_name, method, args)?;
            }
            Expr::StaticPropertyAccess { class, property } => {
                self.compile_static_property_access(class, property)?;
            }
            Expr::StaticPropertyAssign {
                class,
                property,
                value,
            } => {
                self.compile_static_property_assign(class, property, value)?;
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
                if let Some(k) = key {
                    self.compile_expr(k)?;
                }
                if let Some(v) = value {
                    self.compile_expr(v)?;
                }
                self.emit(Opcode::Yield);
            }
            Expr::YieldFrom(inner) => {
                self.compile_expr(inner)?;
                self.emit(Opcode::YieldFrom);
            }
            Expr::Spread(inner) => {
                self.compile_expr(inner)?;
                self.emit(Opcode::ArrayUnpack);
            }
            Expr::CallableCall { callable, args } => {
                for arg in args {
                    self.compile_expr(&arg.value)?;
                }
                self.compile_expr(callable)?;
                self.emit(Opcode::CallCallable(args.len() as u8));
            }
            Expr::CallableFromFunction(name) => {
                let name_idx = self.intern_string(name.clone());
                self.emit(Opcode::PushString(name_idx));
            }
            Expr::CallableFromMethod { object, method } => {
                self.compile_expr(object)?;
                let method_idx = self.intern_string(method.clone());
                self.emit(Opcode::PushString(method_idx));
                self.emit(Opcode::CreateMethodClosure);
            }
            Expr::CallableFromStaticMethod { class, method } => {
                let class_idx = self.intern_string(class.clone());
                let method_idx = self.intern_string(method.clone());
                self.emit(Opcode::PushString(class_idx));
                self.emit(Opcode::PushString(method_idx));
                self.emit(Opcode::CreateStaticMethodClosure);
            }
            Expr::NewAnonymousClass {
                constructor_args,
                parent,
                interfaces: _,
                traits: _,
                properties,
                methods,
            } => {
                self.compile_anonymous_class(constructor_args, parent, properties, methods)?;
            }
            Expr::NewFiber { callback } => {
                self.compile_expr(callback)?;
                self.emit(Opcode::NewFiber);
            }
            Expr::FiberSuspend { value } => {
                if let Some(v) = value {
                    self.compile_expr(v)?;
                } else {
                    self.emit(Opcode::PushNull);
                }
            }
            Expr::FiberGetCurrent => {
                self.emit(Opcode::GetCurrentFiber);
            }
            Expr::CloneWith {
                object,
                modifications,
            } => {
                self.compile_clone_with(object, modifications)?;
            }
            Expr::Placeholder => {
                return Err("Pipe placeholder not yet implemented".to_string());
            }
            // Magic constants
            Expr::MagicFile => {
                // __FILE__ - Full path of the file being executed
                let file_path = self.file_path();
                let idx = self.intern_string(file_path);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicLine(line) => {
                // __LINE__ - Current line number (1-based)
                let idx = self.intern_string(line.to_string());
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicDir => {
                // __DIR__ - Directory of the file being executed
                let dir_path = self.dir_path();
                let idx = self.intern_string(dir_path);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicFunction => {
                // __FUNCTION__ - Current function name (or empty at top level)
                let func_name = self.function_name();
                let idx = self.intern_string(func_name);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicClass => {
                // __CLASS__ - Current class name (or empty at top level)
                let class_name = self.class_name();
                let idx = self.intern_string(class_name);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicMethod => {
                // __METHOD__ - Current method name with class (or empty at top level)
                let method_name = self.method_name();
                let idx = self.intern_string(method_name);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicNamespace => {
                // __NAMESPACE__ - Current namespace (or empty if no namespace)
                let namespace = self.namespace();
                let idx = self.intern_string(namespace);
                self.emit(Opcode::PushString(idx));
            }
            Expr::MagicTrait => {
                let trait_name = self.trait_name();
                let idx = self.intern_string(trait_name);
                self.emit(Opcode::PushString(idx));
            }
            Expr::ListDestructure { elements, array } => {
                self.compile_list_destructure(elements, array)?;
            }
        }
        Ok(())
    }

    /// Compile heredoc string with variable interpolation
    fn compile_heredoc(&mut self, content: &str) -> Result<(), String> {
        let parts: Vec<&str> = content.split("\x00").collect();

        if parts.len() == 1 {
            let idx = self.intern_string(content.to_string());
            self.emit(Opcode::PushString(idx));
        } else {
            let var_count = (parts.len() - 1) / 2;
            let mut var_placeholders = Vec::new();

            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 1 {
                    let var_str = *part;
                    if var_str.starts_with('$') {
                        var_placeholders.push(var_str[1..].to_string());
                    }
                }
            }

            let mut var_idx = 0;
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 0 {
                    if !part.is_empty() {
                        let idx = self.intern_string(part.to_string());
                        self.emit(Opcode::PushString(idx));
                    }
                } else if var_idx < var_placeholders.len() {
                    let var_name = &var_placeholders[var_idx];
                    var_idx += 1;
                    if let Some(&slot) = self.locals.get(var_name) {
                        self.emit(Opcode::LoadFast(slot));
                    } else {
                        let idx = self.intern_string(var_name.clone());
                        self.emit(Opcode::LoadVar(idx));
                    }
                }
            }

            self.emit(Opcode::HeredocInterpolate(var_count as u16));
        }
        Ok(())
    }
}

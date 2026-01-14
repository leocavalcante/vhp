use super::Compiler;

use crate::ast::Expr;
use crate::vm::class::{CompiledClass, CompiledProperty};
use crate::vm::opcode::Opcode;
use std::sync::Arc;

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
            Expr::Variable(name) => {
                if let Some(&slot) = self.locals.get(name) {
                    self.emit(Opcode::LoadFast(slot));
                } else {
                    let idx = self.intern_string(name.clone());
                    self.emit(Opcode::LoadVar(idx));
                }
            }
            Expr::Assign { var, op, value } => {
                use crate::ast::AssignOp;

                if *op != AssignOp::Assign {
                    if let Some(&slot) = self.locals.get(var) {
                        self.emit(Opcode::LoadFast(slot));
                    } else {
                        let idx = self.intern_string(var.clone());
                        self.emit(Opcode::LoadVar(idx));
                    }
                }

                self.compile_expr(value)?;

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

                if !self.locals.contains_key(var) {
                    let slot = self.next_local;
                    self.locals.insert(var.clone(), slot);
                    self.next_local += 1;
                    self.function.local_count = self.next_local;
                    self.function.local_names.push(var.clone());
                }

                if let Some(&slot) = self.locals.get(var) {
                    self.emit(Opcode::Dup);
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
                let count = elements.len();
                for elem in elements {
                    if let Some(key_expr) = &elem.key {
                        self.compile_expr(key_expr)?;
                    } else {
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
                if name.to_lowercase() == "unset" {
                    for arg in args {
                        match arg.value.as_ref() {
                            Expr::PropertyAccess { object, property } => {
                                let prop_idx = self.intern_string(property.clone());

                                if let Expr::Variable(var_name) = object.as_ref() {
                                    if let Some(&slot) = self.locals.get(var_name) {
                                        self.emit(Opcode::UnsetPropertyOnLocal(slot, prop_idx));
                                    } else {
                                        let var_idx = self.intern_string(var_name.clone());
                                        self.emit(Opcode::UnsetPropertyOnGlobal(var_idx, prop_idx));
                                    }
                                } else {
                                    self.compile_expr(object)?;
                                    self.emit(Opcode::UnsetProperty(prop_idx));
                                }
                            }
                            Expr::Variable(var_name) => {
                                if let Some(&slot) = self.locals.get(var_name) {
                                    self.emit(Opcode::PushNull);
                                    self.emit(Opcode::StoreFast(slot));
                                } else {
                                    let idx = self.intern_string(var_name.clone());
                                    self.emit(Opcode::UnsetVar(idx));
                                }
                            }
                            Expr::ArrayAccess { array, index } => {
                                self.compile_expr(array)?;
                                self.compile_expr(index)?;
                                self.emit(Opcode::UnsetArrayElement);
                            }
                            _ => {
                                return Err(format!("Cannot unset expression: {:?}", arg.value));
                            }
                        }
                    }
                    self.emit(Opcode::PushNull);
                    return Ok(());
                }

                if name.to_lowercase() == "isset" && args.len() == 1 {
                    if let Expr::PropertyAccess { object, property } = args[0].value.as_ref() {
                        let prop_idx = self.intern_string(property.clone());

                        if let Expr::Variable(var_name) = object.as_ref() {
                            if let Some(&slot) = self.locals.get(var_name) {
                                self.emit(Opcode::IssetPropertyOnLocal(slot, prop_idx));
                            } else {
                                let var_idx = self.intern_string(var_name.clone());
                                self.emit(Opcode::IssetPropertyOnGlobal(var_idx, prop_idx));
                            }
                        } else {
                            self.compile_expr(object)?;
                            self.emit(Opcode::IssetProperty(prop_idx));
                        }
                        return Ok(());
                    }
                }

                let has_spread = args
                    .iter()
                    .any(|arg| matches!(arg.value.as_ref(), Expr::Spread(_)));
                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_spread {
                    self.emit(Opcode::NewArray(0));

                    for arg in args {
                        match arg.value.as_ref() {
                            Expr::Spread(inner) => {
                                self.compile_expr(inner)?;
                                self.emit(Opcode::ArrayMerge);
                            }
                            _ => {
                                self.compile_expr(&arg.value)?;
                                self.emit(Opcode::ArrayAppend);
                            }
                        }
                    }

                    let name_idx = self.intern_string(name.clone());
                    self.emit(Opcode::CallSpread(name_idx));
                } else if has_named {
                    let mut total_pairs = 0;

                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                        total_pairs += 1;
                    }

                    self.emit(Opcode::NewArray(total_pairs as u16));

                    let name_idx = self.intern_string(name.clone());
                    self.emit(Opcode::CallNamed(name_idx));
                } else {
                    for arg in args {
                        self.compile_expr(&arg.value)?;
                    }
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
                use crate::ast::AssignOp;

                if *op != AssignOp::Assign {
                    return Err("Compound array assignment not yet implemented".to_string());
                }

                match array.as_ref() {
                    Expr::Variable(var_name) => {
                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::LoadFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::LoadVar(idx));
                        }

                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        self.compile_expr(value)?;

                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            self.emit(Opcode::Swap);
                            self.emit(Opcode::Pop);
                            self.emit(Opcode::ArrayAppend);
                        }

                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::Dup);
                            self.emit(Opcode::StoreFast(slot));
                        } else {
                            let idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::Dup);
                            self.emit(Opcode::StoreVar(idx));
                        }
                    }
                    Expr::PropertyAccess { object, property } => {
                        let is_this = matches!(object.as_ref(), Expr::This);

                        self.compile_expr(object)?;
                        let prop_idx = self.intern_string(property.clone());
                        self.emit(Opcode::LoadProperty(prop_idx));

                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        self.compile_expr(value)?;

                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            self.emit(Opcode::Swap);
                            self.emit(Opcode::Pop);
                            self.emit(Opcode::ArrayAppend);
                        }

                        if is_this {
                            self.emit(Opcode::StoreThisProperty(prop_idx));
                        } else {
                            self.compile_expr(object)?;
                            self.emit(Opcode::Swap);
                            self.emit(Opcode::StoreProperty(prop_idx));

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
                        let class_idx = self.intern_string(class.clone());
                        let prop_idx = self.intern_string(property.clone());
                        self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                        if let Some(key_expr) = index {
                            self.compile_expr(key_expr)?;
                        } else {
                            self.emit(Opcode::PushNull);
                        }

                        self.compile_expr(value)?;

                        if index.is_some() {
                            self.emit(Opcode::ArraySet);
                        } else {
                            self.emit(Opcode::Swap);
                            self.emit(Opcode::Pop);
                            self.emit(Opcode::ArrayAppend);
                        }

                        self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                    }
                    _ => return Err("Complex array assignment not yet implemented".to_string()),
                }
            }
            Expr::Grouped(inner) => {
                self.compile_expr(inner)?;
            }
            Expr::New { class_name, args } => {
                let qualified_name = self.qualify_class_name(class_name);
                let class_idx = self.intern_string(qualified_name);
                self.emit(Opcode::NewObject(class_idx));

                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_named {
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                    }
                    self.emit(Opcode::NewArray(args.len() as u16));
                    self.emit(Opcode::CallConstructorNamed);
                } else {
                    for arg in args {
                        self.compile_expr(&arg.value)?;
                    }
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
                if matches!(object.as_ref(), Expr::This) {
                    self.compile_expr(value)?;
                    let prop_idx = self.intern_string(property.clone());
                    self.emit(Opcode::StoreThisProperty(prop_idx));
                } else if let Expr::Variable(var_name) = object.as_ref() {
                    self.compile_expr(object)?;
                    self.compile_expr(value)?;
                    let prop_idx = self.intern_string(property.clone());
                    self.emit(Opcode::StoreProperty(prop_idx));
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

                match object.as_ref() {
                    Expr::Variable(var_name) => {
                        for arg in args {
                            self.compile_expr(&arg.value)?;
                        }

                        if let Some(&slot) = self.locals.get(var_name) {
                            self.emit(Opcode::CallMethodOnLocal(
                                slot,
                                method_idx,
                                args.len() as u8,
                            ));
                        } else {
                            let var_idx = self.intern_string(var_name.clone());
                            self.emit(Opcode::CallMethodOnGlobal(
                                var_idx,
                                method_idx,
                                args.len() as u8,
                            ));
                        }
                    }
                    _ => {
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
                let has_named = args.iter().any(|arg| arg.name.is_some());

                if has_named {
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some(ref param_name) = arg.name {
                            let name_idx = self.intern_string(param_name.clone());
                            self.emit(Opcode::PushString(name_idx));
                            self.compile_expr(&arg.value)?;
                        } else {
                            self.emit(Opcode::PushInt(idx as i64));
                            self.compile_expr(&arg.value)?;
                        }
                    }
                    self.emit(Opcode::NewArray(args.len() as u16));
                    let class_idx = self.intern_string(class_name.clone());
                    let method_idx = self.intern_string(method.clone());
                    self.emit(Opcode::CallStaticMethodNamed(class_idx, method_idx));
                } else {
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
            }
            Expr::CallableFromStaticMethod { class, method } => {
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
                let anon_name = format!("__anon_class_{}", self.classes.len());
                let mut anon_class = CompiledClass::new(anon_name.clone());

                anon_class.parent = parent.clone();

                for prop in properties {
                    let compiled_prop = CompiledProperty::from_ast(prop, false);
                    anon_class.properties.push(compiled_prop);
                }

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

                    method_compiler.function.parameters = method.params.clone();
                    method_compiler.function.attributes = method.attributes.clone();

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

                    for (inner_name, inner_func) in method_compiler.functions.drain() {
                        self.functions.insert(inner_name, inner_func);
                    }

                    let compiled = Arc::new(method_compiler.function);
                    anon_class.methods.insert(method.name.clone(), compiled);
                }

                self.classes.insert(anon_name.clone(), Arc::new(anon_class));

                let class_idx = self.intern_string(anon_name);
                self.emit(Opcode::NewObject(class_idx));

                let arg_count = constructor_args.len();
                for arg in constructor_args {
                    self.compile_expr(&arg.value)?;
                }

                self.emit(Opcode::CallConstructor(arg_count as u8));
            }
            Expr::NewFiber { callback } => {
                self.compile_expr(callback)?;
                self.emit(Opcode::NewFiber);
                // Don't call constructor - callback is already stored in NewFiber
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
                self.compile_expr(object)?;
                self.emit(Opcode::Clone);

                for modification in modifications {
                    self.compile_expr(&modification.value)?;
                    let prop_idx = self.intern_string(modification.property.clone());
                    self.emit(Opcode::StoreCloneProperty(prop_idx));
                }
            }
            Expr::Placeholder => {
                return Err("Pipe placeholder not yet implemented".to_string());
            }
        }
        Ok(())
    }
}

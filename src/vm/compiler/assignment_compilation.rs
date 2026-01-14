use super::Compiler;

use crate::ast::{Argument, Expr};
use crate::vm::opcode::Opcode;

impl Compiler {
    pub(crate) fn compile_assign(
        &mut self,
        var: &str,
        op: &crate::ast::AssignOp,
        value: &Expr,
    ) -> Result<(), String> {
        use crate::ast::AssignOp;

        if *op != AssignOp::Assign {
            if let Some(&slot) = self.locals.get(var) {
                self.emit(Opcode::LoadFast(slot));
            } else {
                let idx = self.intern_string(var.to_string());
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
            self.locals.insert(var.to_string(), slot);
            self.next_local += 1;
            self.function.local_count = self.next_local;
            self.function.local_names.push(var.to_string());
        }

        if let Some(&slot) = self.locals.get(var) {
            self.emit(Opcode::Dup);
            self.emit(Opcode::StoreFast(slot));
        } else {
            let idx = self.intern_string(var.to_string());
            self.emit(Opcode::Dup);
            self.emit(Opcode::StoreVar(idx));
        }

        Ok(())
    }

    pub(crate) fn compile_array_literal(
        &mut self,
        elements: &[crate::ast::ArrayElement],
    ) -> Result<(), String> {
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
        Ok(())
    }

    pub(crate) fn compile_function_call(
        &mut self,
        name: &str,
        args: &[Argument],
    ) -> Result<(), String> {
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

            let name_idx = self.intern_string(name.to_string());
            self.emit(Opcode::CallSpread(name_idx));
        } else if has_named {
            let mut total_pairs = 0;

            for (idx, arg) in args.iter().enumerate() {
                if let Some(ref param_name) = arg.name {
                    let name_idx = self.intern_string(param_name.to_string());
                    self.emit(Opcode::PushString(name_idx));
                    self.compile_expr(&arg.value)?;
                } else {
                    self.emit(Opcode::PushInt(idx as i64));
                    self.compile_expr(&arg.value)?;
                }
                total_pairs += 1;
            }

            self.emit(Opcode::NewArray(total_pairs as u16));

            let name_idx = self.intern_string(name.to_string());
            self.emit(Opcode::CallNamed(name_idx));
        } else {
            for arg in args {
                self.compile_expr(&arg.value)?;
            }
            let name_idx = self.intern_string(name.to_string());
            self.emit(Opcode::Call(name_idx, args.len() as u8));
        }

        Ok(())
    }

    pub(crate) fn compile_array_assign(
        &mut self,
        array: &Expr,
        index: &Option<Box<Expr>>,
        op: &crate::ast::AssignOp,
        value: &Expr,
    ) -> Result<(), String> {
        use crate::ast::AssignOp;

        if *op != AssignOp::Assign {
            return Err("Compound array assignment not yet implemented".to_string());
        }

        match array {
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

        Ok(())
    }
}

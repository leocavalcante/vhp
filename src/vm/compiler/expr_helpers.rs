use super::Compiler;
use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::vm::opcode::Opcode;

impl Compiler {
    /// Compile a binary operation
    pub fn compile_binary_op_internal(
        &mut self,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<(), String> {
        match op {
            BinaryOp::And => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let short_circuit = self.emit_jump(Opcode::JumpIfFalse(0));
                self.emit(Opcode::Pop);
                self.compile_expr(right)?;
                self.patch_jump(short_circuit);
                return Ok(());
            }
            BinaryOp::Or => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let short_circuit = self.emit_jump(Opcode::JumpIfTrue(0));
                self.emit(Opcode::Pop);
                self.compile_expr(right)?;
                self.patch_jump(short_circuit);
                return Ok(());
            }
            BinaryOp::NullCoalesce => {
                self.compile_expr(left)?;
                self.emit(Opcode::Dup);
                let jump_if_not_null = self.emit_jump(Opcode::JumpIfNotNull(0));
                self.emit(Opcode::Pop);
                self.compile_expr(right)?;
                self.patch_jump(jump_if_not_null);
                return Ok(());
            }
            BinaryOp::Pipe => {
                match right {
                    Expr::FunctionCall { name, args } => {
                        let placeholder_pos = args
                            .iter()
                            .position(|arg| matches!(&*arg.value, Expr::Placeholder));

                        if let Some(pos) = placeholder_pos {
                            for (i, arg) in args.iter().enumerate() {
                                if i == pos {
                                    self.compile_expr(left)?;
                                } else {
                                    self.compile_expr(&arg.value)?;
                                }
                            }
                            let func_idx = self.intern_string(name.clone());
                            self.emit(Opcode::Call(func_idx, args.len() as u8));
                        } else {
                            self.compile_expr(left)?;
                            for arg in args {
                                self.compile_expr(&arg.value)?;
                            }
                            let func_idx = self.intern_string(name.clone());
                            self.emit(Opcode::Call(func_idx, (1 + args.len()) as u8));
                        }
                    }
                    Expr::CallableFromFunction(func_name) => {
                        use crate::vm::builtins;
                        let is_builtin = builtins::is_builtin(&func_name);
                        let func_idx = self.intern_string(func_name.clone());
                        self.compile_expr(left)?;
                        self.emit(if is_builtin {
                            Opcode::CallBuiltin(func_idx, 1)
                        } else {
                            Opcode::Call(func_idx, 1)
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

        self.compile_expr(left)?;
        self.compile_expr(right)?;

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
            BinaryOp::And => self.emit(Opcode::And),
            BinaryOp::Or => self.emit(Opcode::Or),
            BinaryOp::Xor => self.emit(Opcode::Xor),
            BinaryOp::NullCoalesce => unreachable!("Handled above"),
            _ => return Err(format!("Binary operator not yet implemented: {:?}", op)),
        };

        Ok(())
    }

    /// Compile a unary operation
    pub fn compile_unary_op_internal(
        &mut self,
        op: &UnaryOp,
        operand: &Expr,
    ) -> Result<(), String> {
        match op {
            UnaryOp::Not => {
                self.compile_expr(operand)?;
                self.emit(Opcode::Not);
            }
            UnaryOp::Neg => {
                self.compile_expr(operand)?;
                self.emit(Opcode::Neg);
            }
            UnaryOp::PreInc | UnaryOp::PreDec => match operand {
                Expr::Variable(var_name) => {
                    if let Some(&slot) = self.locals.get(var_name) {
                        self.emit(Opcode::LoadFast(slot));
                    } else {
                        let idx = self.intern_string(var_name.clone());
                        self.emit(Opcode::LoadVar(idx));
                    }

                    self.emit(Opcode::PushInt(1));
                    if matches!(op, UnaryOp::PreInc) {
                        self.emit(Opcode::Add);
                    } else {
                        self.emit(Opcode::Sub);
                    }

                    self.emit(Opcode::Dup);

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

                    self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                    self.emit(Opcode::PushInt(1));
                    if matches!(op, UnaryOp::PreInc) {
                        self.emit(Opcode::Add);
                    } else {
                        self.emit(Opcode::Sub);
                    }

                    self.emit(Opcode::Dup);

                    self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                    self.emit(Opcode::Pop);
                }
                _ => return Err("Increment/decrement requires a variable".to_string()),
            },
            UnaryOp::PostInc | UnaryOp::PostDec => match operand {
                Expr::Variable(var_name) => {
                    if let Some(&slot) = self.locals.get(var_name) {
                        self.emit(Opcode::LoadFast(slot));
                    } else {
                        let idx = self.intern_string(var_name.clone());
                        self.emit(Opcode::LoadVar(idx));
                    }

                    self.emit(Opcode::Dup);

                    self.emit(Opcode::PushInt(1));
                    if matches!(op, UnaryOp::PostInc) {
                        self.emit(Opcode::Add);
                    } else {
                        self.emit(Opcode::Sub);
                    }

                    if let Some(&slot) = self.locals.get(var_name) {
                        self.emit(Opcode::StoreFast(slot));
                    } else {
                        let idx = self.intern_string(var_name.clone());
                        self.emit(Opcode::StoreVar(idx));
                    }
                    self.emit(Opcode::Pop);
                }
                Expr::StaticPropertyAccess { class, property } => {
                    let class_idx = self.intern_string(class.clone());
                    let prop_idx = self.intern_string(property.clone());

                    self.emit(Opcode::LoadStaticProp(class_idx, prop_idx));

                    self.emit(Opcode::Dup);

                    self.emit(Opcode::PushInt(1));
                    if matches!(op, UnaryOp::PostInc) {
                        self.emit(Opcode::Add);
                    } else {
                        self.emit(Opcode::Sub);
                    }

                    self.emit(Opcode::StoreStaticProp(class_idx, prop_idx));
                    self.emit(Opcode::Pop);
                }
                _ => return Err("Increment/decrement requires a variable".to_string()),
            },
        };

        Ok(())
    }

    /// Compile a ternary expression
    pub fn compile_ternary_internal(
        &mut self,
        condition: &Expr,
        then_val: &Expr,
        else_val: &Expr,
    ) -> Result<(), String> {
        self.compile_expr(condition)?;

        let else_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        self.compile_expr(then_val)?;

        let end_jump = self.emit_jump(Opcode::Jump(0));

        self.patch_jump(else_jump);

        self.compile_expr(else_val)?;

        self.patch_jump(end_jump);

        Ok(())
    }
}

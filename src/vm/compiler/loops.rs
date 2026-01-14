use super::Compiler;
use crate::ast::{Expr, Stmt};
use crate::vm::opcode::Opcode;

impl Compiler {
    pub(crate) fn compile_while_internal(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
    ) -> Result<(), String> {
        let loop_start = self.current_offset();

        self.compile_expr(condition)?;

        let exit_jump = self.emit_jump(Opcode::JumpIfFalse(0));

        let loop_start_idx = self.emit(Opcode::LoopStart(loop_start as u32, 0));

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Opcode::LoopEnd);

        self.emit_loop(loop_start);

        let loop_end = self.current_offset();
        self.patch_jump(exit_jump);

        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    pub(crate) fn compile_do_while_internal(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
    ) -> Result<(), String> {
        let loop_start = self.current_offset();

        let loop_start_idx = self.emit(Opcode::LoopStart(loop_start as u32, 0));

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Opcode::LoopEnd);

        self.compile_expr(condition)?;

        self.emit(Opcode::JumpIfTrue(loop_start as u32));

        let loop_end = self.current_offset();
        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    pub(crate) fn compile_for_internal(
        &mut self,
        init: &Option<Expr>,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
    ) -> Result<(), String> {
        if let Some(init_expr) = init {
            self.compile_expr(init_expr)?;
            self.emit(Opcode::Pop);
        }

        let loop_start = self.current_offset();

        let exit_jump = if let Some(cond_expr) = condition {
            self.compile_expr(cond_expr)?;
            Some(self.emit_jump(Opcode::JumpIfFalse(0)))
        } else {
            None
        };

        let loop_start_idx = self.emit(Opcode::LoopStart(0, 0));

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Opcode::LoopEnd);

        let update_offset = self.current_offset();

        if let Some(update_expr) = update {
            self.compile_expr(update_expr)?;
            self.emit(Opcode::Pop);
        }

        self.emit_loop(loop_start);

        let loop_end = self.current_offset();
        if let Some(exit) = exit_jump {
            self.patch_jump(exit);
        }

        if let Opcode::LoopStart(ref mut continue_target, ref mut break_target) =
            self.function.bytecode[loop_start_idx]
        {
            *continue_target = update_offset as u32;
            *break_target = loop_end as u32;
        }

        Ok(())
    }

    pub(crate) fn compile_foreach_internal(
        &mut self,
        array: &Expr,
        key: &Option<String>,
        value: &str,
        body: &[Stmt],
    ) -> Result<(), String> {
        self.compile_expr(array)?;
        let array_slot = self.allocate_local("__foreach_array__".to_string());
        self.emit(Opcode::StoreFast(array_slot));

        let iter_slot = self.allocate_local("__foreach_iter__".to_string());

        self.emit(Opcode::PushInt(0));
        self.emit(Opcode::StoreFast(iter_slot));

        let key_slot = key
            .as_ref()
            .map(|key_name| self.allocate_local(key_name.clone()));
        let value_slot = self.allocate_local(value.to_string());

        let loop_check = self.current_offset();

        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::ArrayCount);

        self.emit(Opcode::LoadFast(iter_slot));

        self.emit(Opcode::Le);
        let exit_jump = self.emit_jump(Opcode::JumpIfTrue(0));

        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::ArrayGetKeyAt);

        if let Some(slot) = key_slot {
            self.emit(Opcode::StoreFast(slot));
        } else {
            self.emit(Opcode::Pop);
        }

        self.emit(Opcode::LoadFast(array_slot));
        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::ArrayGetValueAt);

        self.emit(Opcode::StoreFast(value_slot));

        let loop_start_idx = self.emit(Opcode::LoopStart(loop_check as u32, 0));

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        self.emit(Opcode::LoopEnd);

        self.emit(Opcode::LoadFast(iter_slot));
        self.emit(Opcode::PushInt(1));
        self.emit(Opcode::Add);
        self.emit(Opcode::StoreFast(iter_slot));

        self.emit_loop(loop_check);

        let loop_end = self.current_offset();
        self.patch_jump(exit_jump);

        if let Opcode::LoopStart(_, ref mut break_target) = self.function.bytecode[loop_start_idx] {
            *break_target = loop_end as u32;
        }

        Ok(())
    }
}

use super::Compiler;

impl Compiler {
    pub(crate) fn compile_try_catch_internal(
        &mut self,
        try_body: &[crate::ast::Stmt],
        catch_clauses: &[crate::ast::CatchClause],
        finally_body: &Option<Vec<crate::ast::Stmt>>,
    ) -> Result<(), String> {
        let try_start = self.emit_jump(crate::vm::opcode::Opcode::TryStart(0, 0));

        for stmt in try_body {
            self.compile_stmt(stmt)?;
        }

        self.emit(crate::vm::opcode::Opcode::TryEnd);

        let skip_catch = self.emit_jump(crate::vm::opcode::Opcode::Jump(0));

        let catch_offset = self.current_offset() as u32;
        if let crate::vm::opcode::Opcode::TryStart(ref mut c, _) = self.function.bytecode[try_start]
        {
            *c = catch_offset;
        }

        let mut end_catch_jumps = Vec::new();
        for (i, catch) in catch_clauses.iter().enumerate() {
            let var_slot = self.allocate_local(catch.variable.clone());

            self.emit(crate::vm::opcode::Opcode::StoreFast(var_slot));

            for stmt in &catch.body {
                self.compile_stmt(stmt)?;
            }

            if i < catch_clauses.len() - 1 {
                let jump_to_end = self.emit_jump(crate::vm::opcode::Opcode::Jump(0));
                end_catch_jumps.push(jump_to_end);
            }
        }

        self.patch_jump(skip_catch);
        for jump in end_catch_jumps {
            self.patch_jump(jump);
        }

        if let Some(finally) = finally_body {
            let finally_offset = self.current_offset() as u32;
            if let crate::vm::opcode::Opcode::TryStart(_, ref mut f) =
                self.function.bytecode[try_start]
            {
                *f = finally_offset;
            }

            self.emit(crate::vm::opcode::Opcode::FinallyStart);
            for stmt in finally {
                self.compile_stmt(stmt)?;
            }
            self.emit(crate::vm::opcode::Opcode::FinallyEnd);
        }

        Ok(())
    }
}

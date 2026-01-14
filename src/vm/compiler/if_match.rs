use super::Compiler;

impl Compiler {
    /// Compile an if statement with elseif and else branches (internal implementation)
    pub(crate) fn compile_if_internal(
        &mut self,
        condition: &crate::ast::Expr,
        then_branch: &[crate::ast::Stmt],
        elseif_branches: &[(crate::ast::Expr, Vec<crate::ast::Stmt>)],
        else_branch: &Option<Vec<crate::ast::Stmt>>,
    ) -> Result<(), String> {
        // Compile condition
        self.compile_expr(condition)?;

        // Jump to first elseif/else if condition is false
        let then_jump = self.emit_jump(crate::vm::opcode::Opcode::JumpIfFalse(0));

        // Compile then branch
        for stmt in then_branch {
            self.compile_stmt(stmt)?;
        }

        // Jump past all elseif/else branches
        let end_jump = self.emit_jump(crate::vm::opcode::Opcode::Jump(0));

        // Patch then_jump to point here
        self.patch_jump(then_jump);

        // Compile elseif branches
        let mut elseif_jumps = Vec::new();
        for (elseif_condition, elseif_body) in elseif_branches {
            self.compile_expr(elseif_condition)?;
            let elseif_jump = self.emit_jump(crate::vm::opcode::Opcode::JumpIfFalse(0));

            for stmt in elseif_body {
                self.compile_stmt(stmt)?;
            }

            elseif_jumps.push(self.emit_jump(crate::vm::opcode::Opcode::Jump(0)));
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

    /// Compile a match expression (internal implementation)
    pub(crate) fn compile_match_internal(
        &mut self,
        expr: &crate::ast::Expr,
        arms: &[crate::ast::MatchArm],
        default: &Option<Box<crate::ast::Expr>>,
    ) -> Result<(), String> {
        // Compile the subject expression and store it
        self.compile_expr(expr)?;
        let subject_slot = self.allocate_local("__match_subject__".to_string());
        self.emit(crate::vm::opcode::Opcode::StoreFast(subject_slot));

        let mut end_jumps = Vec::new();

        // For each arm, check conditions and emit result
        for arm in arms {
            let mut arm_jumps = Vec::new();

            // Check each condition in this arm (can have multiple: 1, 2, 3 =>)
            for condition in &arm.conditions {
                self.emit(crate::vm::opcode::Opcode::LoadFast(subject_slot));
                self.compile_expr(condition)?;
                self.emit(crate::vm::opcode::Opcode::Identical); // Match uses strict comparison
                arm_jumps.push(self.emit_jump(crate::vm::opcode::Opcode::JumpIfTrue(0)));
            }

            // If none matched, jump to next arm
            let skip_arm = self.emit_jump(crate::vm::opcode::Opcode::Jump(0));

            // Patch arm condition jumps to result
            for jump in arm_jumps {
                self.patch_jump(jump);
            }

            // Compile arm result
            self.compile_expr(&arm.result)?;

            // Jump to end
            end_jumps.push(self.emit_jump(crate::vm::opcode::Opcode::Jump(0)));

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
            self.emit(crate::vm::opcode::Opcode::NewObject(class_idx));
            let msg_idx = self.intern_string("Unhandled match value".to_string());
            self.emit(crate::vm::opcode::Opcode::PushString(msg_idx));
            self.emit(crate::vm::opcode::Opcode::CallConstructor(1));
            self.emit(crate::vm::opcode::Opcode::Throw);
        }

        // Patch all end jumps
        for jump in end_jumps {
            self.patch_jump(jump);
        }

        Ok(())
    }

    /// Compile a switch statement (internal implementation)
    pub(crate) fn compile_switch_internal(
        &mut self,
        expr: &crate::ast::Expr,
        cases: &[crate::ast::SwitchCase],
        default: &Option<Vec<crate::ast::Stmt>>,
    ) -> Result<(), String> {
        self.compile_expr(expr)?;
        let switch_slot = self.allocate_local("__switch_expr__".to_string());
        self.emit(crate::vm::opcode::Opcode::StoreFast(switch_slot));

        let loop_start_idx = self.emit(crate::vm::opcode::Opcode::LoopStart(0, 0));

        let mut case_jumps: Vec<usize> = Vec::new();

        for case in cases {
            self.emit(crate::vm::opcode::Opcode::LoadFast(switch_slot));
            self.compile_expr(&case.value)?;
            self.emit(crate::vm::opcode::Opcode::Eq);
            case_jumps.push(self.emit_jump(crate::vm::opcode::Opcode::JumpIfTrue(0)));
        }

        let default_jump = self.emit_jump(crate::vm::opcode::Opcode::Jump(0));

        for (i, case) in cases.iter().enumerate() {
            self.patch_jump(case_jumps[i]);

            for stmt in &case.body {
                self.compile_stmt(stmt)?;
            }
        }

        self.patch_jump(default_jump);
        if let Some(default_body) = default {
            for stmt in default_body {
                self.compile_stmt(stmt)?;
            }
        }

        self.emit(crate::vm::opcode::Opcode::LoopEnd);

        let end_offset = self.current_offset();
        if let crate::vm::opcode::Opcode::LoopStart(_, ref mut break_target) =
            self.function.bytecode[loop_start_idx]
        {
            *break_target = end_offset as u32;
        }

        Ok(())
    }
}

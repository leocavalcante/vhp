//! Control flow statement handlers for if, while, for, foreach, and switch
//!
//! This module implements the execution of loop and conditional statements,
//! managing their control flow signals (break, continue, return).

use crate::ast::{Expr, Stmt, SwitchCase};
use crate::interpreter::value::Value;
use crate::interpreter::{ControlFlow, Interpreter};
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Handle if/elseif/else statements
    pub(super) fn handle_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &[Stmt],
        elseif_branches: &[(Expr, Vec<Stmt>)],
        else_branch: &Option<Vec<Stmt>>,
    ) -> std::io::Result<ControlFlow> {
        let cond_value = match self.eval_expr_safe(condition)? {
            Ok(v) => v,
            Err(cf) => return Ok(cf), // Propagate exception
        };

        if cond_value.to_bool() {
            for stmt in then_branch {
                let cf = self.execute_stmt(stmt)?;
                if cf != ControlFlow::None {
                    return Ok(cf);
                }
            }
        } else {
            let mut executed = false;
            for (elseif_cond, elseif_body) in elseif_branches {
                let elseif_value = match self.eval_expr_safe(elseif_cond)? {
                    Ok(v) => v,
                    Err(cf) => return Ok(cf), // Propagate exception
                };
                if elseif_value.to_bool() {
                    for stmt in elseif_body {
                        let cf = self.execute_stmt(stmt)?;
                        if cf != ControlFlow::None {
                            return Ok(cf);
                        }
                    }
                    executed = true;
                    break;
                }
            }

            if !executed {
                if let Some(else_body) = else_branch {
                    for stmt in else_body {
                        let cf = self.execute_stmt(stmt)?;
                        if cf != ControlFlow::None {
                            return Ok(cf);
                        }
                    }
                }
            }
        }
        Ok(ControlFlow::None)
    }

    /// Handle while loops
    pub(super) fn handle_while_stmt(
        &mut self,
        condition: &Expr,
        body: &[Stmt],
    ) -> std::io::Result<ControlFlow> {
        loop {
            let cond_value = match self.eval_expr_safe(condition)? {
                Ok(v) => v,
                Err(cf) => return Ok(cf), // Propagate exception
            };

            if !cond_value.to_bool() {
                break;
            }

            for stmt in body {
                let cf = self.execute_stmt(stmt)?;
                match cf {
                    ControlFlow::Break => return Ok(ControlFlow::None),
                    ControlFlow::Continue => break,
                    ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                    ControlFlow::Exception(e) => return Ok(ControlFlow::Exception(e)),
                    ControlFlow::None => {}
                }
            }
        }
        Ok(ControlFlow::None)
    }

    /// Handle do-while loops
    pub(super) fn handle_do_while_stmt(
        &mut self,
        body: &[Stmt],
        condition: &Expr,
    ) -> std::io::Result<ControlFlow> {
        loop {
            let mut should_break = false;
            let mut return_val = None;
            for stmt in body {
                let cf = self.execute_stmt(stmt)?;
                match cf {
                    ControlFlow::Break => {
                        should_break = true;
                        break;
                    }
                    ControlFlow::Continue => break,
                    ControlFlow::Return(v) => {
                        return_val = Some(v);
                        break;
                    }
                    ControlFlow::Exception(e) => return Ok(ControlFlow::Exception(e)),
                    ControlFlow::None => {}
                }
            }

            if let Some(v) = return_val {
                return Ok(ControlFlow::Return(v));
            }

            if should_break {
                break;
            }

            let cond_value = self.eval_expr(condition).map_err(std::io::Error::other)?;

            if !cond_value.to_bool() {
                break;
            }
        }
        Ok(ControlFlow::None)
    }

    /// Handle for loops
    pub(super) fn handle_for_stmt(
        &mut self,
        init: &Option<Expr>,
        condition: &Option<Expr>,
        update: &Option<Expr>,
        body: &[Stmt],
    ) -> std::io::Result<ControlFlow> {
        if let Some(init_expr) = init {
            self.eval_expr(init_expr).map_err(std::io::Error::other)?;
        }

        loop {
            if let Some(cond_expr) = condition {
                let cond_value = self.eval_expr(cond_expr).map_err(std::io::Error::other)?;
                if !cond_value.to_bool() {
                    break;
                }
            }

            let mut should_break = false;
            let mut return_val = None;
            for stmt in body {
                let cf = self.execute_stmt(stmt)?;
                match cf {
                    ControlFlow::Break => {
                        should_break = true;
                        break;
                    }
                    ControlFlow::Continue => break,
                    ControlFlow::Return(v) => {
                        return_val = Some(v);
                        break;
                    }
                    ControlFlow::Exception(e) => return Ok(ControlFlow::Exception(e)),
                    ControlFlow::None => {}
                }
            }

            if let Some(v) = return_val {
                return Ok(ControlFlow::Return(v));
            }

            if should_break {
                break;
            }

            if let Some(update_expr) = update {
                self.eval_expr(update_expr).map_err(std::io::Error::other)?;
            }
        }
        Ok(ControlFlow::None)
    }

    /// Handle foreach loops
    pub(super) fn handle_foreach_stmt(
        &mut self,
        array: &Expr,
        key: &Option<String>,
        value: &str,
        body: &[Stmt],
    ) -> std::io::Result<ControlFlow> {
        let array_val = self.eval_expr(array).map_err(std::io::Error::other)?;

        match array_val {
            Value::Array(arr) => {
                for (k, v) in arr {
                    // Bind key if specified
                    if let Some(key_name) = key {
                        self.variables.insert(key_name.clone(), k.to_value());
                    }

                    // Bind value
                    self.variables.insert(value.to_string(), v);

                    // Execute body
                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => return Ok(ControlFlow::None),
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::Exception(e) => return Ok(ControlFlow::Exception(e)),
                            ControlFlow::None => {}
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            _ => {
                // PHP would emit a warning here, we just skip
                Ok(ControlFlow::None)
            }
        }
    }

    /// Handle switch statements
    pub(super) fn handle_switch_stmt(
        &mut self,
        expr: &Expr,
        cases: &[SwitchCase],
        default: &Option<Vec<Stmt>>,
    ) -> std::io::Result<ControlFlow> {
        let switch_value = self.eval_expr(expr).map_err(std::io::Error::other)?;

        let mut matched = false;
        let mut fall_through = false;

        for SwitchCase { value, body } in cases {
            if !matched && !fall_through {
                let case_value = self.eval_expr(value).map_err(std::io::Error::other)?;
                if switch_value.loose_equals(&case_value) {
                    matched = true;
                }
            }

            if matched || fall_through {
                for stmt in body {
                    let cf = self.execute_stmt(stmt)?;
                    match cf {
                        ControlFlow::Break => return Ok(ControlFlow::None),
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        _ => {}
                    }
                }
                fall_through = true;
            }
        }

        if !matched && !fall_through {
            if let Some(default_body) = default {
                for stmt in default_body {
                    let cf = self.execute_stmt(stmt)?;
                    match cf {
                        ControlFlow::Break => return Ok(ControlFlow::None),
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        _ => {}
                    }
                }
            }
        }

        Ok(ControlFlow::None)
    }
}

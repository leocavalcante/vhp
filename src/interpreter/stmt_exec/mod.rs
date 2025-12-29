//! Statement execution implementation for the interpreter
//!
//! This module handles executing all statement types including:
//! - Echo and expression statements
//! - Control flow (if, while, for, foreach, switch)
//! - Function, class, interface, trait, and enum definitions
//! - Break, continue, and return

mod control_flow;
mod definitions;

use crate::ast::{Program, Stmt};
use crate::interpreter::value::Value;
use crate::interpreter::{ControlFlow, Interpreter};
use std::io::{self, Write};

impl<W: Write> Interpreter<W> {
    pub(super) fn execute_stmt(&mut self, stmt: &Stmt) -> io::Result<ControlFlow> {
        match stmt {
            // Output statements
            Stmt::Echo(exprs) => {
                for expr in exprs {
                    let value = self.eval_expr(expr).map_err(io::Error::other)?;
                    write!(self.output, "{}", value.to_output_string())?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr).map_err(io::Error::other)?;
                Ok(ControlFlow::None)
            }
            Stmt::Html(html) => {
                write!(self.output, "{}", html)?;
                Ok(ControlFlow::None)
            }

            // Control flow
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => self.handle_if_stmt(condition, then_branch, elseif_branches, else_branch),

            Stmt::While { condition, body } => self.handle_while_stmt(condition, body),

            Stmt::DoWhile { body, condition } => self.handle_do_while_stmt(body, condition),

            Stmt::For {
                init,
                condition,
                update,
                body,
            } => self.handle_for_stmt(init, condition, update, body),

            Stmt::Foreach {
                array,
                key,
                value,
                body,
            } => self.handle_foreach_stmt(array, key, value, body),

            Stmt::Switch {
                expr,
                cases,
                default,
            } => self.handle_switch_stmt(expr, cases, default),

            // Control signals
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e).map_err(io::Error::other)?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }

            // Type definitions
            Stmt::Function {
                name,
                params,
                body,
                attributes,
            } => self.handle_function_decl(name, params, body, attributes),

            Stmt::Class {
                name,
                readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
                attributes,
            } => self.handle_class_decl(
                name, *readonly, parent, interfaces, trait_uses, properties, methods, attributes,
            ),

            Stmt::Interface {
                name,
                parents,
                methods,
                constants,
                attributes,
            } => self.handle_interface_decl(name, parents, methods, constants, attributes),

            Stmt::Trait {
                name,
                uses,
                properties,
                methods,
                attributes,
            } => self.handle_trait_decl(name, uses, properties, methods, attributes),

            Stmt::Enum {
                name,
                backing_type,
                cases,
                methods,
                attributes,
            } => self.handle_enum_decl(name, *backing_type, cases, methods, attributes),
        }
    }

    /// Check if two values are identical (===)
    pub(super) fn values_identical(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    pub fn execute(&mut self, program: &Program) -> io::Result<()> {
        for stmt in &program.statements {
            let _ = self.execute_stmt(stmt)?;
        }
        self.output.flush()?;
        Ok(())
    }
}

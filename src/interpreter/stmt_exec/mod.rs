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
                    let value = match self.eval_expr(expr) {
                        Ok(v) => v,
                        Err(e) => {
                            // Check if this is a throw exception
                            if let Some(exc) = Self::parse_exception_error(&e) {
                                return Ok(ControlFlow::Exception(exc));
                            }
                            return Err(io::Error::other(e));
                        }
                    };
                    // Use value_to_string to support __toString magic method
                    let str_val = self.value_to_string(&value).map_err(io::Error::other)?;
                    write!(self.output, "{}", str_val)?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                match self.eval_expr(expr) {
                    Ok(_) => Ok(ControlFlow::None),
                    Err(e) => {
                        // Check if this is a throw exception
                        if let Some(exc) = Self::parse_exception_error(&e) {
                            Ok(ControlFlow::Exception(exc))
                        } else {
                            Err(io::Error::other(e))
                        }
                    }
                }
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
                    match self.eval_expr_safe(e)? {
                        Ok(v) => v,
                        Err(cf) => return Ok(cf), // Propagate exception
                    }
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }

            // Type definitions
            Stmt::Function {
                name,
                params,
                return_type,
                body,
                attributes,
            } => self.handle_function_decl(name, params, return_type, body, attributes),

            Stmt::Class {
                name,
                is_abstract,
                is_final,
                readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
                attributes,
            } => self.handle_class_decl(
                name,
                *is_abstract,
                *is_final,
                *readonly,
                parent,
                interfaces,
                trait_uses,
                properties,
                methods,
                attributes,
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

            // Exception handling
            Stmt::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
            } => self.handle_try_catch(try_body, catch_clauses, finally_body),

            Stmt::Throw(expr) => self.handle_throw(expr),

            // Namespace support
            Stmt::Namespace { name, body } => self.handle_namespace(name, body),
            Stmt::Use(items) => self.handle_use_statement(items),
            Stmt::GroupUse(group_use) => self.handle_group_use(group_use),

            // Declare directive
            Stmt::Declare { directives, body } => self.handle_declare(directives, body),
        }
    }

    /// Parse exception error message into ExceptionValue
    fn parse_exception_error(error: &str) -> Option<crate::interpreter::ExceptionValue> {
        if let Some(rest) = error.strip_prefix("__EXCEPTION__:") {
            if let Some((class_name, message)) = rest.split_once(':') {
                return Some(crate::interpreter::ExceptionValue {
                    class_name: class_name.to_string(),
                    message: message.to_string(),
                    code: 0,
                    file: String::new(),
                    line: 0,
                    previous: None,
                });
            }
        }
        None
    }

    /// Safely evaluate expression, converting throw expressions to ControlFlow::Exception
    pub(super) fn eval_expr_safe(
        &mut self,
        expr: &crate::ast::Expr,
    ) -> std::io::Result<Result<Value, ControlFlow>> {
        match self.eval_expr(expr) {
            Ok(v) => Ok(Ok(v)),
            Err(e) => {
                if let Some(exc) = Self::parse_exception_error(&e) {
                    Ok(Err(ControlFlow::Exception(exc)))
                } else {
                    Err(std::io::Error::other(e))
                }
            }
        }
    }

    /// Handle try/catch/finally statement
    fn handle_try_catch(
        &mut self,
        try_body: &[Stmt],
        catch_clauses: &[crate::ast::CatchClause],
        finally_body: &Option<Vec<Stmt>>,
    ) -> io::Result<ControlFlow> {
        // Execute try block
        let mut try_result = ControlFlow::None;
        for stmt in try_body {
            try_result = self.execute_stmt(stmt)?;
            if !matches!(try_result, ControlFlow::None) {
                break;
            }
        }

        let mut result = try_result.clone();
        let mut caught = false;

        // Check if exception was thrown
        if let ControlFlow::Exception(ref exception) = try_result {
            // Find matching catch clause
            for catch_clause in catch_clauses {
                let matches = catch_clause
                    .exception_types
                    .iter()
                    .any(|t| self.is_exception_instance_of(&exception.class_name, t));

                if matches {
                    // Bind exception to variable as an Exception object
                    let exc_obj = Value::Exception(exception.clone());
                    self.variables
                        .insert(catch_clause.variable.clone(), exc_obj);

                    // Execute catch body
                    result = ControlFlow::None;
                    for stmt in &catch_clause.body {
                        result = self.execute_stmt(stmt)?;
                        if !matches!(result, ControlFlow::None) {
                            break;
                        }
                    }
                    caught = true;
                    break;
                }
            }

            if !caught {
                result = try_result; // Re-propagate uncaught exception
            }
        }

        // Always execute finally block
        if let Some(finally_stmts) = finally_body {
            let finally_result = {
                let mut finally_flow = ControlFlow::None;
                for stmt in finally_stmts {
                    finally_flow = self.execute_stmt(stmt)?;
                    if !matches!(finally_flow, ControlFlow::None) {
                        break;
                    }
                }
                finally_flow
            };

            // finally result can override return/exception
            if !matches!(finally_result, ControlFlow::None) {
                result = finally_result;
            }
        }

        Ok(result)
    }

    /// Handle throw statement
    fn handle_throw(&mut self, expr: &crate::ast::Expr) -> io::Result<ControlFlow> {
        let value = self.eval_expr(expr).map_err(io::Error::other)?;

        match value {
            Value::Exception(exc) => Ok(ControlFlow::Exception(exc)),
            Value::Object(obj) => {
                // Create exception from object
                let message = obj
                    .properties
                    .get("message")
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();

                let code = obj
                    .properties
                    .get("code")
                    .and_then(|v| match v {
                        Value::Integer(n) => Some(*n),
                        _ => None,
                    })
                    .unwrap_or(0);

                Ok(ControlFlow::Exception(crate::interpreter::ExceptionValue {
                    class_name: obj.class_name.clone(),
                    message,
                    code,
                    file: String::new(),
                    line: 0,
                    previous: None,
                }))
            }
            _ => Err(io::Error::other(
                "Can only throw objects that extend Exception",
            )),
        }
    }

    /// Check if exception class is instance of (or extends) target class
    fn is_exception_instance_of(&self, exception_class: &str, target_class: &str) -> bool {
        // Direct match
        if exception_class.eq_ignore_ascii_case(target_class) {
            return true;
        }

        // Check inheritance chain
        let mut current = exception_class.to_lowercase();
        while let Some(class_def) = self.classes.get(&current) {
            if let Some(parent) = &class_def.parent {
                if parent.eq_ignore_ascii_case(target_class) {
                    return true;
                }
                current = parent.to_lowercase();
            } else {
                break;
            }
        }

        false
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
            let flow = self.execute_stmt(stmt)?;
            // Check for uncaught exception
            if let ControlFlow::Exception(exc) = flow {
                return Err(io::Error::other(format!(
                    "Uncaught {}: {}",
                    exc.class_name, exc.message
                )));
            }
        }
        self.output.flush()?;
        Ok(())
    }

    /// Handle namespace declaration
    fn handle_namespace(
        &mut self,
        name: &Option<crate::ast::QualifiedName>,
        body: &crate::ast::NamespaceBody,
    ) -> io::Result<ControlFlow> {
        // Set current namespace
        if let Some(ns_name) = name {
            self.namespace_context.current = ns_name.parts.clone();
        } else {
            // Global namespace
            self.namespace_context.current = vec![];
        }

        // Clear imports when entering new namespace
        self.namespace_context.class_imports.clear();
        self.namespace_context.function_imports.clear();
        self.namespace_context.constant_imports.clear();

        // Execute braced namespace body
        match body {
            crate::ast::NamespaceBody::Braced(stmts) => {
                for stmt in stmts {
                    let flow = self.execute_stmt(stmt)?;
                    if !matches!(flow, ControlFlow::None) {
                        return Ok(flow);
                    }
                }
            }
            crate::ast::NamespaceBody::Unbraced => {
                // For unbraced namespaces, the rest of the file is in this namespace
                // This is handled by the parser/caller
            }
        }

        Ok(ControlFlow::None)
    }

    /// Handle use statement
    fn handle_use_statement(&mut self, items: &[crate::ast::UseItem]) -> io::Result<ControlFlow> {
        for item in items {
            self.namespace_context.add_import(item);
        }
        Ok(ControlFlow::None)
    }

    /// Handle group use statement
    fn handle_group_use(&mut self, group_use: &crate::ast::GroupUse) -> io::Result<ControlFlow> {
        for item in &group_use.items {
            // Combine prefix and item name
            let mut full_parts = group_use.prefix.parts.clone();
            full_parts.extend(item.name.parts.clone());

            let full_name = crate::ast::QualifiedName::new(
                full_parts,
                group_use.prefix.is_fully_qualified || item.name.is_fully_qualified,
            );

            let full_item = crate::ast::UseItem {
                name: full_name,
                alias: item.alias.clone(),
                use_type: item.use_type.clone(),
            };

            self.namespace_context.add_import(&full_item);
        }
        Ok(ControlFlow::None)
    }

    /// Handle declare directive
    fn handle_declare(
        &mut self,
        directives: &[crate::ast::DeclareDirective],
        body: &Option<Vec<Stmt>>,
    ) -> io::Result<ControlFlow> {
        use crate::ast::DeclareDirective;

        // Process directives
        for directive in directives {
            match directive {
                DeclareDirective::StrictTypes(enabled) => {
                    if body.is_some() {
                        // Block-scope strict_types
                        self.strict_types_stack.push(self.strict_types);
                        self.strict_types = *enabled;
                    } else {
                        // File-scope strict_types
                        self.strict_types = *enabled;
                    }
                }
                DeclareDirective::Encoding(_) => {
                    // Encoding is mostly ignored in modern PHP
                }
                DeclareDirective::Ticks(_) => {
                    // Ticks for register_tick_function (advanced, not implemented)
                }
            }
        }

        // Execute body if present
        if let Some(stmts) = body {
            let mut result = ControlFlow::None;
            for stmt in stmts {
                result = self.execute_stmt(stmt)?;
                if !matches!(result, ControlFlow::None) {
                    break;
                }
            }

            // Restore strict_types after block
            if !self.strict_types_stack.is_empty() {
                self.strict_types = self.strict_types_stack.pop().unwrap();
            }

            return Ok(result);
        }

        Ok(ControlFlow::None)
    }
}

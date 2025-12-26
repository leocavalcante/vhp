//! Interpreter module for VHP
//!
//! This module contains the tree-walking interpreter that executes
//! the AST produced by the parser.

mod builtins;
mod value;

pub use value::Value;

use crate::ast::{AssignOp, BinaryOp, Expr, FunctionParam, Program, Stmt, SwitchCase, UnaryOp};
use std::collections::HashMap;
use std::io::{self, Write};

/// Control flow signals for break/continue/return
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Break,
    Continue,
    Return(Value),
}

/// User-defined function
#[derive(Debug, Clone)]
pub struct UserFunction {
    pub params: Vec<FunctionParam>,
    pub body: Vec<Stmt>,
}

pub struct Interpreter<W: Write> {
    output: W,
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Null => Ok(Value::Null),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Variable(name) => Ok(self
                .variables
                .get(name)
                .cloned()
                .unwrap_or(Value::Null)),

            Expr::Grouped(inner) => self.eval_expr(inner),

            Expr::Unary { op, expr } => self.eval_unary(op, expr),

            Expr::Binary { left, op, right } => self.eval_binary(left, op, right),

            Expr::Assign { var, op, value } => self.eval_assign(var, op, value),

            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond = self.eval_expr(condition)?;
                if cond.to_bool() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }

            Expr::FunctionCall { name, args } => self.call_function(name, args),
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<Value, String> {
        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        // Check for built-in functions first (case-insensitive)
        let lower_name = name.to_lowercase();
        match lower_name.as_str() {
            // String functions
            "strlen" => builtins::string::strlen(&arg_values),
            "substr" => builtins::string::substr(&arg_values),
            "strtoupper" => builtins::string::strtoupper(&arg_values),
            "strtolower" => builtins::string::strtolower(&arg_values),
            "trim" => builtins::string::trim(&arg_values),
            "ltrim" => builtins::string::ltrim(&arg_values),
            "rtrim" => builtins::string::rtrim(&arg_values),
            "str_repeat" => builtins::string::str_repeat(&arg_values),
            "str_replace" => builtins::string::str_replace(&arg_values),
            "strpos" => builtins::string::strpos(&arg_values),
            "str_contains" => builtins::string::str_contains(&arg_values),
            "str_starts_with" => builtins::string::str_starts_with(&arg_values),
            "str_ends_with" => builtins::string::str_ends_with(&arg_values),
            "ucfirst" => builtins::string::ucfirst(&arg_values),
            "lcfirst" => builtins::string::lcfirst(&arg_values),
            "ucwords" => builtins::string::ucwords(&arg_values),
            "strrev" => builtins::string::strrev(&arg_values),
            "str_pad" => builtins::string::str_pad(&arg_values),
            "explode" => builtins::string::explode(&arg_values),
            "implode" | "join" => builtins::string::implode(&arg_values),
            "sprintf" => builtins::string::sprintf(&arg_values),
            "printf" => builtins::output::printf(&mut self.output, &arg_values),
            "chr" => builtins::string::chr(&arg_values),
            "ord" => builtins::string::ord(&arg_values),

            // Math functions
            "abs" => builtins::math::abs(&arg_values),
            "ceil" => builtins::math::ceil(&arg_values),
            "floor" => builtins::math::floor(&arg_values),
            "round" => builtins::math::round(&arg_values),
            "max" => builtins::math::max(&arg_values),
            "min" => builtins::math::min(&arg_values),
            "pow" => builtins::math::pow(&arg_values),
            "sqrt" => builtins::math::sqrt(&arg_values),
            "rand" | "mt_rand" => builtins::math::rand(&arg_values),

            // Type functions
            "intval" => builtins::types::intval(&arg_values),
            "floatval" | "doubleval" => builtins::types::floatval(&arg_values),
            "strval" => builtins::types::strval(&arg_values),
            "boolval" => builtins::types::boolval(&arg_values),
            "gettype" => builtins::types::gettype(&arg_values),
            "is_null" => builtins::types::is_null(&arg_values),
            "is_bool" => builtins::types::is_bool(&arg_values),
            "is_int" | "is_integer" | "is_long" => builtins::types::is_int(&arg_values),
            "is_float" | "is_double" | "is_real" => builtins::types::is_float(&arg_values),
            "is_string" => builtins::types::is_string(&arg_values),
            "is_numeric" => builtins::types::is_numeric(&arg_values),
            "isset" => builtins::types::isset(&arg_values),
            "empty" => builtins::types::empty(&arg_values),

            // Output functions
            "print" => builtins::output::print(&mut self.output, &arg_values),
            "var_dump" => builtins::output::var_dump(&mut self.output, &arg_values),
            "print_r" => builtins::output::print_r(&mut self.output, &arg_values),

            // User-defined function
            _ => {
                // Look up in user-defined functions (case-insensitive)
                let func = self
                    .functions
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == lower_name)
                    .map(|(_, v)| v.clone());

                if let Some(func) = func {
                    self.call_user_function(&func, &arg_values)
                } else {
                    Err(format!("Call to undefined function {}()", name))
                }
            }
        }
    }

    fn call_user_function(
        &mut self,
        func: &UserFunction,
        args: &[Value],
    ) -> Result<Value, String> {
        // Save current variables (for scoping)
        let saved_variables = self.variables.clone();

        // Bind parameters
        for (i, param) in func.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default) = &param.default {
                self.eval_expr(default)?
            } else {
                return Err(format!(
                    "Missing argument {} for parameter ${}",
                    i + 1,
                    param.name
                ));
            };
            self.variables.insert(param.name.clone(), value);
        }

        // Execute function body
        let mut return_value = Value::Null;
        for stmt in &func.body.clone() {
            let cf = self
                .execute_stmt(stmt)
                .map_err(|e| e.to_string())?;
            if let ControlFlow::Return(val) = cf {
                return_value = val;
                break;
            }
        }

        // Restore variables
        self.variables = saved_variables;

        Ok(return_value)
    }

    fn eval_unary(&mut self, op: &UnaryOp, expr: &Expr) -> Result<Value, String> {
        match op {
            UnaryOp::Neg => {
                let val = self.eval_expr(expr)?;
                match val {
                    Value::Integer(n) => Ok(Value::Integer(-n)),
                    Value::Float(n) => Ok(Value::Float(-n)),
                    _ => Ok(Value::Float(-val.to_float())),
                }
            }
            UnaryOp::Not => {
                let val = self.eval_expr(expr)?;
                Ok(Value::Bool(!val.to_bool()))
            }
            UnaryOp::PreInc => {
                if let Expr::Variable(name) = expr {
                    let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                    let new_val = match val {
                        Value::Integer(n) => Value::Integer(n + 1),
                        Value::Float(n) => Value::Float(n + 1.0),
                        _ => Value::Integer(val.to_int() + 1),
                    };
                    self.variables.insert(name.clone(), new_val.clone());
                    Ok(new_val)
                } else {
                    Err("Pre-increment requires a variable".to_string())
                }
            }
            UnaryOp::PreDec => {
                if let Expr::Variable(name) = expr {
                    let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                    let new_val = match val {
                        Value::Integer(n) => Value::Integer(n - 1),
                        Value::Float(n) => Value::Float(n - 1.0),
                        _ => Value::Integer(val.to_int() - 1),
                    };
                    self.variables.insert(name.clone(), new_val.clone());
                    Ok(new_val)
                } else {
                    Err("Pre-decrement requires a variable".to_string())
                }
            }
            UnaryOp::PostInc => {
                if let Expr::Variable(name) = expr {
                    let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                    let new_val = match &val {
                        Value::Integer(n) => Value::Integer(n + 1),
                        Value::Float(n) => Value::Float(n + 1.0),
                        _ => Value::Integer(val.to_int() + 1),
                    };
                    self.variables.insert(name.clone(), new_val);
                    Ok(val)
                } else {
                    Err("Post-increment requires a variable".to_string())
                }
            }
            UnaryOp::PostDec => {
                if let Expr::Variable(name) = expr {
                    let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                    let new_val = match &val {
                        Value::Integer(n) => Value::Integer(n - 1),
                        Value::Float(n) => Value::Float(n - 1.0),
                        _ => Value::Integer(val.to_int() - 1),
                    };
                    self.variables.insert(name.clone(), new_val);
                    Ok(val)
                } else {
                    Err("Post-decrement requires a variable".to_string())
                }
            }
        }
    }

    fn eval_binary(&mut self, left: &Expr, op: &BinaryOp, right: &Expr) -> Result<Value, String> {
        // Short-circuit evaluation for logical operators
        match op {
            BinaryOp::And => {
                let left_val = self.eval_expr(left)?;
                if !left_val.to_bool() {
                    return Ok(Value::Bool(false));
                }
                let right_val = self.eval_expr(right)?;
                return Ok(Value::Bool(right_val.to_bool()));
            }
            BinaryOp::Or => {
                let left_val = self.eval_expr(left)?;
                if left_val.to_bool() {
                    return Ok(Value::Bool(true));
                }
                let right_val = self.eval_expr(right)?;
                return Ok(Value::Bool(right_val.to_bool()));
            }
            BinaryOp::NullCoalesce => {
                let left_val = self.eval_expr(left)?;
                if !matches!(left_val, Value::Null) {
                    return Ok(left_val);
                }
                return self.eval_expr(right);
            }
            _ => {}
        }

        let left_val = self.eval_expr(left)?;
        let right_val = self.eval_expr(right)?;

        match op {
            // Arithmetic
            BinaryOp::Add => self.numeric_op(&left_val, &right_val, |a, b| a + b, |a, b| a + b),
            BinaryOp::Sub => self.numeric_op(&left_val, &right_val, |a, b| a - b, |a, b| a - b),
            BinaryOp::Mul => self.numeric_op(&left_val, &right_val, |a, b| a * b, |a, b| a * b),
            BinaryOp::Div => {
                let right_f = right_val.to_float();
                if right_f == 0.0 {
                    return Err("Division by zero".to_string());
                }
                let left_f = left_val.to_float();
                let result = left_f / right_f;
                if result.fract() == 0.0 {
                    Ok(Value::Integer(result as i64))
                } else {
                    Ok(Value::Float(result))
                }
            }
            BinaryOp::Mod => {
                let right_i = right_val.to_int();
                if right_i == 0 {
                    return Err("Division by zero".to_string());
                }
                Ok(Value::Integer(left_val.to_int() % right_i))
            }
            BinaryOp::Pow => {
                let base = left_val.to_float();
                let exp = right_val.to_float();
                let result = base.powf(exp);
                if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
                    Ok(Value::Integer(result as i64))
                } else {
                    Ok(Value::Float(result))
                }
            }

            // String
            BinaryOp::Concat => Ok(Value::String(format!(
                "{}{}",
                left_val.to_string_val(),
                right_val.to_string_val()
            ))),

            // Comparison
            BinaryOp::Equal => Ok(Value::Bool(left_val.loose_equals(&right_val))),
            BinaryOp::NotEqual => Ok(Value::Bool(!left_val.loose_equals(&right_val))),
            BinaryOp::Identical => Ok(Value::Bool(left_val.type_equals(&right_val))),
            BinaryOp::NotIdentical => Ok(Value::Bool(!left_val.type_equals(&right_val))),
            BinaryOp::LessThan => Ok(Value::Bool(left_val.to_float() < right_val.to_float())),
            BinaryOp::GreaterThan => Ok(Value::Bool(left_val.to_float() > right_val.to_float())),
            BinaryOp::LessEqual => Ok(Value::Bool(left_val.to_float() <= right_val.to_float())),
            BinaryOp::GreaterEqual => Ok(Value::Bool(left_val.to_float() >= right_val.to_float())),
            BinaryOp::Spaceship => {
                let l = left_val.to_float();
                let r = right_val.to_float();
                Ok(Value::Integer(if l < r {
                    -1
                } else if l > r {
                    1
                } else {
                    0
                }))
            }

            // Logical (non-short-circuit case - xor)
            BinaryOp::Xor => Ok(Value::Bool(left_val.to_bool() ^ right_val.to_bool())),

            // Already handled above
            BinaryOp::And | BinaryOp::Or | BinaryOp::NullCoalesce => unreachable!(),
        }
    }

    fn numeric_op<F, G>(&self, left: &Value, right: &Value, int_op: F, float_op: G) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> i64,
        G: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(int_op(*a, *b))),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(float_op(*a, *b))),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(float_op(*a as f64, *b))),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(float_op(*a, *b as f64))),
            _ => Ok(Value::Float(float_op(left.to_float(), right.to_float()))),
        }
    }

    fn eval_assign(&mut self, var: &str, op: &AssignOp, value: &Expr) -> Result<Value, String> {
        let new_value = match op {
            AssignOp::Assign => self.eval_expr(value)?,
            AssignOp::AddAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                self.numeric_op(&current, &right, |a, b| a + b, |a, b| a + b)?
            }
            AssignOp::SubAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                self.numeric_op(&current, &right, |a, b| a - b, |a, b| a - b)?
            }
            AssignOp::MulAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                self.numeric_op(&current, &right, |a, b| a * b, |a, b| a * b)?
            }
            AssignOp::DivAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                let right_f = right.to_float();
                if right_f == 0.0 {
                    return Err("Division by zero".to_string());
                }
                let result = current.to_float() / right_f;
                if result.fract() == 0.0 {
                    Value::Integer(result as i64)
                } else {
                    Value::Float(result)
                }
            }
            AssignOp::ModAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                let right_i = right.to_int();
                if right_i == 0 {
                    return Err("Division by zero".to_string());
                }
                Value::Integer(current.to_int() % right_i)
            }
            AssignOp::ConcatAssign => {
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                let right = self.eval_expr(value)?;
                Value::String(format!(
                    "{}{}",
                    current.to_string_val(),
                    right.to_string_val()
                ))
            }
        };

        self.variables.insert(var.to_string(), new_value.clone());
        Ok(new_value)
    }

    pub fn execute(&mut self, program: &Program) -> io::Result<()> {
        for stmt in &program.statements {
            let _ = self.execute_stmt(stmt)?;
        }
        self.output.flush()?;
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> io::Result<ControlFlow> {
        match stmt {
            Stmt::Echo(exprs) => {
                for expr in exprs {
                    let value = self.eval_expr(expr).map_err(|e| {
                        io::Error::other(e)
                    })?;
                    write!(self.output, "{}", value.to_output_string())?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr).map_err(|e| {
                    io::Error::other(e)
                })?;
                Ok(ControlFlow::None)
            }
            Stmt::Html(html) => {
                write!(self.output, "{}", html)?;
                Ok(ControlFlow::None)
            }
            Stmt::If {
                condition,
                then_branch,
                elseif_branches,
                else_branch,
            } => {
                let cond_value = self.eval_expr(condition).map_err(|e| {
                    io::Error::other(e)
                })?;

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
                        let elseif_value = self.eval_expr(elseif_cond).map_err(|e| {
                            io::Error::other(e)
                        })?;
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
            Stmt::While { condition, body } => {
                loop {
                    let cond_value = self.eval_expr(condition).map_err(|e| {
                        io::Error::other(e)
                    })?;

                    if !cond_value.to_bool() {
                        break;
                    }

                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => return Ok(ControlFlow::None),
                            ControlFlow::Continue => break,
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::None => {}
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::DoWhile { body, condition } => {
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
                            ControlFlow::None => {}
                        }
                    }

                    if let Some(v) = return_val {
                        return Ok(ControlFlow::Return(v));
                    }

                    if should_break {
                        break;
                    }

                    let cond_value = self.eval_expr(condition).map_err(|e| {
                        io::Error::other(e)
                    })?;

                    if !cond_value.to_bool() {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For {
                init,
                condition,
                update,
                body,
            } => {
                if let Some(init_expr) = init {
                    self.eval_expr(init_expr).map_err(|e| {
                        io::Error::other(e)
                    })?;
                }

                loop {
                    if let Some(cond_expr) = condition {
                        let cond_value = self.eval_expr(cond_expr).map_err(|e| {
                            io::Error::other(e)
                        })?;
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
                        self.eval_expr(update_expr).map_err(|e| {
                            io::Error::other(e)
                        })?;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Foreach {
                array: _,
                key: _,
                value: _,
                body: _,
            } => {
                Err(io::Error::other(
                    "foreach requires array support (not yet implemented)",
                ))
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                let switch_value = self.eval_expr(expr).map_err(|e| {
                    io::Error::other(e)
                })?;

                let mut matched = false;
                let mut fall_through = false;

                for SwitchCase { value, body } in cases {
                    if !matched && !fall_through {
                        let case_value = self.eval_expr(value).map_err(|e| {
                            io::Error::other(e)
                        })?;
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
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
            Stmt::Function { name, params, body } => {
                self.functions.insert(
                    name.clone(),
                    UserFunction {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e).map_err(|e| {
                        io::Error::other(e)
                    })?
                } else {
                    Value::Null
                };
                Ok(ControlFlow::Return(value))
            }
        }
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

use crate::ast::{AssignOp, BinaryOp, Expr, Program, Stmt, SwitchCase, UnaryOp};
use std::collections::HashMap;
use std::io::{self, Write};

/// Control flow signals for break/continue
#[derive(Debug, Clone, PartialEq)]
pub enum ControlFlow {
    None,
    Break,
    Continue,
}

/// Runtime value representation
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

impl Value {
    /// Convert value to string for output
    pub fn to_output_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    String::new()
                }
            }
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
        }
    }

    /// Convert to boolean (PHP truthiness)
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty() && s != "0",
        }
    }

    /// Convert to integer
    pub fn to_int(&self) -> i64 {
        match self {
            Value::Null => 0,
            Value::Bool(b) => {
                if *b {
                    1
                } else {
                    0
                }
            }
            Value::Integer(n) => *n,
            Value::Float(n) => *n as i64,
            Value::String(s) => s.parse().unwrap_or(0),
        }
    }

    /// Convert to float
    pub fn to_float(&self) -> f64 {
        match self {
            Value::Null => 0.0,
            Value::Bool(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Integer(n) => *n as f64,
            Value::Float(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0.0),
        }
    }

    /// Convert to string
    pub fn to_string_val(&self) -> String {
        match self {
            Value::Null => String::new(),
            Value::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    String::new()
                }
            }
            Value::Integer(n) => n.to_string(),
            Value::Float(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{:.0}", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
        }
    }

    /// Check if value is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Float(_))
    }

    /// Check type equality for === and !==
    pub fn type_equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    /// Loose equality for == and !=
    pub fn loose_equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Null, Value::Bool(b)) | (Value::Bool(b), Value::Null) => !b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
                (*a as f64) == *b
            }
            (Value::String(a), Value::String(b)) => a == b,
            // Numeric string comparisons
            (Value::Integer(n), Value::String(s)) | (Value::String(s), Value::Integer(n)) => {
                if let Ok(sn) = s.parse::<i64>() {
                    *n == sn
                } else if let Ok(sf) = s.parse::<f64>() {
                    (*n as f64) == sf
                } else {
                    false
                }
            }
            (Value::Float(n), Value::String(s)) | (Value::String(s), Value::Float(n)) => {
                if let Ok(sf) = s.parse::<f64>() {
                    *n == sf
                } else {
                    false
                }
            }
            _ => self.to_bool() == other.to_bool(),
        }
    }
}

pub struct Interpreter<W: Write> {
    output: W,
    variables: HashMap<String, Value>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            variables: HashMap::new(),
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
        }
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
                    Ok(val) // Return old value
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
                    Ok(val) // Return old value
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
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;
                    write!(self.output, "{}", value.to_output_string())?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::Expression(expr) => {
                self.eval_expr(expr).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
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
                    io::Error::new(io::ErrorKind::Other, e)
                })?;

                if cond_value.to_bool() {
                    for stmt in then_branch {
                        let cf = self.execute_stmt(stmt)?;
                        if cf != ControlFlow::None {
                            return Ok(cf);
                        }
                    }
                } else {
                    // Try elseif branches
                    let mut executed = false;
                    for (elseif_cond, elseif_body) in elseif_branches {
                        let elseif_value = self.eval_expr(elseif_cond).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
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

                    // Execute else branch if no condition was true
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
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    if !cond_value.to_bool() {
                        break;
                    }

                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => return Ok(ControlFlow::None),
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::DoWhile { body, condition } => {
                loop {
                    let mut should_break = false;
                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }

                    if should_break {
                        break;
                    }

                    let cond_value = self.eval_expr(condition).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
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
                // Execute init
                if let Some(init_expr) = init {
                    self.eval_expr(init_expr).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;
                }

                loop {
                    // Check condition
                    if let Some(cond_expr) = condition {
                        let cond_value = self.eval_expr(cond_expr).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
                        })?;
                        if !cond_value.to_bool() {
                            break;
                        }
                    }

                    // Execute body
                    let mut should_break = false;
                    for stmt in body {
                        let cf = self.execute_stmt(stmt)?;
                        match cf {
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Continue => break,
                            ControlFlow::None => {}
                        }
                    }

                    if should_break {
                        break;
                    }

                    // Execute update
                    if let Some(update_expr) = update {
                        self.eval_expr(update_expr).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
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
                // Foreach requires array support - skip for now with a clear error
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "foreach requires array support (not yet implemented)",
                ))
            }
            Stmt::Switch {
                expr,
                cases,
                default,
            } => {
                let switch_value = self.eval_expr(expr).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
                })?;

                let mut matched = false;
                let mut fall_through = false;

                for SwitchCase { value, body } in cases {
                    if !matched && !fall_through {
                        let case_value = self.eval_expr(value).map_err(|e| {
                            io::Error::new(io::ErrorKind::Other, e)
                        })?;
                        if switch_value.loose_equals(&case_value) {
                            matched = true;
                        }
                    }

                    if matched || fall_through {
                        for stmt in body {
                            let cf = self.execute_stmt(stmt)?;
                            if cf == ControlFlow::Break {
                                return Ok(ControlFlow::None);
                            }
                        }
                        fall_through = true;
                    }
                }

                // Execute default if no case matched
                if !matched && !fall_through {
                    if let Some(default_body) = default {
                        for stmt in default_body {
                            let cf = self.execute_stmt(stmt)?;
                            if cf == ControlFlow::Break {
                                return Ok(ControlFlow::None);
                            }
                        }
                    }
                }

                Ok(ControlFlow::None)
            }
            Stmt::Break => Ok(ControlFlow::Break),
            Stmt::Continue => Ok(ControlFlow::Continue),
        }
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

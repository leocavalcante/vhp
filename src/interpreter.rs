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

/// Runtime value representation
#[derive(Debug, Clone, PartialEq)]
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
            "strlen" => self.builtin_strlen(&arg_values),
            "substr" => self.builtin_substr(&arg_values),
            "strtoupper" => self.builtin_strtoupper(&arg_values),
            "strtolower" => self.builtin_strtolower(&arg_values),
            "trim" => self.builtin_trim(&arg_values),
            "ltrim" => self.builtin_ltrim(&arg_values),
            "rtrim" => self.builtin_rtrim(&arg_values),
            "str_repeat" => self.builtin_str_repeat(&arg_values),
            "str_replace" => self.builtin_str_replace(&arg_values),
            "strpos" => self.builtin_strpos(&arg_values),
            "str_contains" => self.builtin_str_contains(&arg_values),
            "str_starts_with" => self.builtin_str_starts_with(&arg_values),
            "str_ends_with" => self.builtin_str_ends_with(&arg_values),
            "ucfirst" => self.builtin_ucfirst(&arg_values),
            "lcfirst" => self.builtin_lcfirst(&arg_values),
            "ucwords" => self.builtin_ucwords(&arg_values),
            "strrev" => self.builtin_strrev(&arg_values),
            "str_pad" => self.builtin_str_pad(&arg_values),
            "explode" => self.builtin_explode(&arg_values),
            "implode" | "join" => self.builtin_implode(&arg_values),
            "sprintf" => self.builtin_sprintf(&arg_values),
            "printf" => self.builtin_printf(&arg_values),
            "chr" => self.builtin_chr(&arg_values),
            "ord" => self.builtin_ord(&arg_values),

            // Math functions
            "abs" => self.builtin_abs(&arg_values),
            "ceil" => self.builtin_ceil(&arg_values),
            "floor" => self.builtin_floor(&arg_values),
            "round" => self.builtin_round(&arg_values),
            "max" => self.builtin_max(&arg_values),
            "min" => self.builtin_min(&arg_values),
            "pow" => self.builtin_pow(&arg_values),
            "sqrt" => self.builtin_sqrt(&arg_values),
            "rand" => self.builtin_rand(&arg_values),
            "mt_rand" => self.builtin_rand(&arg_values),

            // Type functions
            "intval" => self.builtin_intval(&arg_values),
            "floatval" | "doubleval" => self.builtin_floatval(&arg_values),
            "strval" => self.builtin_strval(&arg_values),
            "boolval" => self.builtin_boolval(&arg_values),
            "gettype" => self.builtin_gettype(&arg_values),
            "is_null" => self.builtin_is_null(&arg_values),
            "is_bool" => self.builtin_is_bool(&arg_values),
            "is_int" | "is_integer" | "is_long" => self.builtin_is_int(&arg_values),
            "is_float" | "is_double" | "is_real" => self.builtin_is_float(&arg_values),
            "is_string" => self.builtin_is_string(&arg_values),
            "is_numeric" => self.builtin_is_numeric(&arg_values),
            "isset" => {
                // isset is special - we need to check the original expression
                if args.is_empty() {
                    return Err("isset() expects at least 1 parameter".to_string());
                }
                // For simplicity, check if arg evaluates to non-null
                Ok(Value::Bool(!matches!(arg_values.first(), Some(Value::Null) | None)))
            }
            "empty" => {
                if args.is_empty() {
                    return Err("empty() expects exactly 1 parameter".to_string());
                }
                Ok(Value::Bool(!arg_values.first().map(|v| v.to_bool()).unwrap_or(false)))
            }

            // Output functions
            "print" => self.builtin_print(&arg_values),
            "var_dump" => self.builtin_var_dump(&arg_values),
            "print_r" => self.builtin_print_r(&arg_values),

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

        // Restore variables (except by-ref params)
        // For simplicity, we restore all variables for now
        // By-ref handling would need special treatment
        self.variables = saved_variables;

        Ok(return_value)
    }

    // Built-in function implementations

    fn builtin_strlen(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("strlen() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Integer(args[0].to_string_val().len() as i64))
    }

    fn builtin_substr(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("substr() expects at least 2 parameters".to_string());
        }
        let s = args[0].to_string_val();
        let start = args[1].to_int();
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len() as i64;

        let start_idx = if start < 0 {
            (len + start).max(0) as usize
        } else {
            start.min(len) as usize
        };

        let result = if args.len() >= 3 {
            let length = args[2].to_int();
            if length < 0 {
                let end_idx = ((len + length) as usize).max(start_idx);
                chars[start_idx..end_idx].iter().collect()
            } else {
                chars[start_idx..].iter().take(length as usize).collect()
            }
        } else {
            chars[start_idx..].iter().collect()
        };

        Ok(Value::String(result))
    }

    fn builtin_strtoupper(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("strtoupper() expects exactly 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val().to_uppercase()))
    }

    fn builtin_strtolower(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("strtolower() expects exactly 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val().to_lowercase()))
    }

    fn builtin_trim(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("trim() expects at least 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val().trim().to_string()))
    }

    fn builtin_ltrim(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("ltrim() expects at least 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val().trim_start().to_string()))
    }

    fn builtin_rtrim(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("rtrim() expects at least 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val().trim_end().to_string()))
    }

    fn builtin_str_repeat(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("str_repeat() expects exactly 2 parameters".to_string());
        }
        let s = args[0].to_string_val();
        let times = args[1].to_int().max(0) as usize;
        Ok(Value::String(s.repeat(times)))
    }

    fn builtin_str_replace(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 3 {
            return Err("str_replace() expects at least 3 parameters".to_string());
        }
        let search = args[0].to_string_val();
        let replace = args[1].to_string_val();
        let subject = args[2].to_string_val();
        Ok(Value::String(subject.replace(&search, &replace)))
    }

    fn builtin_strpos(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("strpos() expects at least 2 parameters".to_string());
        }
        let haystack = args[0].to_string_val();
        let needle = args[1].to_string_val();
        match haystack.find(&needle) {
            Some(pos) => Ok(Value::Integer(pos as i64)),
            None => Ok(Value::Bool(false)),
        }
    }

    fn builtin_str_contains(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("str_contains() expects exactly 2 parameters".to_string());
        }
        let haystack = args[0].to_string_val();
        let needle = args[1].to_string_val();
        Ok(Value::Bool(haystack.contains(&needle)))
    }

    fn builtin_str_starts_with(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("str_starts_with() expects exactly 2 parameters".to_string());
        }
        let haystack = args[0].to_string_val();
        let needle = args[1].to_string_val();
        Ok(Value::Bool(haystack.starts_with(&needle)))
    }

    fn builtin_str_ends_with(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("str_ends_with() expects exactly 2 parameters".to_string());
        }
        let haystack = args[0].to_string_val();
        let needle = args[1].to_string_val();
        Ok(Value::Bool(haystack.ends_with(&needle)))
    }

    fn builtin_ucfirst(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("ucfirst() expects exactly 1 parameter".to_string());
        }
        let s = args[0].to_string_val();
        let mut chars = s.chars();
        let result = match chars.next() {
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
            None => String::new(),
        };
        Ok(Value::String(result))
    }

    fn builtin_lcfirst(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("lcfirst() expects exactly 1 parameter".to_string());
        }
        let s = args[0].to_string_val();
        let mut chars = s.chars();
        let result = match chars.next() {
            Some(c) => c.to_lowercase().to_string() + chars.as_str(),
            None => String::new(),
        };
        Ok(Value::String(result))
    }

    fn builtin_ucwords(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("ucwords() expects at least 1 parameter".to_string());
        }
        let s = args[0].to_string_val();
        let result: String = s
            .split(' ')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        Ok(Value::String(result))
    }

    fn builtin_strrev(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("strrev() expects exactly 1 parameter".to_string());
        }
        let s = args[0].to_string_val();
        Ok(Value::String(s.chars().rev().collect()))
    }

    fn builtin_str_pad(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("str_pad() expects at least 2 parameters".to_string());
        }
        let s = args[0].to_string_val();
        let length = args[1].to_int() as usize;
        let pad_string = if args.len() >= 3 {
            args[2].to_string_val()
        } else {
            " ".to_string()
        };
        let pad_type = if args.len() >= 4 {
            args[3].to_int()
        } else {
            1 // STR_PAD_RIGHT
        };

        if s.len() >= length || pad_string.is_empty() {
            return Ok(Value::String(s));
        }

        let pad_needed = length - s.len();
        let pad_chars: Vec<char> = pad_string.chars().collect();

        match pad_type {
            0 => {
                // STR_PAD_LEFT
                let mut result = String::new();
                for i in 0..pad_needed {
                    result.push(pad_chars[i % pad_chars.len()]);
                }
                result.push_str(&s);
                Ok(Value::String(result))
            }
            2 => {
                // STR_PAD_BOTH
                let left_pad = pad_needed / 2;
                let right_pad = pad_needed - left_pad;
                let mut result = String::new();
                for i in 0..left_pad {
                    result.push(pad_chars[i % pad_chars.len()]);
                }
                result.push_str(&s);
                for i in 0..right_pad {
                    result.push(pad_chars[i % pad_chars.len()]);
                }
                Ok(Value::String(result))
            }
            _ => {
                // STR_PAD_RIGHT (default)
                let mut result = s;
                for i in 0..pad_needed {
                    result.push(pad_chars[i % pad_chars.len()]);
                }
                Ok(Value::String(result))
            }
        }
    }

    fn builtin_explode(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("explode() expects at least 2 parameters".to_string());
        }
        // Since we don't have arrays yet, return the string as-is or first part
        // This is a stub implementation
        let _delimiter = args[0].to_string_val();
        let string = args[1].to_string_val();
        // For now, just return the original string
        // Full implementation requires array support
        Ok(Value::String(string))
    }

    fn builtin_implode(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("implode() expects at least 1 parameter".to_string());
        }
        // Since we don't have arrays yet, return empty string
        // This is a stub implementation
        Ok(Value::String(String::new()))
    }

    fn builtin_sprintf(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("sprintf() expects at least 1 parameter".to_string());
        }
        let format = args[0].to_string_val();
        let mut arg_idx = 1;

        // Simple implementation - replace %s, %d, %f placeholders
        let chars: Vec<char> = format.chars().collect();
        let mut i = 0;
        let mut output = String::new();

        while i < chars.len() {
            if chars[i] == '%' && i + 1 < chars.len() {
                match chars[i + 1] {
                    '%' => {
                        output.push('%');
                        i += 2;
                    }
                    's' => {
                        if arg_idx < args.len() {
                            output.push_str(&args[arg_idx].to_string_val());
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'd' | 'i' => {
                        if arg_idx < args.len() {
                            output.push_str(&args[arg_idx].to_int().to_string());
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    'f' => {
                        if arg_idx < args.len() {
                            output.push_str(&format!("{:.6}", args[arg_idx].to_float()));
                            arg_idx += 1;
                        }
                        i += 2;
                    }
                    _ => {
                        output.push(chars[i]);
                        i += 1;
                    }
                }
            } else {
                output.push(chars[i]);
                i += 1;
            }
        }

        Ok(Value::String(output))
    }

    fn builtin_printf(&mut self, args: &[Value]) -> Result<Value, String> {
        let result = self.builtin_sprintf(args)?;
        write!(self.output, "{}", result.to_string_val())
            .map_err(|e| e.to_string())?;
        Ok(Value::Integer(result.to_string_val().len() as i64))
    }

    fn builtin_chr(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("chr() expects exactly 1 parameter".to_string());
        }
        let code = args[0].to_int() as u8;
        Ok(Value::String((code as char).to_string()))
    }

    fn builtin_ord(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("ord() expects exactly 1 parameter".to_string());
        }
        let s = args[0].to_string_val();
        match s.chars().next() {
            Some(c) => Ok(Value::Integer(c as i64)),
            None => Ok(Value::Integer(0)),
        }
    }

    // Math functions

    fn builtin_abs(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("abs() expects exactly 1 parameter".to_string());
        }
        match &args[0] {
            Value::Integer(n) => Ok(Value::Integer(n.abs())),
            Value::Float(n) => Ok(Value::Float(n.abs())),
            v => Ok(Value::Float(v.to_float().abs())),
        }
    }

    fn builtin_ceil(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("ceil() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Float(args[0].to_float().ceil()))
    }

    fn builtin_floor(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("floor() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Float(args[0].to_float().floor()))
    }

    fn builtin_round(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("round() expects at least 1 parameter".to_string());
        }
        let val = args[0].to_float();
        let precision = if args.len() >= 2 {
            args[1].to_int() as i32
        } else {
            0
        };
        let factor = 10_f64.powi(precision);
        Ok(Value::Float((val * factor).round() / factor))
    }

    fn builtin_max(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("max() expects at least 1 parameter".to_string());
        }
        let mut max_val = args[0].to_float();
        for arg in args.iter().skip(1) {
            let val = arg.to_float();
            if val > max_val {
                max_val = val;
            }
        }
        // Return integer if all args are integers
        if args.iter().all(|a| matches!(a, Value::Integer(_))) {
            Ok(Value::Integer(max_val as i64))
        } else {
            Ok(Value::Float(max_val))
        }
    }

    fn builtin_min(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("min() expects at least 1 parameter".to_string());
        }
        let mut min_val = args[0].to_float();
        for arg in args.iter().skip(1) {
            let val = arg.to_float();
            if val < min_val {
                min_val = val;
            }
        }
        if args.iter().all(|a| matches!(a, Value::Integer(_))) {
            Ok(Value::Integer(min_val as i64))
        } else {
            Ok(Value::Float(min_val))
        }
    }

    fn builtin_pow(&self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("pow() expects exactly 2 parameters".to_string());
        }
        let base = args[0].to_float();
        let exp = args[1].to_float();
        let result = base.powf(exp);
        if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
            Ok(Value::Integer(result as i64))
        } else {
            Ok(Value::Float(result))
        }
    }

    fn builtin_sqrt(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("sqrt() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Float(args[0].to_float().sqrt()))
    }

    fn builtin_rand(&self, args: &[Value]) -> Result<Value, String> {
        // Simple pseudo-random implementation
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let (min, max) = if args.len() >= 2 {
            (args[0].to_int(), args[1].to_int())
        } else if args.len() == 1 {
            (0, args[0].to_int())
        } else {
            (0, i32::MAX as i64)
        };

        let range = (max - min + 1) as u128;
        let random = if range > 0 {
            min + ((seed % range) as i64)
        } else {
            min
        };

        Ok(Value::Integer(random))
    }

    // Type functions

    fn builtin_intval(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("intval() expects at least 1 parameter".to_string());
        }
        Ok(Value::Integer(args[0].to_int()))
    }

    fn builtin_floatval(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("floatval() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Float(args[0].to_float()))
    }

    fn builtin_strval(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("strval() expects exactly 1 parameter".to_string());
        }
        Ok(Value::String(args[0].to_string_val()))
    }

    fn builtin_boolval(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("boolval() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(args[0].to_bool()))
    }

    fn builtin_gettype(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("gettype() expects exactly 1 parameter".to_string());
        }
        let type_name = match &args[0] {
            Value::Null => "NULL",
            Value::Bool(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "double",
            Value::String(_) => "string",
        };
        Ok(Value::String(type_name.to_string()))
    }

    fn builtin_is_null(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_null() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Null)))
    }

    fn builtin_is_bool(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_bool() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
    }

    fn builtin_is_int(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_int() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Integer(_))))
    }

    fn builtin_is_float(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_float() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::Float(_))))
    }

    fn builtin_is_string(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_string() expects exactly 1 parameter".to_string());
        }
        Ok(Value::Bool(matches!(args[0], Value::String(_))))
    }

    fn builtin_is_numeric(&self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("is_numeric() expects exactly 1 parameter".to_string());
        }
        let is_numeric = match &args[0] {
            Value::Integer(_) | Value::Float(_) => true,
            Value::String(s) => s.parse::<f64>().is_ok(),
            _ => false,
        };
        Ok(Value::Bool(is_numeric))
    }

    // Output functions

    fn builtin_print(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("print() expects exactly 1 parameter".to_string());
        }
        write!(self.output, "{}", args[0].to_output_string())
            .map_err(|e| e.to_string())?;
        Ok(Value::Integer(1))
    }

    fn builtin_var_dump(&mut self, args: &[Value]) -> Result<Value, String> {
        for arg in args {
            let dump = match arg {
                Value::Null => "NULL\n".to_string(),
                Value::Bool(b) => format!("bool({})\n", b),
                Value::Integer(n) => format!("int({})\n", n),
                Value::Float(n) => format!("float({})\n", n),
                Value::String(s) => format!("string({}) \"{}\"\n", s.len(), s),
            };
            write!(self.output, "{}", dump).map_err(|e| e.to_string())?;
        }
        Ok(Value::Null)
    }

    fn builtin_print_r(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.is_empty() {
            return Err("print_r() expects at least 1 parameter".to_string());
        }
        let return_output = args.len() >= 2 && args[1].to_bool();
        let output = args[0].to_string_val();

        if return_output {
            Ok(Value::String(output))
        } else {
            write!(self.output, "{}", output).map_err(|e| e.to_string())?;
            Ok(Value::Bool(true))
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
                            match cf {
                                ControlFlow::Break => return Ok(ControlFlow::None),
                                ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                                _ => {}
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
                // Register the function
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
                        io::Error::new(io::ErrorKind::Other, e)
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

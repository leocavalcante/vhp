//! Interpreter module for VHP
//!
//! This module contains the tree-walking interpreter that executes
//! the AST produced by the parser.

mod builtins;
mod value;

pub use value::{ArrayKey, ObjectInstance, Value};

use crate::ast::{
    AssignOp, BinaryOp, Expr, FunctionParam, Program, Property, Stmt, SwitchCase, UnaryOp,
    Visibility,
};
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

/// Class definition stored in the interpreter
#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: String,
    #[allow(dead_code)] // Will be used for inheritance support
    pub parent: Option<String>,
    pub properties: Vec<Property>,
    pub methods: HashMap<String, UserFunction>,
    #[allow(dead_code)] // Will be used for visibility enforcement
    pub method_visibility: HashMap<String, Visibility>,
}

pub struct Interpreter<W: Write> {
    output: W,
    variables: HashMap<String, Value>,
    functions: HashMap<String, UserFunction>,
    classes: HashMap<String, ClassDefinition>,
    current_object: Option<ObjectInstance>,
}

impl<W: Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            current_object: None,
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

            Expr::Array(elements) => self.eval_array(elements),

            Expr::ArrayAccess { array, index } => self.eval_array_access(array, index),

            Expr::ArrayAssign {
                array,
                index,
                op,
                value,
            } => self.eval_array_assign(array, index, op, value),

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

            Expr::New { class_name, args } => self.eval_new(class_name, args),

            Expr::PropertyAccess { object, property } => self.eval_property_access(object, property),

            Expr::MethodCall {
                object,
                method,
                args,
            } => self.eval_method_call(object, method, args),

            Expr::PropertyAssign {
                object,
                property,
                value,
            } => self.eval_property_assign(object, property, value),

            Expr::This => {
                if let Some(ref obj) = self.current_object {
                    Ok(Value::Object(obj.clone()))
                } else {
                    Err("Cannot use $this outside of object context".to_string())
                }
            }

            Expr::StaticMethodCall {
                class_name,
                method,
                args,
            } => self.eval_static_method_call(class_name, method, args),
        }
    }

    fn eval_array(&mut self, elements: &[crate::ast::ArrayElement]) -> Result<Value, String> {
        let mut arr = Vec::new();
        let mut next_int_key: i64 = 0;

        for elem in elements {
            let key = if let Some(key_expr) = &elem.key {
                let key_val = self.eval_expr(key_expr)?;
                let key = ArrayKey::from_value(&key_val);
                // Update next_int_key if this is an integer key
                if let ArrayKey::Integer(n) = &key {
                    if *n >= next_int_key {
                        next_int_key = *n + 1;
                    }
                }
                key
            } else {
                let key = ArrayKey::Integer(next_int_key);
                next_int_key += 1;
                key
            };

            let value = self.eval_expr(&elem.value)?;
            arr.push((key, value));
        }

        Ok(Value::Array(arr))
    }

    fn eval_array_access(&mut self, array: &Expr, index: &Expr) -> Result<Value, String> {
        let array_val = self.eval_expr(array)?;
        let index_val = self.eval_expr(index)?;
        let key = ArrayKey::from_value(&index_val);

        match array_val {
            Value::Array(arr) => {
                for (k, v) in arr {
                    if k == key {
                        return Ok(v);
                    }
                }
                Ok(Value::Null)
            }
            Value::String(s) => {
                // String access by index
                let idx = index_val.to_int();
                if idx >= 0 && (idx as usize) < s.len() {
                    Ok(Value::String(s.chars().nth(idx as usize).unwrap().to_string()))
                } else {
                    Ok(Value::String(String::new()))
                }
            }
            _ => Ok(Value::Null),
        }
    }

    fn eval_array_assign(
        &mut self,
        array_expr: &Expr,
        index: &Option<Box<Expr>>,
        op: &AssignOp,
        value_expr: &Expr,
    ) -> Result<Value, String> {
        let new_value = self.eval_expr(value_expr)?;

        // Get the variable name from the array expression
        let var_name = match array_expr {
            Expr::Variable(name) => name.clone(),
            Expr::ArrayAccess { array, .. } => {
                // Nested array access - get the root variable
                let mut current: &Expr = array;
                while let Expr::ArrayAccess { array: inner, .. } = current {
                    current = inner;
                }
                if let Expr::Variable(name) = current {
                    name.clone()
                } else {
                    return Err("Cannot assign to non-variable array".to_string());
                }
            }
            _ => return Err("Cannot assign to non-variable array".to_string()),
        };

        // Get or create the array
        let mut arr = match self.variables.get(&var_name).cloned() {
            Some(Value::Array(a)) => a,
            Some(_) => return Err("Cannot use array assignment on non-array".to_string()),
            None => Vec::new(),
        };

        // For nested access, we need to traverse and update
        if let Expr::ArrayAccess { index: outer_index, .. } = array_expr {
            // This is nested: $arr[outer][index] = value
            // We need to handle this recursively
            let outer_key = ArrayKey::from_value(&self.eval_expr(outer_index)?);

            // Find or create the inner array
            let inner_arr_idx = arr.iter().position(|(k, _)| k == &outer_key);

            let inner_arr = if let Some(idx) = inner_arr_idx {
                if let Value::Array(ref inner) = arr[idx].1 {
                    inner.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            let mut new_inner = inner_arr;

            // Apply the assignment to the inner array
            let key = if let Some(idx_expr) = index {
                ArrayKey::from_value(&self.eval_expr(idx_expr)?)
            } else {
                // Append: find max int key + 1
                let max_key = new_inner
                    .iter()
                    .filter_map(|(k, _)| if let ArrayKey::Integer(n) = k { Some(*n) } else { None })
                    .max()
                    .unwrap_or(-1);
                ArrayKey::Integer(max_key + 1)
            };

            let final_value = self.apply_assign_op(op, &new_inner, &key, new_value.clone())?;

            // Update or add the element
            let pos = new_inner.iter().position(|(k, _)| k == &key);
            if let Some(idx) = pos {
                new_inner[idx].1 = final_value.clone();
            } else {
                new_inner.push((key, final_value.clone()));
            }

            // Update or add the inner array in the outer array
            if let Some(idx) = inner_arr_idx {
                arr[idx].1 = Value::Array(new_inner);
            } else {
                arr.push((outer_key, Value::Array(new_inner)));
            }

            self.variables.insert(var_name, Value::Array(arr));
            return Ok(final_value);
        }

        // Simple case: $arr[index] = value or $arr[] = value
        let key = if let Some(idx_expr) = index {
            ArrayKey::from_value(&self.eval_expr(idx_expr)?)
        } else {
            // Append: find max int key + 1
            let max_key = arr
                .iter()
                .filter_map(|(k, _)| if let ArrayKey::Integer(n) = k { Some(*n) } else { None })
                .max()
                .unwrap_or(-1);
            ArrayKey::Integer(max_key + 1)
        };

        let final_value = self.apply_assign_op(op, &arr, &key, new_value)?;

        // Update or add the element
        let pos = arr.iter().position(|(k, _)| k == &key);
        if let Some(idx) = pos {
            arr[idx].1 = final_value.clone();
        } else {
            arr.push((key, final_value.clone()));
        }

        self.variables.insert(var_name, Value::Array(arr));
        Ok(final_value)
    }

    fn apply_assign_op(
        &self,
        op: &AssignOp,
        arr: &[(ArrayKey, Value)],
        key: &ArrayKey,
        new_value: Value,
    ) -> Result<Value, String> {
        match op {
            AssignOp::Assign => Ok(new_value),
            _ => {
                // Get current value for compound assignment
                let current = arr
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null);

                match op {
                    AssignOp::AddAssign => {
                        self.numeric_op(&current, &new_value, |a, b| a + b, |a, b| a + b)
                    }
                    AssignOp::SubAssign => {
                        self.numeric_op(&current, &new_value, |a, b| a - b, |a, b| a - b)
                    }
                    AssignOp::MulAssign => {
                        self.numeric_op(&current, &new_value, |a, b| a * b, |a, b| a * b)
                    }
                    AssignOp::DivAssign => {
                        let right_f = new_value.to_float();
                        if right_f == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        let result = current.to_float() / right_f;
                        if result.fract() == 0.0 {
                            Ok(Value::Integer(result as i64))
                        } else {
                            Ok(Value::Float(result))
                        }
                    }
                    AssignOp::ModAssign => {
                        let right_i = new_value.to_int();
                        if right_i == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(Value::Integer(current.to_int() % right_i))
                    }
                    AssignOp::ConcatAssign => Ok(Value::String(format!(
                        "{}{}",
                        current.to_string_val(),
                        new_value.to_string_val()
                    ))),
                    AssignOp::Assign => unreachable!(),
                }
            }
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
            "is_array" => builtins::types::is_array(&arg_values),
            "is_numeric" => builtins::types::is_numeric(&arg_values),
            "isset" => builtins::types::isset(&arg_values),
            "empty" => builtins::types::empty(&arg_values),

            // Output functions
            "print" => builtins::output::print(&mut self.output, &arg_values),
            "var_dump" => builtins::output::var_dump(&mut self.output, &arg_values),
            "print_r" => builtins::output::print_r(&mut self.output, &arg_values),

            // Array functions
            "count" | "sizeof" => builtins::array::count(&arg_values),
            "array_push" => builtins::array::array_push(&arg_values),
            "array_pop" => builtins::array::array_pop(&arg_values),
            "array_shift" => builtins::array::array_shift(&arg_values),
            "array_unshift" => builtins::array::array_unshift(&arg_values),
            "array_keys" => builtins::array::array_keys(&arg_values),
            "array_values" => builtins::array::array_values(&arg_values),
            "in_array" => builtins::array::in_array(&arg_values),
            "array_search" => builtins::array::array_search(&arg_values),
            "array_reverse" => builtins::array::array_reverse(&arg_values),
            "array_merge" => builtins::array::array_merge(&arg_values),
            "array_key_exists" => builtins::array::array_key_exists(&arg_values),
            "range" => builtins::array::range(&arg_values),

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

    /// Evaluate object instantiation (new ClassName(...))
    fn eval_new(&mut self, class_name: &str, args: &[Expr]) -> Result<Value, String> {
        let class_name_lower = class_name.to_lowercase();

        // Look up class definition
        let class_def = self
            .classes
            .get(&class_name_lower)
            .cloned()
            .ok_or_else(|| format!("Class '{}' not found", class_name))?;

        // Create new object instance
        let mut instance = ObjectInstance::new(class_def.name.clone());

        // Initialize properties with default values
        for prop in &class_def.properties {
            let default_val = if let Some(ref default_expr) = prop.default {
                self.eval_expr(default_expr)?
            } else {
                Value::Null
            };
            instance.properties.insert(prop.name.clone(), default_val);
        }

        // Check for constructor (__construct)
        if let Some(constructor) = class_def.methods.get("__construct") {
            // Evaluate constructor arguments
            let mut arg_values = Vec::new();
            for arg in args {
                arg_values.push(self.eval_expr(arg)?);
            }

            // Call constructor with $this bound
            self.call_method_on_object(&mut instance, constructor, &arg_values)?;
        }

        Ok(Value::Object(instance))
    }

    /// Evaluate property access ($obj->property)
    fn eval_property_access(&mut self, object: &Expr, property: &str) -> Result<Value, String> {
        let obj_value = self.eval_expr(object)?;

        match obj_value {
            Value::Object(instance) => {
                instance
                    .properties
                    .get(property)
                    .cloned()
                    .ok_or_else(|| format!("Undefined property: {}", property))
            }
            _ => Err(format!(
                "Cannot access property on non-object ({})",
                obj_value.get_type()
            )),
        }
    }

    /// Evaluate method call ($obj->method(...))
    fn eval_method_call(
        &mut self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, String> {
        // Get the variable name if object is a variable, so we can update it after the method call
        let var_name = match object {
            Expr::Variable(name) => Some(name.clone()),
            _ => None,
        };

        let obj_value = self.eval_expr(object)?;

        match obj_value {
            Value::Object(mut instance) => {
                let class_name_lower = instance.class_name.to_lowercase();

                // Look up class definition
                let class_def = self
                    .classes
                    .get(&class_name_lower)
                    .cloned()
                    .ok_or_else(|| format!("Class '{}' not found", instance.class_name))?;

                let method_lower = method.to_lowercase();

                // Look up method
                let method_func = class_def
                    .methods
                    .get(&method_lower)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Call to undefined method {}::{}()",
                            instance.class_name, method
                        )
                    })?;

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.eval_expr(arg)?);
                }

                // Call method with $this bound
                let result = self.call_method_on_object(&mut instance, &method_func, &arg_values)?;

                // Write back the modified instance to the variable if applicable
                if let Some(name) = var_name {
                    self.variables.insert(name, Value::Object(instance));
                }

                Ok(result)
            }
            _ => Err(format!(
                "Cannot call method on non-object ({})",
                obj_value.get_type()
            )),
        }
    }

    /// Evaluate property assignment ($obj->property = value)
    fn eval_property_assign(
        &mut self,
        object: &Expr,
        property: &str,
        value: &Expr,
    ) -> Result<Value, String> {
        // For property assignment, we need to handle $this specially
        match object {
            Expr::This => {
                // Evaluate value first to avoid borrow conflicts
                let val = self.eval_expr(value)?;
                if let Some(ref mut obj) = self.current_object {
                    obj.properties.insert(property.to_string(), val.clone());
                    Ok(val)
                } else {
                    Err("Cannot use $this outside of object context".to_string())
                }
            }
            Expr::Variable(var_name) => {
                // Evaluate value first
                let val = self.eval_expr(value)?;
                // Get the object from variable
                if let Some(Value::Object(mut instance)) = self.variables.get(var_name).cloned() {
                    instance.properties.insert(property.to_string(), val.clone());
                    self.variables
                        .insert(var_name.clone(), Value::Object(instance));
                    Ok(val)
                } else {
                    Err(format!(
                        "Cannot access property on non-object variable ${}",
                        var_name
                    ))
                }
            }
            _ => {
                // For other expressions, evaluate and try to assign
                let obj_value = self.eval_expr(object)?;
                match obj_value {
                    Value::Object(_) => Err(
                        "Cannot assign property on temporary object expression".to_string(),
                    ),
                    _ => Err(format!(
                        "Cannot access property on non-object ({})",
                        obj_value.get_type()
                    )),
                }
            }
        }
    }

    /// Evaluate static method call (ClassName::method(...))
    fn eval_static_method_call(
        &mut self,
        class_name: &str,
        method: &str,
        args: &[Expr],
    ) -> Result<Value, String> {
        let class_name_lower = class_name.to_lowercase();

        // Look up class definition
        let class_def = self
            .classes
            .get(&class_name_lower)
            .cloned()
            .ok_or_else(|| format!("Class '{}' not found", class_name))?;

        let method_lower = method.to_lowercase();

        // Look up method
        let method_func = class_def.methods.get(&method_lower).cloned().ok_or_else(|| {
            format!(
                "Call to undefined method {}::{}()",
                class_def.name, method
            )
        })?;

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.eval_expr(arg)?);
        }

        // Call method without $this (static call)
        self.call_user_function(&method_func, &arg_values)
    }

    /// Call a method on an object instance
    fn call_method_on_object(
        &mut self,
        instance: &mut ObjectInstance,
        method: &UserFunction,
        args: &[Value],
    ) -> Result<Value, String> {
        // Save current state
        let saved_variables = self.variables.clone();
        let saved_current_object = self.current_object.take();

        // Set current object to this instance
        self.current_object = Some(instance.clone());

        // Clear variables and set parameters
        self.variables.clear();

        // Bind arguments to parameters
        for (i, param) in method.params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(ref default_expr) = param.default {
                self.eval_expr(default_expr)?
            } else {
                Value::Null
            };
            self.variables.insert(param.name.clone(), value);
        }

        // Execute method body
        let mut return_value = Value::Null;
        for stmt in &method.body {
            let cf = self.execute_stmt(stmt).map_err(|e| e.to_string())?;
            match cf {
                ControlFlow::Return(v) => {
                    return_value = v;
                    break;
                }
                ControlFlow::Break | ControlFlow::Continue => break,
                ControlFlow::None => {}
            }
        }

        // Copy back any property changes from $this
        if let Some(ref obj) = self.current_object {
            *instance = obj.clone();
        }

        // Restore previous state
        self.variables = saved_variables;
        self.current_object = saved_current_object;

        Ok(return_value)
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
                array,
                key,
                value,
                body,
            } => {
                let array_val = self.eval_expr(array).map_err(|e| {
                    io::Error::other(e)
                })?;

                match array_val {
                    Value::Array(arr) => {
                        for (k, v) in arr {
                            // Bind key if specified
                            if let Some(key_name) = key {
                                self.variables.insert(key_name.clone(), k.to_value());
                            }

                            // Bind value
                            self.variables.insert(value.clone(), v);

                            // Execute body
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
                    _ => {
                        // PHP would emit a warning here, we just skip
                        Ok(ControlFlow::None)
                    }
                }
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
            Stmt::Class {
                name,
                parent,
                properties,
                methods,
            } => {
                // Build methods map
                let mut methods_map = HashMap::new();
                let mut visibility_map = HashMap::new();

                for method in methods {
                    let func = UserFunction {
                        params: method.params.clone(),
                        body: method.body.clone(),
                    };
                    let method_name_lower = method.name.to_lowercase();
                    methods_map.insert(method_name_lower.clone(), func);
                    visibility_map.insert(method_name_lower, method.visibility);
                }

                let class_def = ClassDefinition {
                    name: name.clone(),
                    parent: parent.clone(),
                    properties: properties.clone(),
                    methods: methods_map,
                    method_visibility: visibility_map,
                };

                // Store class definition (case-insensitive)
                self.classes.insert(name.to_lowercase(), class_def);
                Ok(ControlFlow::None)
            }
        }
    }
}

impl Default for Interpreter<io::Stdout> {
    fn default() -> Self {
        Self::new(io::stdout())
    }
}

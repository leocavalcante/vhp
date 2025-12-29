//! Expression evaluation implementation for the interpreter
//!
//! This module handles evaluating all expression types including:
//! - Literals (null, bool, integer, float, string)
//! - Variables
//! - Arrays and array access
//! - Unary and binary operations
//! - Match expressions
//! - Clone expressions
//! - Pipe operator

use crate::ast::{ArrayElement, AssignOp, BinaryOp, Expr, MatchArm, PropertyModification, UnaryOp};
use crate::interpreter::value::{ArrayKey, Value};
use crate::interpreter::Interpreter;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Null => Ok(Value::Null),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            Expr::Variable(name) => Ok(self.variables.get(name).cloned().unwrap_or(Value::Null)),

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

            Expr::PropertyAccess { object, property } => {
                self.eval_property_access(object, property)
            }

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

            Expr::Match {
                expr,
                arms,
                default,
            } => self.eval_match(expr, arms, default),

            Expr::EnumCase {
                enum_name,
                case_name,
            } => self.eval_enum_case(enum_name, case_name),

            Expr::Clone { object } => self.eval_clone(object),

            Expr::CloneWith {
                object,
                modifications,
            } => self.eval_clone_with(object, modifications),

            Expr::Placeholder => {
                // Placeholder is only valid inside pipe operator argument lists
                // If we reach here, it's an error
                Err(
                    "Placeholder (...) can only be used in pipe operator function calls"
                        .to_string(),
                )
            }
        }
    }

    fn eval_array(&mut self, elements: &[ArrayElement]) -> Result<Value, String> {
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
                    Ok(Value::String(
                        s.chars().nth(idx as usize).unwrap().to_string(),
                    ))
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
        if let Expr::ArrayAccess {
            index: outer_index, ..
        } = array_expr
        {
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
                    .filter_map(|(k, _)| {
                        if let ArrayKey::Integer(n) = k {
                            Some(*n)
                        } else {
                            None
                        }
                    })
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
                .filter_map(|(k, _)| {
                    if let ArrayKey::Integer(n) = k {
                        Some(*n)
                    } else {
                        None
                    }
                })
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

    pub(super) fn apply_assign_op(
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
        // Special handling for operators that need unevaluated right side
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
            BinaryOp::Pipe => {
                // Pipe operator needs special handling: right side should not be evaluated as normal expression
                return self.eval_pipe(left, right);
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
            BinaryOp::And | BinaryOp::Or | BinaryOp::NullCoalesce | BinaryOp::Pipe => {
                unreachable!()
            }
        }
    }

    pub(super) fn numeric_op<F, G>(
        &self,
        left: &Value,
        right: &Value,
        int_op: F,
        float_op: G,
    ) -> Result<Value, String>
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

    pub(super) fn eval_match(
        &mut self,
        expr: &Expr,
        arms: &[MatchArm],
        default: &Option<Box<Expr>>,
    ) -> Result<Value, String> {
        let match_value = self.eval_expr(expr)?;

        // Try each arm
        for arm in arms {
            // Check if any condition matches (using strict identity comparison)
            for condition in &arm.conditions {
                let cond_value = self.eval_expr(condition)?;
                if match_value.type_equals(&cond_value) {
                    return self.eval_expr(&arm.result);
                }
            }
        }

        // No match found, try default
        if let Some(default_expr) = default {
            return self.eval_expr(default_expr);
        }

        // PHP 8 throws UnhandledMatchError if no match and no default
        Err(format!(
            "Unhandled match value: {}",
            match_value.to_output_string()
        ))
    }

    pub(super) fn eval_enum_case(&self, enum_name: &str, case_name: &str) -> Result<Value, String> {
        let enum_name_lower = enum_name.to_lowercase();

        // Look up enum definition
        let enum_def = self
            .enums
            .get(&enum_name_lower)
            .ok_or_else(|| format!("Undefined enum '{}'", enum_name))?;

        // Find the case
        for (name, value) in &enum_def.cases {
            if name == case_name {
                return Ok(Value::EnumCase {
                    enum_name: enum_def.name.clone(),
                    case_name: name.clone(),
                    backing_value: value.as_ref().map(|v| Box::new(v.clone())),
                });
            }
        }

        Err(format!(
            "Undefined case '{}' for enum '{}'",
            case_name, enum_name
        ))
    }

    pub(super) fn eval_clone(&mut self, object_expr: &Expr) -> Result<Value, String> {
        let object_value = self.eval_expr(object_expr)?;

        match object_value {
            Value::Object(instance) => {
                // Create a deep clone of the object
                let cloned_instance = crate::interpreter::ObjectInstance {
                    class_name: instance.class_name.clone(),
                    properties: instance.properties.clone(),
                    readonly_properties: instance.readonly_properties.clone(),
                    initialized_readonly: std::collections::HashSet::new(), // Reset initialization tracking
                };

                // For a cloned object, readonly properties can be re-initialized
                // This is PHP's behavior: clone creates a new object context
                Ok(Value::Object(cloned_instance))
            }
            _ => Err(format!(
                "__clone method called on non-object ({})",
                object_value.get_type()
            )),
        }
    }

    pub(super) fn eval_clone_with(
        &mut self,
        object_expr: &Expr,
        modifications: &[PropertyModification],
    ) -> Result<Value, String> {
        let object_value = self.eval_expr(object_expr)?;

        match object_value {
            Value::Object(instance) => {
                // Create a deep clone of the object
                let mut cloned_instance = crate::interpreter::ObjectInstance {
                    class_name: instance.class_name.clone(),
                    properties: instance.properties.clone(),
                    readonly_properties: instance.readonly_properties.clone(),
                    initialized_readonly: std::collections::HashSet::new(), // Reset for clone
                };

                // Apply modifications
                for modification in modifications {
                    let property_name = &modification.property;

                    // Check if property exists in the original object
                    if !cloned_instance.properties.contains_key(property_name) {
                        return Err(format!(
                            "Property '{}' does not exist on class '{}'",
                            property_name, cloned_instance.class_name
                        ));
                    }

                    // Evaluate the new value
                    let new_value = self.eval_expr(&modification.value)?;

                    // Set the property value
                    cloned_instance
                        .properties
                        .insert(property_name.clone(), new_value);

                    // Mark readonly property as initialized if it's readonly
                    if cloned_instance.readonly_properties.contains(property_name) {
                        cloned_instance
                            .initialized_readonly
                            .insert(property_name.clone());
                    }
                }

                Ok(Value::Object(cloned_instance))
            }
            _ => Err(format!(
                "Clone with called on non-object ({})",
                object_value.get_type()
            )),
        }
    }

    pub(super) fn eval_pipe(&mut self, left: &Expr, right: &Expr) -> Result<Value, String> {
        // Evaluate the left side to get the value to pipe
        let piped_value = self.eval_expr(left)?;

        // The right side must be a function call
        match right {
            Expr::FunctionCall { name, args } => {
                // Find placeholder position
                let placeholder_pos = args
                    .iter()
                    .position(|arg| matches!(&*arg.value, Expr::Placeholder));

                let mut arg_values = Vec::new();

                if let Some(pos) = placeholder_pos {
                    // Placeholder found: insert piped value at that position
                    for (i, arg) in args.iter().enumerate() {
                        if i == pos {
                            arg_values.push(piped_value.clone());
                        } else {
                            arg_values.push(self.eval_expr(&arg.value)?);
                        }
                    }
                } else {
                    // No placeholder: insert piped value as first argument
                    arg_values.push(piped_value);
                    for arg in args {
                        arg_values.push(self.eval_expr(&arg.value)?);
                    }
                }

                // Call the function with the modified argument list
                self.call_function_with_values(name, &arg_values)
            }

            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let object_value = self.eval_expr(object)?;

                // Evaluate arguments with piped value as first
                let mut arg_values = vec![piped_value];
                for arg in args {
                    arg_values.push(self.eval_expr(&arg.value)?);
                }

                match object_value {
                    Value::Object(instance) => {
                        let method_lower = method.to_lowercase();

                        // Look up the method in the class definition
                        if let Some(class_def) = self
                            .classes
                            .get(&instance.class_name.to_lowercase())
                            .cloned()
                        {
                            if let Some(method_func) = class_def.methods.get(&method_lower) {
                                // Set current object context
                                let saved_object = self.current_object.clone();
                                let saved_class = self.current_class.clone();
                                self.current_object = Some(instance.clone());
                                self.current_class = Some(class_def.name.clone());

                                // Call the method
                                let result = self.call_user_function(method_func, &arg_values);

                                // Restore context
                                self.current_object = saved_object;
                                self.current_class = saved_class;

                                return result;
                            }

                            return Err(format!(
                                "Method '{}' not found on class '{}'",
                                method, class_def.name
                            ));
                        }

                        Err(format!("Class '{}' not found", instance.class_name))
                    }
                    _ => Err(format!(
                        "Attempting to call method on non-object ({})",
                        object_value.get_type()
                    )),
                }
            }

            _ => Err(format!(
                "Pipe operator right-hand side must be a function call or method call, got {:?}",
                right
            )),
        }
    }
}

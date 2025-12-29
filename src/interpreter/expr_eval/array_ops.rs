//! Array operations for expression evaluation
//!
//! Handles:
//! - Array literal evaluation
//! - Array element access
//! - Array element assignment (including nested arrays)

use crate::ast::{ArrayElement, AssignOp, Expr};
use crate::interpreter::value::{ArrayKey, Value};
use crate::interpreter::Interpreter;
use std::io::Write;

/// Evaluate array literal: [key => value, ...]
pub(crate) fn eval_array<W: Write>(
    interpreter: &mut Interpreter<W>,
    elements: &[ArrayElement],
) -> Result<Value, String> {
    let mut arr = Vec::new();
    let mut next_int_key: i64 = 0;

    for elem in elements {
        let key = if let Some(key_expr) = &elem.key {
            let key_val = interpreter.eval_expr(key_expr)?;
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

        let value = interpreter.eval_expr(&elem.value)?;
        arr.push((key, value));
    }

    Ok(Value::Array(arr))
}

/// Evaluate array access: $arr[index]
pub(crate) fn eval_array_access<W: Write>(
    interpreter: &mut Interpreter<W>,
    array: &Expr,
    index: &Expr,
) -> Result<Value, String> {
    let array_val = interpreter.eval_expr(array)?;
    let index_val = interpreter.eval_expr(index)?;
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

/// Evaluate array element assignment: $arr[index] = value or $arr[] = value
pub(crate) fn eval_array_assign<W: Write>(
    interpreter: &mut Interpreter<W>,
    array_expr: &Expr,
    index: &Option<Box<Expr>>,
    op: &AssignOp,
    value_expr: &Expr,
) -> Result<Value, String> {
    let new_value = interpreter.eval_expr(value_expr)?;

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
    let mut arr = match interpreter.variables.get(&var_name).cloned() {
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
        let outer_key = ArrayKey::from_value(&interpreter.eval_expr(outer_index)?);

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
            ArrayKey::from_value(&interpreter.eval_expr(idx_expr)?)
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

        let final_value =
            apply_array_assign_op(interpreter, op, &new_inner, &key, new_value.clone())?;

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

        interpreter.variables.insert(var_name, Value::Array(arr));
        return Ok(final_value);
    }

    // Simple case: $arr[index] = value or $arr[] = value
    let key = if let Some(idx_expr) = index {
        ArrayKey::from_value(&interpreter.eval_expr(idx_expr)?)
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

    let final_value = apply_array_assign_op(interpreter, op, &arr, &key, new_value)?;

    // Update or add the element
    let pos = arr.iter().position(|(k, _)| k == &key);
    if let Some(idx) = pos {
        arr[idx].1 = final_value.clone();
    } else {
        arr.push((key, final_value.clone()));
    }

    interpreter.variables.insert(var_name, Value::Array(arr));
    Ok(final_value)
}

/// Helper: apply assignment operator to array element
fn apply_array_assign_op<W: Write>(
    interpreter: &Interpreter<W>,
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

            interpreter.apply_compound_assign_op(&current, op, &new_value)
        }
    }
}

//! Binary operator evaluation
//!
//! Handles:
//! - Arithmetic operators: +, -, *, /, %, **
//! - String concatenation: .
//! - Comparison operators: ==, ===, !=, !==, <, >, <=, >=, <=>
//! - Logical operators: &&, ||, and, or, xor
//! - Null coalescing: ??
//! - Short-circuit evaluation for logical operators

use crate::ast::{BinaryOp, Expr};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

/// Evaluate binary operations
pub(crate) fn eval_binary<W: Write>(
    interpreter: &mut Interpreter<W>,
    left: &Expr,
    op: &BinaryOp,
    right: &Expr,
) -> Result<Value, String> {
    // Special handling for operators that need unevaluated right side (short-circuit)
    match op {
        BinaryOp::And => {
            let left_val = interpreter.eval_expr(left)?;
            if !left_val.to_bool() {
                return Ok(Value::Bool(false));
            }
            let right_val = interpreter.eval_expr(right)?;
            return Ok(Value::Bool(right_val.to_bool()));
        }
        BinaryOp::Or => {
            let left_val = interpreter.eval_expr(left)?;
            if left_val.to_bool() {
                return Ok(Value::Bool(true));
            }
            let right_val = interpreter.eval_expr(right)?;
            return Ok(Value::Bool(right_val.to_bool()));
        }
        BinaryOp::NullCoalesce => {
            let left_val = interpreter.eval_expr(left)?;
            if !matches!(left_val, Value::Null) {
                return Ok(left_val);
            }
            return interpreter.eval_expr(right);
        }
        BinaryOp::Pipe => {
            // Pipe operator is handled in special_ops
            return super::special_ops::eval_pipe(interpreter, left, right);
        }
        _ => {}
    }

    // For all other operators, evaluate both sides first
    let left_val = interpreter.eval_expr(left)?;
    let right_val = interpreter.eval_expr(right)?;

    match op {
        // Arithmetic operators
        BinaryOp::Add => interpreter.numeric_op(&left_val, &right_val, |a, b| a + b, |a, b| a + b),
        BinaryOp::Sub => interpreter.numeric_op(&left_val, &right_val, |a, b| a - b, |a, b| a - b),
        BinaryOp::Mul => interpreter.numeric_op(&left_val, &right_val, |a, b| a * b, |a, b| a * b),
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

        // String concatenation
        BinaryOp::Concat => Ok(Value::String(format!(
            "{}{}",
            left_val.to_string_val(),
            right_val.to_string_val()
        ))),

        // Comparison operators
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

        // Logical operator (non-short-circuit case)
        BinaryOp::Xor => Ok(Value::Bool(left_val.to_bool() ^ right_val.to_bool())),

        // Bitwise operators
        BinaryOp::BitwiseOr => Ok(Value::Integer(left_val.to_int() | right_val.to_int())),

        // Already handled above with short-circuit
        BinaryOp::And | BinaryOp::Or | BinaryOp::NullCoalesce | BinaryOp::Pipe => {
            unreachable!()
        }
    }
}

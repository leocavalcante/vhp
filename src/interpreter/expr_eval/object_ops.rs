//! Object/OOP operations for expression evaluation
//!
//! Handles:
//! - Object instantiation (new ClassName)
//! - Property access and assignment
//! - Method calls
//! - Static method calls

use crate::ast::{Argument, Expr};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

/// Evaluate object instantiation (new ClassName(...))
pub(crate) fn eval_new<W: Write>(
    interpreter: &mut Interpreter<W>,
    class_name: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_new in objects.rs
    interpreter.eval_new(class_name, args)
}

/// Evaluate property access ($obj->property)
pub(crate) fn eval_property_access<W: Write>(
    interpreter: &mut Interpreter<W>,
    object: &Expr,
    property: &str,
) -> Result<Value, String> {
    // Delegate to the existing eval_property_access in objects.rs
    interpreter.eval_property_access(object, property)
}

/// Evaluate property assignment ($obj->property = value)
pub(crate) fn eval_property_assign<W: Write>(
    interpreter: &mut Interpreter<W>,
    object_expr: &Expr,
    property: &str,
    value_expr: &Expr,
) -> Result<Value, String> {
    // Delegate to the existing eval_property_assign in objects.rs
    interpreter.eval_property_assign(object_expr, property, value_expr)
}

/// Evaluate method call ($obj->method(...))
pub(crate) fn eval_method_call<W: Write>(
    interpreter: &mut Interpreter<W>,
    object: &Expr,
    method: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_method_call in objects.rs
    interpreter.eval_method_call(object, method, args)
}

/// Evaluate static method call (ClassName::method(...))
pub(crate) fn eval_static_method_call<W: Write>(
    interpreter: &mut Interpreter<W>,
    class_name: &str,
    method: &str,
    args: &[Argument],
) -> Result<Value, String> {
    // Delegate to the existing eval_static_method_call in objects.rs
    interpreter.eval_static_method_call(class_name, method, args)
}

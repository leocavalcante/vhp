//! Expression evaluation implementation for the interpreter
//!
//! This module is split into focused submodules:
//! - mod.rs: Main dispatcher and simple literals
//! - array_ops.rs: Array operations (literals, access, assignment)
//! - binary_ops.rs: Binary operators (arithmetic, comparison, logical)
//! - object_ops.rs: Object operations (instantiation, properties, methods)
//! - special_ops.rs: Advanced features (match, enum, clone, pipe)

mod array_ops;
mod binary_ops;
mod object_ops;
mod special_ops;

// Re-export module functions
pub(super) use array_ops::{eval_array, eval_array_access, eval_array_assign};
pub(super) use binary_ops::eval_binary;
pub(super) use special_ops::{eval_clone, eval_clone_with, eval_enum_case, eval_match};

use crate::ast::{AssignOp, Expr, UnaryOp};
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;
use std::io::Write;

impl<W: Write> Interpreter<W> {
    /// Main expression dispatcher
    pub(super) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            // Simple literals
            Expr::Null => Ok(Value::Null),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),

            // Variables
            Expr::Variable(name) => Ok(self.variables.get(name).cloned().unwrap_or(Value::Null)),

            // Grouped expressions
            Expr::Grouped(inner) => self.eval_expr(inner),

            // Ternary operator
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

            // Assignment
            Expr::Assign { var, op, value } => self.eval_assign(var, op, value),

            // Unary operations
            Expr::Unary { op, expr } => self.eval_unary(op, expr),

            // Array operations
            Expr::Array(elements) => eval_array(self, elements),
            Expr::ArrayAccess { array, index } => eval_array_access(self, array, index),
            Expr::ArrayAssign {
                array,
                index,
                op,
                value,
            } => eval_array_assign(self, array, index, op, value),

            // Binary operations
            Expr::Binary { left, op, right } => eval_binary(self, left, op, right),

            // Object/OOP operations
            Expr::New { class_name, args } => object_ops::eval_new(self, class_name, args),
            Expr::NewAnonymousClass {
                constructor_args,
                parent,
                interfaces,
                traits,
                properties,
                methods,
            } => object_ops::eval_new_anonymous_class(
                self,
                constructor_args,
                parent,
                interfaces,
                traits,
                properties,
                methods,
            ),
            Expr::PropertyAccess { object, property } => {
                object_ops::eval_property_access(self, object, property)
            }
            Expr::PropertyAssign {
                object,
                property,
                value,
            } => object_ops::eval_property_assign(self, object, property, value),
            Expr::MethodCall {
                object,
                method,
                args,
            } => object_ops::eval_method_call(self, object, method, args),
            Expr::StaticMethodCall {
                class_name,
                method,
                args,
            } => object_ops::eval_static_method_call(self, class_name, method, args),
            Expr::StaticPropertyAccess { class, property } => {
                self.get_static_property(class, property)
            }
            Expr::StaticPropertyAssign {
                class,
                property,
                value,
            } => {
                let val = self.eval_expr(value)?;
                self.set_static_property(class, property, val.clone())?;
                Ok(val)
            }
            Expr::This => {
                if let Some(ref obj) = self.current_object {
                    Ok(Value::Object(obj.clone()))
                } else {
                    Err("Cannot use $this outside of object context".to_string())
                }
            }

            // Function calls
            Expr::FunctionCall { name, args } => self.call_function(name, args),
            Expr::CallableCall { callable, args } => self.eval_callable_call(callable, args),

            // Fiber expressions
            Expr::NewFiber { callback } => self.eval_new_fiber(callback),
            Expr::FiberSuspend { value } => {
                self.eval_fiber_suspend(value.as_ref().map(|v| v.as_ref()))
            }
            Expr::FiberGetCurrent => Ok(self.eval_fiber_get_current()),

            // Special expressions
            Expr::Match {
                expr,
                arms,
                default,
            } => eval_match(self, expr, arms, default),
            Expr::EnumCase {
                enum_name,
                case_name,
            } => eval_enum_case(self, enum_name, case_name),
            Expr::Clone { object } => eval_clone(self, object),
            Expr::CloneWith {
                object,
                modifications,
            } => eval_clone_with(self, object, modifications),
            Expr::Placeholder => {
                // Placeholder is only valid inside pipe operator argument lists
                Err(
                    "Placeholder (...) can only be used in pipe operator function calls"
                        .to_string(),
                )
            }
            Expr::Spread(_) => {
                // Spread is only valid inside function call argument lists
                Err("Spread operator (...) can only be used in function call arguments".to_string())
            }
            Expr::ArrowFunction { params, body } => self.eval_arrow_function(params, body),
            Expr::CallableFromFunction(name) => self.eval_callable_from_function(name),
            Expr::CallableFromMethod { object, method } => {
                self.eval_callable_from_method(object, method)
            }
            Expr::CallableFromStaticMethod { class, method } => {
                self.eval_callable_from_static_method(class, method)
            }
            Expr::Throw(expr) => {
                // Throw expression - evaluate and return special error that can be caught
                let value = self.eval_expr(expr)?;
                match value {
                    Value::Exception(exc) => {
                        Err(format!("__EXCEPTION__:{}:{}", exc.class_name, exc.message))
                    }
                    Value::Object(obj) => {
                        let message = obj
                            .properties
                            .get("message")
                            .and_then(|v| match v {
                                Value::String(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        Err(format!("__EXCEPTION__:{}:{}", obj.class_name, message))
                    }
                    _ => Err("Can only throw objects that extend Exception".to_string()),
                }
            }
        }
    }

    /// Handle variable assignment (including compound assignments)
    fn eval_assign(&mut self, var: &str, op: &AssignOp, value: &Expr) -> Result<Value, String> {
        let new_value = self.eval_expr(value)?;

        let final_value = match op {
            AssignOp::Assign => new_value,
            _ => {
                // Compound assignment: get current value first
                let current = self.variables.get(var).cloned().unwrap_or(Value::Null);
                self.apply_compound_assign_op(&current, op, &new_value)?
            }
        };

        self.variables.insert(var.to_string(), final_value.clone());
        Ok(final_value)
    }

    /// Handle unary operators
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
                match expr {
                    Expr::Variable(name) => {
                        let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                        let new_val = match val {
                            Value::Integer(n) => Value::Integer(n + 1),
                            Value::Float(n) => Value::Float(n + 1.0),
                            _ => Value::Integer(val.to_int() + 1),
                        };
                        self.variables.insert(name.clone(), new_val.clone());
                        Ok(new_val)
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let val = self.get_static_property(class, property)?;
                        let new_val = match val {
                            Value::Integer(n) => Value::Integer(n + 1),
                            Value::Float(n) => Value::Float(n + 1.0),
                            _ => Value::Integer(val.to_int() + 1),
                        };
                        self.set_static_property(class, property, new_val.clone())?;
                        Ok(new_val)
                    }
                    _ => Err("Pre-increment requires a variable or static property".to_string()),
                }
            }
            UnaryOp::PreDec => {
                match expr {
                    Expr::Variable(name) => {
                        let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                        let new_val = match val {
                            Value::Integer(n) => Value::Integer(n - 1),
                            Value::Float(n) => Value::Float(n - 1.0),
                            _ => Value::Integer(val.to_int() - 1),
                        };
                        self.variables.insert(name.clone(), new_val.clone());
                        Ok(new_val)
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let val = self.get_static_property(class, property)?;
                        let new_val = match val {
                            Value::Integer(n) => Value::Integer(n - 1),
                            Value::Float(n) => Value::Float(n - 1.0),
                            _ => Value::Integer(val.to_int() - 1),
                        };
                        self.set_static_property(class, property, new_val.clone())?;
                        Ok(new_val)
                    }
                    _ => Err("Pre-decrement requires a variable or static property".to_string()),
                }
            }
            UnaryOp::PostInc => {
                match expr {
                    Expr::Variable(name) => {
                        let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                        let new_val = match &val {
                            Value::Integer(n) => Value::Integer(n + 1),
                            Value::Float(n) => Value::Float(n + 1.0),
                            _ => Value::Integer(val.to_int() + 1),
                        };
                        self.variables.insert(name.clone(), new_val);
                        Ok(val)
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let val = self.get_static_property(class, property)?;
                        let new_val = match &val {
                            Value::Integer(n) => Value::Integer(n + 1),
                            Value::Float(n) => Value::Float(n + 1.0),
                            _ => Value::Integer(val.to_int() + 1),
                        };
                        self.set_static_property(class, property, new_val)?;
                        Ok(val)
                    }
                    _ => Err("Post-increment requires a variable or static property".to_string()),
                }
            }
            UnaryOp::PostDec => {
                match expr {
                    Expr::Variable(name) => {
                        let val = self.variables.get(name).cloned().unwrap_or(Value::Null);
                        let new_val = match &val {
                            Value::Integer(n) => Value::Integer(n - 1),
                            Value::Float(n) => Value::Float(n - 1.0),
                            _ => Value::Integer(val.to_int() - 1),
                        };
                        self.variables.insert(name.clone(), new_val);
                        Ok(val)
                    }
                    Expr::StaticPropertyAccess { class, property } => {
                        let val = self.get_static_property(class, property)?;
                        let new_val = match &val {
                            Value::Integer(n) => Value::Integer(n - 1),
                            Value::Float(n) => Value::Float(n - 1.0),
                            _ => Value::Integer(val.to_int() - 1),
                        };
                        self.set_static_property(class, property, new_val)?;
                        Ok(val)
                    }
                    _ => Err("Post-decrement requires a variable or static property".to_string()),
                }
            }
        }
    }

    /// Helper: apply compound assignment operator
    pub(super) fn apply_compound_assign_op(
        &self,
        current: &Value,
        op: &AssignOp,
        new_value: &Value,
    ) -> Result<Value, String> {
        match op {
            AssignOp::Assign => unreachable!(),
            AssignOp::AddAssign => self.numeric_op(current, new_value, |a, b| a + b, |a, b| a + b),
            AssignOp::SubAssign => self.numeric_op(current, new_value, |a, b| a - b, |a, b| a - b),
            AssignOp::MulAssign => self.numeric_op(current, new_value, |a, b| a * b, |a, b| a * b),
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
        }
    }

    /// Helper: numeric operation with type coercion
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

    /// Evaluate arrow function (PHP 7.4): capture variables and create closure
    fn eval_arrow_function(
        &self,
        params: &[crate::ast::FunctionParam],
        body: &Expr,
    ) -> Result<Value, String> {
        use crate::interpreter::value::{Closure, ClosureBody};
        use std::collections::HashMap;

        // Capture ALL variables from current scope by value
        // This is the key feature of arrow functions
        let mut captured_vars = HashMap::new();
        for (name, value) in &self.variables {
            captured_vars.insert(name.clone(), value.clone());
        }

        // Create the closure value
        let closure = Closure {
            params: params.to_vec(),
            body: ClosureBody::Expression(Box::new(body.clone())),
            captured_vars,
        };

        Ok(Value::Closure(Box::new(closure)))
    }

    /// Evaluate callable call: $func(), where $func is a closure or string function name
    fn eval_callable_call(
        &mut self,
        callable: &Expr,
        args: &[crate::ast::Argument],
    ) -> Result<Value, String> {
        // Evaluate the callable expression
        let callable_value = self.eval_expr(callable)?;

        match callable_value {
            Value::Closure(closure) => {
                // Call the closure
                self.call_closure(&closure, args)
            }
            Value::String(func_name) => {
                // Variable function call: $func = "strlen"; $func("hello");
                self.call_function(&func_name, args)
            }
            Value::Object(mut obj) => {
                // Check for __invoke magic method
                let class = self.classes.get(&obj.class_name.to_lowercase()).cloned();
                if let Some(class) = class {
                    if let Some(method) = class.get_magic_method("__invoke") {
                        // Evaluate arguments
                        let arg_values: Result<Vec<_>, _> =
                            args.iter().map(|arg| self.eval_expr(&arg.value)).collect();
                        let arg_values = arg_values?;

                        let class_name = obj.class_name.clone();
                        return self.call_method_on_object(
                            &mut obj,
                            method,
                            &arg_values,
                            class_name,
                        );
                    }
                }
                Err(format!(
                    "Call to undefined method {}::__invoke()",
                    obj.class_name
                ))
            }
            _ => Err(format!(
                "Value of type {} is not callable",
                callable_value.get_type()
            )),
        }
    }

    /// Create closure from function name (PHP 8.1 first-class callable)
    fn eval_callable_from_function(&self, name: &str) -> Result<Value, String> {
        use crate::interpreter::value::{Closure, ClosureBody};
        use std::collections::HashMap;

        // Create a closure that references the function
        let closure = Closure {
            params: vec![], // Will be determined at call time
            body: ClosureBody::FunctionRef(name.to_string()),
            captured_vars: HashMap::new(),
        };

        Ok(Value::Closure(Box::new(closure)))
    }

    /// Create closure from method (PHP 8.1 first-class callable)
    fn eval_callable_from_method(&mut self, object: &Expr, method: &str) -> Result<Value, String> {
        use crate::interpreter::value::{Closure, ClosureBody};
        use std::collections::HashMap;

        // Evaluate the object
        let obj_value = self.eval_expr(object)?;

        // Verify it's an object
        if !matches!(obj_value, Value::Object(_)) {
            return Err("Cannot create callable from non-object".to_string());
        }

        // Create closure bound to this object and method
        let closure = Closure {
            params: vec![],
            body: ClosureBody::MethodRef {
                object: Box::new(obj_value),
                method: method.to_string(),
            },
            captured_vars: HashMap::new(),
        };

        Ok(Value::Closure(Box::new(closure)))
    }

    /// Create closure from static method (PHP 8.1 first-class callable)
    fn eval_callable_from_static_method(&self, class: &str, method: &str) -> Result<Value, String> {
        use crate::interpreter::value::{Closure, ClosureBody};
        use std::collections::HashMap;

        // Create closure referencing the static method
        let closure = Closure {
            params: vec![],
            body: ClosureBody::StaticMethodRef {
                class: class.to_string(),
                method: method.to_string(),
            },
            captured_vars: HashMap::new(),
        };

        Ok(Value::Closure(Box::new(closure)))
    }
}

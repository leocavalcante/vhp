//! Value operations and builtin function helpers for VM
//!
//! This module contains:
//! - Arithmetic operations on values
//! - Value comparison operations
//! - Builtin function dispatcher

use crate::runtime::Value;
use crate::vm::{builtins, reflection, VM};

impl<W: std::io::Write> VM<W> {
    pub fn add_values(&self, left: Value, right: Value) -> Result<Value, String> {
        match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                for (k, v) in b {
                    if !result.iter().any(|(key, _)| key == k) {
                        result.push((k.clone(), v.clone()));
                    }
                }
                Ok(Value::Array(result))
            }
            _ => {
                let a = left.to_float();
                let b = right.to_float();
                Ok(Value::Float(a + b))
            }
        }
    }

    pub fn compare_values(&self, left: &Value, right: &Value) -> Result<i64, String> {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok((*a).cmp(b) as i64),
            (Value::Float(a), Value::Float(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Integer(a), Value::Float(b)) => {
                let a = *a as f64;
                if a < *b {
                    Ok(-1)
                } else if a > *b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::Float(a), Value::Integer(b)) => {
                let b = *b as f64;
                if a < &b {
                    Ok(-1)
                } else if a > &b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::String(a), Value::String(b)) => Ok(a.cmp(b) as i64),
            _ => {
                let a = left.to_float();
                let b = right.to_float();
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
        }
    }

    pub fn call_reflection_or_builtin(
        &mut self,
        func_name: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        match func_name {
            "get_class_attributes" => {
                if args.is_empty() {
                    return Err("get_class_attributes() expects 1 argument".to_string());
                }
                let class_name = args[0].to_string_val();
                reflection::get_class_attributes(&class_name, &self.classes)
            }
            "get_property_attributes" => {
                if args.len() < 2 {
                    return Err("get_property_attributes() expects 2 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let property_name = args[1].to_string_val();
                reflection::get_property_attributes(&class_name, &property_name, &self.classes)
            }
            "get_method_attributes" => {
                if args.len() < 2 {
                    return Err("get_method_attributes() expects 2 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let method_name = args[1].to_string_val();
                reflection::get_method_attributes(&class_name, &method_name, &self.classes)
            }
            "get_method_parameter_attributes" => {
                if args.len() < 3 {
                    return Err("get_method_parameter_attributes() expects 3 arguments".to_string());
                }
                let class_name = args[0].to_string_val();
                let method_name = args[1].to_string_val();
                let parameter_name = args[2].to_string_val();
                reflection::get_method_parameter_attributes(
                    &class_name,
                    &method_name,
                    &parameter_name,
                    &self.classes,
                )
            }
            "get_function_attributes" => {
                if args.is_empty() {
                    return Err("get_function_attributes() expects 1 argument".to_string());
                }
                let function_name = args[0].to_string_val();
                reflection::get_function_attributes(&function_name, &self.functions)
            }
            "get_parameter_attributes" => {
                if args.len() < 2 {
                    return Err("get_parameter_attributes() expects 2 arguments".to_string());
                }
                let function_name = args[0].to_string_val();
                let parameter_name = args[1].to_string_val();
                reflection::get_parameter_attributes(
                    &function_name,
                    &parameter_name,
                    &self.functions,
                )
            }
            "get_interface_attributes" => {
                if args.is_empty() {
                    return Err("get_interface_attributes() expects 1 argument".to_string());
                }
                let interface_name = args[0].to_string_val();
                reflection::get_interface_attributes(&interface_name, &self.interfaces)
            }
            "get_trait_attributes" => {
                if args.is_empty() {
                    return Err("get_trait_attributes() expects 1 argument".to_string());
                }
                let trait_name = args[0].to_string_val();
                reflection::get_trait_attributes(&trait_name, &self.traits)
            }
            _ => builtins::call_builtin(func_name, args, &mut self.output),
        }
    }
}

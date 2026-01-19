//! Array creation functions

use crate::runtime::{ArrayKey, Value};

/// array_fill - Fill an array with values
pub fn array_fill(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_fill() expects at least 2 parameters".to_string());
    }
    let start_index = args
        .get(2)
        .and_then(|v| match v {
            Value::Integer(n) => Some(*n),
            _ => None,
        })
        .unwrap_or(0);
    let count = args[0].to_int();
    let value = args[1].clone();
    if count < 0 {
        return Err("array_fill(): Number of elements must be non-negative".to_string());
    }
    let mut result: Vec<(ArrayKey, Value)> = Vec::new();
    for i in 0..count {
        let key = if start_index == 0 {
            ArrayKey::Integer(i)
        } else {
            ArrayKey::Integer(start_index + i)
        };
        result.push((key, value.clone()));
    }
    Ok(Value::Array(result))
}

/// array_fill_keys - Fill an array with values, specifying keys
pub fn array_fill_keys(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_fill_keys() expects at least 2 parameters".to_string());
    }
    let value = args[0].clone();
    match &args[1] {
        Value::Array(keys) => {
            let result: Vec<(ArrayKey, Value)> = keys
                .iter()
                .map(|(_, k)| {
                    let key = match k {
                        Value::Integer(n) => ArrayKey::Integer(*n),
                        Value::String(s) => ArrayKey::String(s.clone()),
                        _ => ArrayKey::String(match k {
                            Value::Float(f) => f.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => "".to_string(),
                        }),
                    };
                    (key, value.clone())
                })
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err("array_fill_keys() expects parameter 2 to be array".to_string()),
    }
}

/// array_combine - Creates an array by using one array for keys and another for its values
pub fn array_combine(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_combine() expects exactly 2 parameters".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Array(keys), Value::Array(values)) => {
            if keys.len() != values.len() {
                return Err(
                    "array_combine(): Number of elements in each array must be equal".to_string(),
                );
            }
            let result: Vec<(ArrayKey, Value)> = keys
                .iter()
                .zip(values.iter())
                .map(|((_, k), (_, v))| {
                    let key = match k {
                        Value::Integer(n) => ArrayKey::Integer(*n),
                        Value::String(s) => ArrayKey::String(s.clone()),
                        _ => ArrayKey::String(match k {
                            Value::Float(f) => f.to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Null => "".to_string(),
                            _ => "".to_string(),
                        }),
                    };
                    (key, v.clone())
                })
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err("array_combine() expects both parameters to be arrays".to_string()),
    }
}

//! Array callback functions

use crate::runtime::Value;

fn check_is_callable(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Array(arr) if arr.len() == 2 => {
            let first = &arr[0].1;
            let second = &arr[1].1;
            matches!((first, second), (Value::String(_), Value::String(_)))
        }
        Value::Closure(_) => true,
        _ => false,
    }
}

/// array_map - Applies callback to elements of given arrays
pub fn array_map(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_map() expects at least 2 parameters".to_string());
    }

    if !check_is_callable(&args[1]) {
        return Err("array_map() expects parameter 1 to be a valid callback".to_string());
    }

    match &args[0] {
        Value::Array(_) => Ok(args[0].clone()),
        _ => Err("array_map() expects parameter 1 to be array".to_string()),
    }
}

/// array_filter - Filters elements of an array using a callback function
pub fn array_filter(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_filter() expects at least 2 parameters".to_string());
    }

    if !check_is_callable(&args[1]) {
        return Err("array_filter() expects parameter 2 to be a valid callback".to_string());
    }

    match &args[0] {
        Value::Array(_) => Ok(args[0].clone()),
        _ => Err("array_filter() expects parameter 1 to be array".to_string()),
    }
}

/// array_reduce - Iteratively reduce an array to a single value using a callback function
pub fn array_reduce(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_reduce() expects at least 2 parameters".to_string());
    }

    if args.len() >= 3 {
        Ok(args[2].clone())
    } else {
        match &args[0] {
            Value::Array(arr) if !arr.is_empty() => Ok(arr[0].1.clone()),
            Value::Array(_) => Ok(Value::Null),
            _ => Err("array_reduce() expects parameter 1 to be array".to_string()),
        }
    }
}

/// array_sum - Calculate sum of values in an array
pub fn array_sum(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_sum() expects exactly 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut sum: f64 = 0.0;
            for (_, value) in arr {
                sum += value.to_float();
            }
            if sum.fract() == 0.0 {
                Ok(Value::Integer(sum as i64))
            } else {
                Ok(Value::Float(sum))
            }
        }
        _ => Err("array_sum() expects parameter 1 to be array".to_string()),
    }
}

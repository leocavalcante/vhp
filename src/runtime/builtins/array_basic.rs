//! Basic array access functions

use crate::runtime::{ArrayKey, Value};

/// count - Count all elements in an array
pub fn count(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("count() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Null => Ok(Value::Integer(0)),
        _ => Ok(Value::Integer(1)),
    }
}

/// array_push - Push one or more elements onto the end of array
pub fn array_push(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_push() expects at least 2 parameters".to_string());
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            let max_key = new_arr
                .iter()
                .filter_map(|(k, _)| {
                    if let ArrayKey::Integer(i) = k {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(-1);

            let mut next_key = max_key + 1;
            for value in args.iter().skip(1) {
                new_arr.push((ArrayKey::Integer(next_key), value.clone()));
                next_key += 1;
            }
            Ok(Value::Integer(new_arr.len() as i64))
        }
        _ => Err("array_push() expects parameter 1 to be array".to_string()),
    }
}

/// array_pop - Pop element off the end of array
pub fn array_pop(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_pop() expects exactly 1 parameter".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(arr.last().map(|(_, v)| v.clone()).unwrap_or(Value::Null))
            }
        }
        _ => Err("array_pop() expects parameter 1 to be array".to_string()),
    }
}

/// array_shift - Shift an element off the beginning of array
pub fn array_shift(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_shift() expects exactly 1 parameter".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(arr.first().map(|(_, v)| v.clone()).unwrap_or(Value::Null))
            }
        }
        _ => Err("array_shift() expects parameter 1 to be array".to_string()),
    }
}

/// array_unshift - Prepend one or more elements to the beginning of an array
pub fn array_unshift(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_unshift() expects at least 2 parameters".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let new_count = arr.len() + args.len() - 1;
            Ok(Value::Integer(new_count as i64))
        }
        _ => Err("array_unshift() expects parameter 1 to be array".to_string()),
    }
}

/// array_keys - Return all the keys of an array
pub fn array_keys(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_keys() expects at least 1 parameter".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let keys: Vec<(ArrayKey, Value)> = arr
                .iter()
                .enumerate()
                .map(|(i, (k, _))| {
                    let key_val = match k {
                        ArrayKey::Integer(n) => Value::Integer(*n),
                        ArrayKey::String(s) => Value::String(s.clone()),
                    };
                    (ArrayKey::Integer(i as i64), key_val)
                })
                .collect();
            Ok(Value::Array(keys))
        }
        _ => Err("array_keys() expects parameter 1 to be array".to_string()),
    }
}

/// array_values - Return all the values of an array
pub fn array_values(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_values() expects at least 1 parameter".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let values: Vec<(ArrayKey, Value)> = arr
                .iter()
                .enumerate()
                .map(|(i, (_, v))| (ArrayKey::Integer(i as i64), v.clone()))
                .collect();
            Ok(Value::Array(values))
        }
        _ => Err("array_values() expects parameter 1 to be array".to_string()),
    }
}

/// array_slice - Returns a slice of an array
pub fn array_slice(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_slice() expects at least 2 parameters".to_string());
    }
    match &args[0] {
        Value::Array(arr) => {
            let offset = match &args[1] {
                Value::Integer(n) => *n,
                _ => return Err("array_slice() offset must be integer".to_string()),
            };
            let length = args.get(2).and_then(|v| match v {
                Value::Integer(n) => Some(*n),
                _ => None,
            });
            let start = if offset < 0 {
                (arr.len() as i64 + offset).max(0) as usize
            } else {
                offset as usize
            };
            if start >= arr.len() {
                return Ok(Value::Array(Vec::new()));
            }
            let arr_len = arr.len() as i64;
            let end = match length {
                Some(len) if len < 0 => (arr_len + len).max(0) as usize,
                Some(len) => (start as i64 + len).min(arr_len) as usize,
                None => arr_len as usize,
            }
            .min(arr.len());
            let result: Vec<(ArrayKey, Value)> = arr[start..end]
                .iter()
                .enumerate()
                .map(|(i, v)| (ArrayKey::Integer(i as i64), v.1.clone()))
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err("array_slice() expects parameter 1 to be array".to_string()),
    }
}

/// array_first - Get first value of an array (PHP 8.5)
pub fn array_first(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_first() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(arr.first().map(|(_, v)| v.clone()).unwrap_or(Value::Null))
            }
        }
        _ => Err("array_first() expects parameter 1 to be array".to_string()),
    }
}

/// array_last - Get last value of an array (PHP 8.5)
pub fn array_last(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_last() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(arr.last().map(|(_, v)| v.clone()).unwrap_or(Value::Null))
            }
        }
        _ => Err("array_last() expects parameter 1 to be array".to_string()),
    }
}

//! Array built-in functions

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

/// in_array - Checks if a value exists in an array
pub fn in_array(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("in_array() expects at least 2 parameters".to_string());
    }

    let needle = &args[0];
    let strict = args.get(2).map(|v| v.to_bool()).unwrap_or(false);

    match &args[1] {
        Value::Array(arr) => {
            let found = arr.iter().any(|(_, v)| {
                if strict {
                    needle.type_equals(v)
                } else {
                    needle.loose_equals(v)
                }
            });
            Ok(Value::Bool(found))
        }
        _ => Err("in_array() expects parameter 2 to be array".to_string()),
    }
}

/// array_search - Searches array for a given value and returns the key
pub fn array_search(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_search() expects at least 2 parameters".to_string());
    }

    let needle = &args[0];
    let strict = args.get(2).map(|v| v.to_bool()).unwrap_or(false);

    match &args[1] {
        Value::Array(arr) => {
            for (k, v) in arr {
                let matches = if strict {
                    needle.type_equals(v)
                } else {
                    needle.loose_equals(v)
                };
                if matches {
                    return Ok(match k {
                        ArrayKey::Integer(n) => Value::Integer(*n),
                        ArrayKey::String(s) => Value::String(s.clone()),
                    });
                }
            }
            Ok(Value::Bool(false))
        }
        _ => Err("array_search() expects parameter 2 to be array".to_string()),
    }
}

/// array_reverse - Return an array with elements in reverse order
pub fn array_reverse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_reverse() expects at least 1 parameter".to_string());
    }

    let preserve_keys = args.get(1).map(|v| v.to_bool()).unwrap_or(false);

    match &args[0] {
        Value::Array(arr) => {
            if preserve_keys {
                Ok(Value::Array(arr.iter().rev().cloned().collect()))
            } else {
                let reversed: Vec<(ArrayKey, Value)> = arr
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(i, (k, v))| {
                        let new_key = match k {
                            ArrayKey::String(_) => k.clone(),
                            ArrayKey::Integer(_) => ArrayKey::Integer(i as i64),
                        };
                        (new_key, v.clone())
                    })
                    .collect();
                Ok(Value::Array(reversed))
            }
        }
        _ => Err("array_reverse() expects parameter 1 to be array".to_string()),
    }
}

/// array_merge - Merge one or more arrays
pub fn array_merge(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_merge() expects at least 1 parameter".to_string());
    }

    let mut result: Vec<(ArrayKey, Value)> = Vec::new();
    let mut next_int_key: i64 = 0;

    for arg in args {
        match arg {
            Value::Array(arr) => {
                for (k, v) in arr {
                    match k {
                        ArrayKey::Integer(_) => {
                            result.push((ArrayKey::Integer(next_int_key), v.clone()));
                            next_int_key += 1;
                        }
                        ArrayKey::String(s) => {
                            // String keys overwrite existing
                            if let Some(pos) = result
                                .iter()
                                .position(|(rk, _)| matches!(rk, ArrayKey::String(rs) if rs == s))
                            {
                                result[pos] = (k.clone(), v.clone());
                            } else {
                                result.push((k.clone(), v.clone()));
                            }
                        }
                    }
                }
            }
            _ => return Err("array_merge() expects all parameters to be arrays".to_string()),
        }
    }

    Ok(Value::Array(result))
}

/// array_key_exists - Checks if the given key or index exists in an array
pub fn array_key_exists(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_key_exists() expects exactly 2 parameters".to_string());
    }

    let key = match &args[0] {
        Value::Integer(n) => ArrayKey::Integer(*n),
        Value::String(s) => ArrayKey::String(s.clone()),
        _ => return Ok(Value::Bool(false)),
    };

    match &args[1] {
        Value::Array(arr) => {
            let exists = arr.iter().any(|(k, _)| k == &key);
            Ok(Value::Bool(exists))
        }
        _ => Err("array_key_exists() expects parameter 2 to be array".to_string()),
    }
}

/// range - Create an array containing a range of elements
pub fn range(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("range() expects at least 2 parameters".to_string());
    }

    let start = args[0].to_int();
    let end = args[1].to_int();
    let step = args.get(2).map(|v| v.to_int()).unwrap_or(1);

    if step == 0 {
        return Err("range(): step exceeds the specified range".to_string());
    }

    let mut result: Vec<(ArrayKey, Value)> = Vec::new();
    let mut i = 0i64;

    if start <= end {
        let mut current = start;
        while current <= end {
            result.push((ArrayKey::Integer(i), Value::Integer(current)));
            current += step.abs();
            i += 1;
        }
    } else {
        let mut current = start;
        while current >= end {
            result.push((ArrayKey::Integer(i), Value::Integer(current)));
            current -= step.abs();
            i += 1;
        }
    }

    Ok(Value::Array(result))
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

/// array_map - Applies callback to elements of given arrays
pub fn array_map(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_map() expects at least 2 parameters".to_string());
    }

    // For now, return the array unchanged (placeholder - callbacks need full generator support)
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

    // For now, return the array unchanged (placeholder - full generator support needed)
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

    // For now, return the initial value or first element (placeholder - full generator support needed)
    if args.len() >= 3 {
        Ok(args[2].clone())
    } else {
        // Return the first element of the array
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

/// array_unique - Removes duplicate values from an array
pub fn array_unique(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_unique() expects exactly 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut result: Vec<(ArrayKey, Value)> = Vec::new();
            let mut seen: Vec<String> = Vec::new();

            for (_, value) in arr {
                let value_str = match value {
                    Value::Integer(n) => n.to_string(),
                    Value::Float(f) => {
                        if f.fract() == 0.0 && f.abs() < 1e15 {
                            format!("{:.0}", f)
                        } else {
                            f.to_string()
                        }
                    }
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => continue,
                };

                if !seen.contains(&value_str) {
                    seen.push(value_str);
                    result.push((ArrayKey::Integer(result.len() as i64), value.clone()));
                }
            }

            Ok(Value::Array(result))
        }
        _ => Err("array_unique() expects parameter 1 to be array".to_string()),
    }
}

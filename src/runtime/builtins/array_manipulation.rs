//! Array manipulation functions

use crate::runtime::{ArrayKey, Value};

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

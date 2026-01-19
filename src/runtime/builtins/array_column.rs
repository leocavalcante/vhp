//! Array column and value operations

use crate::runtime::{ArrayKey, Value};

/// array_column - Return the values from a single column in the input array
pub fn array_column(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_column() expects at least 1 parameter".to_string());
    }
    let column_key = args.get(1).map(|v| v.to_string_val()).unwrap_or_default();
    let index_key = args.get(2).map(|v| v.to_string_val());
    match &args[0] {
        Value::Array(arr) => {
            let mut result: Vec<(ArrayKey, Value)> = Vec::new();
            for (i, row) in arr.iter().enumerate() {
                if let Value::Array(row_arr) = &row.1 {
                    let mut col_value: Option<Value> = None;
                    let mut key_value: Option<Value> = None;
                    for (j, (k, v)) in row_arr.iter().enumerate() {
                        let key_str = match k {
                            ArrayKey::String(s) => s.clone(),
                            ArrayKey::Integer(n) => n.to_string(),
                        };
                        if key_str == column_key {
                            col_value = Some(v.clone());
                        }
                        if let Some(ref idx) = index_key {
                            if key_str == *idx {
                                key_value = Some(v.clone());
                            }
                        }
                        #[allow(clippy::collapsible_if)]
                        if j == 0 && index_key.is_none() {}
                    }
                    if let Some(val) = col_value {
                        let final_key = if let Some(kv) = key_value {
                            kv
                        } else {
                            Value::Integer(i as i64)
                        };
                        let final_array_key = if let Value::Integer(n) = &final_key {
                            ArrayKey::Integer(*n)
                        } else if let Value::String(s) = &final_key {
                            ArrayKey::String(s.clone())
                        } else {
                            ArrayKey::Integer(i as i64)
                        };
                        result.push((final_array_key, val));
                    }
                }
            }
            Ok(Value::Array(result))
        }
        _ => Err("array_column() expects parameter 1 to be array".to_string()),
    }
}

/// array_flip - Exchanges all keys with their associated values
pub fn array_flip(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_flip() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut result: Vec<(ArrayKey, Value)> = Vec::new();
            for (_, v) in arr.iter() {
                let new_key = match v {
                    Value::Integer(n) => ArrayKey::Integer(*n),
                    Value::String(s) => ArrayKey::String(s.clone()),
                    _ => continue,
                };
                result.push((new_key, v.clone()));
            }
            Ok(Value::Array(result))
        }
        _ => Err("array_flip() expects parameter 1 to be array".to_string()),
    }
}

/// array_count_values - Counts all the values of an array
pub fn array_count_values(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_count_values() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut counts: Vec<(ArrayKey, Value)> = Vec::new();
            let mut seen: Vec<String> = Vec::new();
            for (_, v) in arr.iter() {
                let v_str = match v {
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
                if let Some(pos) = seen.iter().position(|s| s == &v_str) {
                    if let Some((_, Value::Integer(n))) = counts.get_mut(pos) {
                        *n += 1;
                    }
                } else {
                    seen.push(v_str.clone());
                    let key = match v {
                        Value::Integer(n) => ArrayKey::Integer(*n),
                        Value::String(s) => ArrayKey::String(s.clone()),
                        _ => ArrayKey::String(v_str),
                    };
                    counts.push((key, Value::Integer(1)));
                }
            }
            Ok(Value::Array(counts))
        }
        _ => Err("array_count_values() expects parameter 1 to be array".to_string()),
    }
}

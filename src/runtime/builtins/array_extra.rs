//! Additional Array built-in functions

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

/// array_chunk - Split an array into chunks
pub fn array_chunk(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_chunk() expects at least 2 parameters".to_string());
    }
    let preserve_keys = args.get(2).map(|v| v.to_bool()).unwrap_or(false);
    match &args[0] {
        Value::Array(arr) => {
            let size = match &args[1] {
                Value::Integer(n) if *n > 0 => *n as usize,
                _ => return Err("array_chunk() size must be positive integer".to_string()),
            };
            let mut chunks: Vec<(ArrayKey, Value)> = Vec::new();
            let mut current_chunk: Vec<(ArrayKey, Value)> = Vec::new();
            let mut chunk_index = 0i64;
            for (i, (k, v)) in arr.iter().enumerate() {
                if preserve_keys {
                    current_chunk.push((k.clone(), v.clone()));
                } else {
                    current_chunk.push((ArrayKey::Integer(i as i64), v.clone()));
                }
                if current_chunk.len() == size {
                    chunks.push((ArrayKey::Integer(chunk_index), Value::Array(current_chunk)));
                    current_chunk = Vec::new();
                    chunk_index += 1;
                }
            }
            if !current_chunk.is_empty() {
                chunks.push((ArrayKey::Integer(chunk_index), Value::Array(current_chunk)));
            }
            Ok(Value::Array(chunks))
        }
        _ => Err("array_chunk() expects parameter 1 to be array".to_string()),
    }
}

/// array_pad - Pad array to the specified length with a value
pub fn array_pad(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_pad() expects at least 2 parameters".to_string());
    }
    let pad_count = args[1].to_int();
    let pad_value = args.get(2).cloned().unwrap_or(Value::Null);
    match &args[0] {
        Value::Array(arr) => {
            let arr_len = arr.len() as i64;
            if pad_count >= arr_len {
                let mut result = arr.clone();
                let to_add = (pad_count - arr_len) as usize;
                for _ in 0..to_add {
                    result.push((ArrayKey::Integer(result.len() as i64), pad_value.clone()));
                }
                Ok(Value::Array(result))
            } else if pad_count < -arr_len {
                let mut result: Vec<(ArrayKey, Value)> = Vec::new();
                let to_prepend = (-pad_count - arr_len) as usize;
                for i in 0..to_prepend {
                    result.push((ArrayKey::Integer(i as i64), pad_value.clone()));
                }
                for (i, (_, v)) in arr.iter().enumerate() {
                    let new_key = ArrayKey::Integer((i as i64) + (to_prepend as i64));
                    result.push((new_key, v.clone()));
                }
                Ok(Value::Array(result))
            } else {
                Ok(args[0].clone())
            }
        }
        _ => Err("array_pad() expects parameter 1 to be array".to_string()),
    }
}

/// array_splice - Remove a portion of the array and replace it with something else
pub fn array_splice(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_splice() expects at least 2 parameters".to_string());
    }
    match &args[0] {
        Value::Array(arr) => {
            let offset = match &args[1] {
                Value::Integer(n) => *n,
                _ => return Err("array_splice() offset must be integer".to_string()),
            };
            let length = args.get(2).and_then(|v| match v {
                Value::Integer(n) => Some(*n),
                _ => None,
            });
            let replacement = args.get(3).cloned();
            let arr_len = arr.len() as i64;
            let start = if offset < 0 {
                (arr_len + offset).max(0) as usize
            } else {
                offset as usize
            };
            let end = match length {
                Some(len) if len < 0 => (arr_len + len).max(0) as usize,
                Some(len) => (start as i64 + len).min(arr_len) as usize,
                None => arr_len as usize,
            }
            .min(arr.len());
            if let Some(repl) = replacement {
                let mut final_result: Vec<(ArrayKey, Value)> = Vec::new();
                for (i, (_, v)) in arr.iter().enumerate() {
                    if i == start {
                        match &repl {
                            Value::Array(repl_arr) => {
                                for (_, rv) in repl_arr {
                                    final_result.push((
                                        ArrayKey::Integer(final_result.len() as i64),
                                        rv.clone(),
                                    ));
                                }
                            }
                            _ => {
                                final_result.push((
                                    ArrayKey::Integer(final_result.len() as i64),
                                    repl.clone(),
                                ));
                            }
                        }
                    }
                    if i < start || i >= end {
                        final_result
                            .push((ArrayKey::Integer(final_result.len() as i64), v.clone()));
                    }
                }
                Ok(Value::Array(final_result))
            } else {
                let _removed: Vec<(ArrayKey, Value)> = arr[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v.1.clone()))
                    .collect();
                let mut result: Vec<(ArrayKey, Value)> = Vec::new();
                for (i, (_, v)) in arr.iter().enumerate() {
                    if i < start || i >= end {
                        result.push((ArrayKey::Integer(result.len() as i64), v.clone()));
                    }
                }
                Ok(Value::Array(result))
            }
        }
        _ => Err("array_splice() expects parameter 1 to be array".to_string()),
    }
}

/// array_diff - Computes the difference of arrays
pub fn array_diff(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_diff() expects at least 2 parameters".to_string());
    }
    let mut result: Vec<(ArrayKey, Value)> = Vec::new();
    match &args[0] {
        Value::Array(arr1) => {
            let arrays: Vec<&Vec<(ArrayKey, Value)>> = args
                .iter()
                .skip(1)
                .filter_map(|a| match a {
                    Value::Array(a) => Some(a),
                    _ => None,
                })
                .collect();
            'outer: for (_, v) in arr1.iter() {
                for arr in &arrays {
                    if arr.iter().any(|(_, av)| v.loose_equals(av)) {
                        continue 'outer;
                    }
                }
                result.push((ArrayKey::Integer(result.len() as i64), v.clone()));
            }
            Ok(Value::Array(result))
        }
        _ => Err("array_diff() expects parameter 1 to be array".to_string()),
    }
}

/// array_intersect - Computes the intersection of arrays
pub fn array_intersect(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("array_intersect() expects at least 2 parameters".to_string());
    }
    let mut result: Vec<(ArrayKey, Value)> = Vec::new();
    match &args[0] {
        Value::Array(arr1) => {
            let arrays: Vec<&Vec<(ArrayKey, Value)>> = args
                .iter()
                .skip(1)
                .filter_map(|a| match a {
                    Value::Array(a) => Some(a),
                    _ => None,
                })
                .collect();
            for (_, v) in arr1.iter() {
                let mut in_all = true;
                for arr in &arrays {
                    if !arr.iter().any(|(_, av)| v.loose_equals(av)) {
                        in_all = false;
                        break;
                    }
                }
                if in_all {
                    result.push((ArrayKey::Integer(result.len() as i64), v.clone()));
                }
            }
            Ok(Value::Array(result))
        }
        _ => Err("array_intersect() expects parameter 1 to be array".to_string()),
    }
}

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
                        if j == 0 && index_key.is_none() {
                            // First element used as key when no index key specified
                        }
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

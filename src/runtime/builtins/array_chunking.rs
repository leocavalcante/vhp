//! Array chunking and padding functions

use crate::runtime::{ArrayKey, Value};

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

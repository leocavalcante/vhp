//! Array search and lookup functions

use crate::runtime::{ArrayKey, Value};

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

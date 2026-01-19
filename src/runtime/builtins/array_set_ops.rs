//! Array set operations (diff and intersect)

use crate::runtime::{ArrayKey, Value};

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

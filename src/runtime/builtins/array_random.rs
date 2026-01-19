//! Array randomization functions

use crate::runtime::{ArrayKey, Value};

/// shuffle - Shuffle an array randomly
///
/// Returns true on success, false on failure.
pub fn shuffle(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("shuffle() expects exactly 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut values: Vec<(ArrayKey, Value)> =
                arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

            let n = values.len();
            for i in 0..n {
                let j = fastrand::usize(0..n);
                values.swap(i, j);
            }

            Ok(Value::Array(values))
        }
        _ => Err("shuffle() expects parameter 1 to be array".to_string()),
    }
}

/// array_rand - Pick one or more random keys from an array
///
/// If called without the second parameter, returns a random key.
/// If called with num_req, returns an array of random keys.
pub fn array_rand(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("array_rand() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err("array_rand(): Array is empty".to_string());
            }

            let num_keys = args.get(1).map(|v| v.to_int()).unwrap_or(1);
            let num_keys = if num_keys < 1 { 1 } else { num_keys as usize };
            let num_keys = num_keys.min(arr.len());

            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);

            let mut rng = fastrand::Rng::new();
            rng.seed(seed);

            let keys: Vec<ArrayKey> = arr.iter().map(|(k, _)| k.clone()).collect();

            if num_keys == 1 {
                let random_idx = rng.usize(0..keys.len());
                Ok(match &keys[random_idx] {
                    ArrayKey::Integer(n) => Value::Integer(*n),
                    ArrayKey::String(s) => Value::String(s.clone()),
                })
            } else {
                let mut indices: Vec<usize> = (0..keys.len()).collect();
                for i in 0..indices.len() {
                    let j = rng.usize(0..indices.len());
                    indices.swap(i, j);
                }

                let result: Vec<(ArrayKey, Value)> = indices[..num_keys]
                    .iter()
                    .enumerate()
                    .map(|(i, &idx)| {
                        (
                            ArrayKey::Integer(i as i64),
                            match &keys[idx] {
                                ArrayKey::Integer(n) => Value::Integer(*n),
                                ArrayKey::String(s) => Value::String(s.clone()),
                            },
                        )
                    })
                    .collect();
                Ok(Value::Array(result))
            }
        }
        _ => Err("array_rand() expects parameter 1 to be array".to_string()),
    }
}

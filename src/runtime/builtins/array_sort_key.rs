//! Array key-based sorting functions

use crate::runtime::{ArrayKey, Value};

/// ksort - Sort an array by keys in ascending order
///
/// Returns true on success, false on failure.
pub fn ksort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ksort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut pairs: Vec<(ArrayKey, Value)> =
                arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::manual_range_patterns)]
            match flags {
                0..=3 => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| match (a_key, b_key) {
                        (ArrayKey::Integer(n1), ArrayKey::Integer(n2)) => n1.cmp(n2),
                        (ArrayKey::String(s1), ArrayKey::String(s2)) => s1.cmp(s2),
                        (ArrayKey::Integer(_), ArrayKey::String(_)) => std::cmp::Ordering::Less,
                        (ArrayKey::String(_), ArrayKey::Integer(_)) => std::cmp::Ordering::Greater,
                    });
                }
                4 => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| {
                        let a_str = match a_key {
                            ArrayKey::String(s) => s.clone(),
                            ArrayKey::Integer(n) => n.to_string(),
                        };
                        let b_str = match b_key {
                            ArrayKey::String(s) => s.clone(),
                            ArrayKey::Integer(n) => n.to_string(),
                        };
                        a_str
                            .cmp(&b_str)
                            .then_with(|| a_str.len().cmp(&b_str.len()))
                    });
                }
                _ => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| match (a_key, b_key) {
                        (ArrayKey::Integer(n1), ArrayKey::Integer(n2)) => n1.cmp(n2),
                        (ArrayKey::String(s1), ArrayKey::String(s2)) => s1.cmp(s2),
                        (ArrayKey::Integer(_), ArrayKey::String(_)) => std::cmp::Ordering::Less,
                        (ArrayKey::String(_), ArrayKey::Integer(_)) => std::cmp::Ordering::Greater,
                    });
                }
            }

            Ok(Value::Array(pairs))
        }
        _ => Err("ksort() expects parameter 1 to be array".to_string()),
    }
}

/// krsort - Sort an array by keys in descending order
///
/// Returns true on success, false on failure.
pub fn krsort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("krsort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut pairs: Vec<(ArrayKey, Value)> =
                arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::manual_range_patterns)]
            match flags {
                0..=3 => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| match (a_key, b_key) {
                        (ArrayKey::Integer(n1), ArrayKey::Integer(n2)) => n2.cmp(n1),
                        (ArrayKey::String(s1), ArrayKey::String(s2)) => s2.cmp(s1),
                        (ArrayKey::Integer(_), ArrayKey::String(_)) => std::cmp::Ordering::Greater,
                        (ArrayKey::String(_), ArrayKey::Integer(_)) => std::cmp::Ordering::Less,
                    });
                }
                4 => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| {
                        let a_str = match a_key {
                            ArrayKey::String(s) => s.clone(),
                            ArrayKey::Integer(n) => n.to_string(),
                        };
                        let b_str = match b_key {
                            ArrayKey::String(s) => s.clone(),
                            ArrayKey::Integer(n) => n.to_string(),
                        };
                        b_str
                            .cmp(&a_str)
                            .then_with(|| b_str.len().cmp(&a_str.len()))
                    });
                }
                _ => {
                    pairs.sort_by(|(a_key, _), (b_key, _)| match (a_key, b_key) {
                        (ArrayKey::Integer(n1), ArrayKey::Integer(n2)) => n2.cmp(n1),
                        (ArrayKey::String(s1), ArrayKey::String(s2)) => s2.cmp(s1),
                        (ArrayKey::Integer(_), ArrayKey::String(_)) => std::cmp::Ordering::Greater,
                        (ArrayKey::String(_), ArrayKey::Integer(_)) => std::cmp::Ordering::Less,
                    });
                }
            }

            Ok(Value::Array(pairs))
        }
        _ => Err("krsort() expects parameter 1 to be array".to_string()),
    }
}

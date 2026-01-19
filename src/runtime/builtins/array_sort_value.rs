//! Array value-based sorting functions

use crate::runtime::{ArrayKey, Value};

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Integer(n1), Value::Integer(n2)) => n1.cmp(n2),
        (Value::Integer(n1), Value::Float(f2)) => {
            let f1 = *n1 as f64;
            f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::Float(f1), Value::Integer(n2)) => {
            let f2 = *n2 as f64;
            f1.partial_cmp(&f2).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::Float(f1), Value::Float(f2)) => {
            f1.partial_cmp(f2).unwrap_or(std::cmp::Ordering::Equal)
        }
        (Value::String(s1), Value::String(s2)) => s1.cmp(s2),
        (Value::Bool(b1), Value::Bool(b2)) => b1.cmp(b2),
        (Value::Null, Value::Null) => std::cmp::Ordering::Equal,
        (Value::Null, _) => std::cmp::Ordering::Less,
        (_, Value::Null) => std::cmp::Ordering::Greater,
        _ => {
            let a_str = a.to_string_val();
            let b_str = b.to_string_val();
            a_str.cmp(&b_str)
        }
    }
}

/// sort - Sort an array in ascending order
///
/// Returns true on success, false on failure.
///
/// PHP equivalent: sort($array, $flags = SORT_REGULAR)
pub fn sort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut values: Vec<Value> = arr.iter().map(|(_, v)| v.clone()).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::unnecessary_sort_by, clippy::redundant_closure)]
            match flags {
                0..=1 => {
                    values.sort_by(|a, b| compare_values(a, b));
                }
                2 => {
                    values.sort_by(|a, b| a.to_string_val().cmp(&b.to_string_val()));
                }
                3 => {
                    values.sort_by(|a, b| a.to_string_val().cmp(&b.to_string_val()));
                }
                4 => {
                    values.sort_by(|a, b| {
                        let a_str = a.to_string_val();
                        let b_str = b.to_string_val();
                        a_str
                            .cmp(&b_str)
                            .then_with(|| a_str.len().cmp(&b_str.len()))
                    });
                }
                _ => {
                    values.sort_by(|a, b| compare_values(a, b));
                }
            }

            let result: Vec<(ArrayKey, Value)> = values
                .into_iter()
                .enumerate()
                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err("sort() expects parameter 1 to be array".to_string()),
    }
}

/// rsort - Sort an array in descending order
///
/// Returns true on success, false on failure.
pub fn rsort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("rsort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut values: Vec<Value> = arr.iter().map(|(_, v)| v.clone()).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::unnecessary_sort_by)]
            match flags {
                0..=1 => {
                    values.sort_by(|a, b| compare_values(a, b).reverse());
                }
                2..=3 => {
                    values.sort_by(|a, b| b.to_string_val().cmp(&a.to_string_val()));
                }
                4 => {
                    values.sort_by(|a, b| {
                        let b_str = b.to_string_val();
                        let a_str = a.to_string_val();
                        b_str
                            .cmp(&a_str)
                            .then_with(|| b_str.len().cmp(&a_str.len()))
                    });
                }
                _ => {
                    values.sort_by(|a, b| compare_values(a, b).reverse());
                }
            }

            let result: Vec<(ArrayKey, Value)> = values
                .into_iter()
                .enumerate()
                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                .collect();
            Ok(Value::Array(result))
        }
        _ => Err("rsort() expects parameter 1 to be array".to_string()),
    }
}

/// asort - Sort an array in ascending order, preserving keys
///
/// Returns true on success, false on failure.
pub fn asort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("asort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut pairs: Vec<(ArrayKey, Value)> =
                arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::unnecessary_sort_by)]
            match flags {
                0..=1 => {
                    pairs.sort_by(|(_, a), (_, b)| compare_values(a, b));
                }
                2..=3 => {
                    pairs.sort_by(|(_, a), (_, b)| a.to_string_val().cmp(&b.to_string_val()));
                }
                4 => {
                    pairs.sort_by(|(_, a), (_, b)| {
                        let a_str = a.to_string_val();
                        let b_str = b.to_string_val();
                        a_str
                            .cmp(&b_str)
                            .then_with(|| a_str.len().cmp(&b_str.len()))
                    });
                }
                _ => {
                    pairs.sort_by(|(_, a), (_, b)| compare_values(a, b));
                }
            }

            Ok(Value::Array(pairs))
        }
        _ => Err("asort() expects parameter 1 to be array".to_string()),
    }
}

/// arsort - Sort an array in descending order, preserving keys
///
/// Returns true on success, false on failure.
pub fn arsort(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("arsort() expects at least 1 parameter, 0 given".to_string());
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut pairs: Vec<(ArrayKey, Value)> =
                arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let flags = args.get(1).map(|v| v.to_int()).unwrap_or(0);

            #[allow(clippy::unnecessary_sort_by)]
            match flags {
                0..=1 => {
                    pairs.sort_by(|(_, a), (_, b)| compare_values(a, b).reverse());
                }
                2..=3 => {
                    pairs.sort_by(|(_, a), (_, b)| b.to_string_val().cmp(&a.to_string_val()));
                }
                4 => {
                    pairs.sort_by(|(_, a), (_, b)| {
                        let a_str = a.to_string_val();
                        let b_str = b.to_string_val();
                        b_str
                            .cmp(&a_str)
                            .then_with(|| b_str.len().cmp(&a_str.len()))
                    });
                }
                _ => {
                    pairs.sort_by(|(_, a), (_, b)| compare_values(a, b).reverse());
                }
            }

            Ok(Value::Array(pairs))
        }
        _ => Err("arsort() expects parameter 1 to be array".to_string()),
    }
}

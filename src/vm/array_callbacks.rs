//! VM-aware array callback functions
//!
//! This module provides array_map, array_filter, and array_reduce
//! which require VM access to execute callbacks.

use crate::runtime::{ArrayKey, Value};
use crate::vm::callback_helpers::{call_callback, is_callable};
use crate::vm::VM;
use std::io::Write;

impl<W: Write> VM<W> {
    /// array_map - Apply callback to each element of array
    ///
    /// Applies the callback to each element of the given array and returns
    /// a new array with the results.
    ///
    /// PHP equivalent:
    ///   $result = array_map(fn($x) => $x * 2, $array);
    ///
    /// Returns array with keys reindexed from 0
    pub fn array_map(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("array_map() expects at least 2 parameters".to_string());
        }

        // Determine which argument is the callback and which is the array
        let (callback, array) = match (&args[0], &args[1]) {
            (Value::Closure(_) | Value::String(_), Value::Array(arr)) => (&args[0], arr),
            (Value::Array(arr), Value::Closure(_) | Value::String(_)) => (&args[1], arr),
            (Value::Array(_), Value::Array(_)) => {
                return Err(
                    "array_map() expects parameter 1 or 2 to be a valid callback".to_string(),
                );
            }
            _ => {
                return Err("array_map() expects parameter 1 to be a valid callback".to_string());
            }
        };

        if !is_callable(callback) {
            return Err("array_map() expects parameter 1 to be a valid callback".to_string());
        }

        let mut result = Vec::new();
        for (i, (_, value)) in array.iter().enumerate() {
            let mapped_value = call_callback(self, callback, std::slice::from_ref(value))?;
            result.push((ArrayKey::Integer(i as i64), mapped_value));
        }

        Ok(Value::Array(result))
    }

    /// array_filter - Filter array elements using callback
    ///
    /// Iterates over each value in the array, passing them to the callback.
    /// If the callback returns true, the value is included in the result array.
    ///
    /// PHP equivalent:
    ///   $even = array_filter($array, fn($x) => $x % 2 == 0);
    ///
    /// Note: Arguments are passed with array first, callback second
    /// (due to stack-based argument passing)
    ///
    /// Preserves original keys
    pub fn array_filter(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("array_filter() expects at least 2 parameters".to_string());
        }

        // args[0] is the array, args[1] is the callback (reversed from PHP order)
        let array = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err("array_filter() expects parameter 1 to be array".to_string()),
        };

        let callback = &args[1];

        if !is_callable(callback) {
            return Err("array_filter() expects parameter 2 to be a valid callback".to_string());
        }

        let mut result = Vec::new();
        for (key, value) in array.iter() {
            let keep = call_callback(self, callback, std::slice::from_ref(value))?;
            let should_keep = match &keep {
                Value::Bool(b) => *b,
                Value::Integer(n) => *n != 0,
                Value::Float(f) => *f != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::Array(arr) => !arr.is_empty(),
                Value::Object(_) => true,
                Value::EnumCase { .. } => true,
                Value::Exception(_) => true,
                Value::Fiber(_) => true,
                Value::Closure(_) => true,
                Value::Generator(_) => true,
                Value::Null => false,
            };
            if should_keep {
                result.push((key.clone(), value.clone()));
            }
        }

        Ok(Value::Array(result))
    }

    /// array_reduce - Reduce array to single value using callback
    ///
    /// Iteratively reduces the array to a single value using the callback.
    /// The callback receives the accumulated value and current element.
    ///
    /// PHP equivalent:
    ///   $sum = array_reduce($array, fn($carry, $item) => $carry + $item, 0);
    ///
    /// Note: Arguments are passed with array first, callback second, then initial
    /// (due to stack-based argument passing)
    ///
    /// If initial value is not provided and array is empty, returns NULL.
    /// If initial value is not provided and array has elements, uses first element as initial.
    pub fn array_reduce(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("array_reduce() expects at least 2 parameters".to_string());
        }

        // args[0] is the array, args[1] is the callback (reversed from PHP order)
        let array = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err("array_reduce() expects parameter 1 to be array".to_string()),
        };

        let callback = &args[1];

        if !is_callable(callback) {
            return Err("array_reduce() expects parameter 2 to be a valid callback".to_string());
        }

        if array.is_empty() {
            return Ok(args.get(2).cloned().unwrap_or(Value::Null));
        }

        let mut accumulator = match args.get(2) {
            Some(initial) => initial.clone(),
            None => array[0].1.clone(),
        };

        let start_idx = if args.len() < 3 { 1 } else { 0 };

        for (_, value) in array.iter().skip(start_idx) {
            accumulator = call_callback(self, callback, &[accumulator, value.clone()])?;
        }

        Ok(accumulator)
    }

    /// usort - Sort an array by values using a user-defined comparison function
    ///
    /// Sorts array by its values using a comparison function.
    /// If the comparison function returns 0, the order is undefined.
    /// Returns true on success, false on failure.
    ///
    /// PHP equivalent:
    ///   usort($array, fn($a, $b) => $a <=> $b);
    pub fn usort(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("usort() expects at least 2 parameters".to_string());
        }

        let array = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err("usort() expects parameter 1 to be array".to_string()),
        };

        let callback = &args[1];

        if !is_callable(callback) {
            return Err("usort() expects parameter 2 to be a valid callback".to_string());
        }

        let mut values: Vec<Value> = array.iter().map(|(_, v)| v.clone()).collect();

        values.sort_by(|a, b| {
            let cmp_result = call_callback(self, callback, &[a.clone(), b.clone()]);
            match cmp_result {
                Ok(Value::Integer(n)) => {
                    if n < 0 {
                        std::cmp::Ordering::Less
                    } else if n > 0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                Ok(Value::Float(f)) => {
                    if f < 0.0 {
                        std::cmp::Ordering::Less
                    } else if f > 0.0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        let result_array: Vec<(ArrayKey, Value)> = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
            .collect();
        Ok(Value::Array(result_array))
    }

    /// uasort - Sort an array by values using a user-defined comparison function, preserving keys
    ///
    /// Sorts array by its values using a comparison function while preserving keys.
    /// Returns true on success, false on failure.
    pub fn uasort(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("uasort() expects at least 2 parameters".to_string());
        }

        let array = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err("uasort() expects parameter 1 to be array".to_string()),
        };

        let callback = &args[1];

        if !is_callable(callback) {
            return Err("uasort() expects parameter 2 to be a valid callback".to_string());
        }

        let mut pairs: Vec<(ArrayKey, Value)> =
            array.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        pairs.sort_by(|(_, a), (_, b)| {
            let cmp_result = call_callback(self, callback, &[a.clone(), b.clone()]);
            match cmp_result {
                Ok(Value::Integer(n)) => {
                    if n < 0 {
                        std::cmp::Ordering::Less
                    } else if n > 0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                Ok(Value::Float(f)) => {
                    if f < 0.0 {
                        std::cmp::Ordering::Less
                    } else if f > 0.0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(Value::Array(pairs))
    }

    /// uksort - Sort an array by keys using a user-defined comparison function
    ///
    /// Sorts array by its keys using a comparison function.
    /// Returns true on success, false on failure.
    pub fn uksort(&mut self, args: &[Value]) -> Result<Value, String> {
        if args.len() < 2 {
            return Err("uksort() expects at least 2 parameters".to_string());
        }

        let array = match &args[0] {
            Value::Array(arr) => arr,
            _ => return Err("uksort() expects parameter 1 to be array".to_string()),
        };

        let callback = &args[1];

        if !is_callable(callback) {
            return Err("uksort() expects parameter 2 to be a valid callback".to_string());
        }

        let mut pairs: Vec<(ArrayKey, Value)> =
            array.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        pairs.sort_by(|(a_key, _), (b_key, _)| {
            let a_key_val = match a_key {
                ArrayKey::Integer(n) => Value::Integer(*n),
                ArrayKey::String(s) => Value::String(s.clone()),
            };
            let b_key_val = match b_key {
                ArrayKey::Integer(n) => Value::Integer(*n),
                ArrayKey::String(s) => Value::String(s.clone()),
            };

            let cmp_result = call_callback(self, callback, &[a_key_val, b_key_val]);
            match cmp_result {
                Ok(Value::Integer(n)) => {
                    if n < 0 {
                        std::cmp::Ordering::Less
                    } else if n > 0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                Ok(Value::Float(f)) => {
                    if f < 0.0 {
                        std::cmp::Ordering::Less
                    } else if f > 0.0 {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                }
                _ => std::cmp::Ordering::Equal,
            }
        });

        Ok(Value::Array(pairs))
    }
}

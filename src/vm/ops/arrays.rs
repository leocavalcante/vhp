use crate::runtime::{ArrayKey, Value};

pub fn execute_new_array<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    count: u16,
) -> Result<(), String> {
    let mut arr = Vec::new();
    for _ in 0..count {
        let value = vm.stack.pop().ok_or("Stack underflow")?;
        let key = vm.stack.pop().ok_or("Stack underflow")?;
        arr.push((ArrayKey::from_value(&key), value));
    }
    arr.reverse();
    vm.stack.push(Value::Array(arr));
    Ok(())
}

pub fn execute_array_get<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let key = vm.stack.pop().ok_or("Stack underflow")?;
    let array = vm.stack.pop().ok_or("Stack underflow")?;
    match array {
        Value::Array(arr) => {
            let array_key = ArrayKey::from_value(&key);
            let value = arr
                .iter()
                .find(|(k, _)| k == &array_key)
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Null);
            vm.stack.push(value);
        }
        _ => return Err("Cannot use [] on non-array".to_string()),
    }
    Ok(())
}

pub fn execute_array_set<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let key = vm.stack.pop().ok_or("Stack underflow")?;
    let array = vm.stack.pop().ok_or("Stack underflow")?;
    match array {
        Value::Array(mut arr) => {
            let array_key = ArrayKey::from_value(&key);
            if let Some(pos) = arr.iter().position(|(k, _)| k == &array_key) {
                arr[pos] = (array_key, value);
            } else {
                arr.push((array_key, value));
            }
            vm.stack.push(Value::Array(arr));
        }
        _ => return Err("Cannot use [] on non-array".to_string()),
    }
    Ok(())
}

pub fn execute_array_append<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let array = vm.stack.pop().ok_or("Stack underflow")?;
    match array {
        Value::Array(mut arr) => {
            let next_idx = arr
                .iter()
                .filter_map(|(k, _)| match k {
                    ArrayKey::Integer(n) => Some(*n),
                    _ => None,
                })
                .max()
                .unwrap_or(-1)
                + 1;
            arr.push((ArrayKey::Integer(next_idx), value));
            vm.stack.push(Value::Array(arr));
        }
        _ => return Err("Cannot append to non-array".to_string()),
    }
    Ok(())
}

pub fn execute_array_merge<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let array2 = vm.stack.pop().ok_or("Stack underflow")?;
    let array1 = vm.stack.pop().ok_or("Stack underflow")?;
    match (array1, array2) {
        (Value::Array(mut arr1), Value::Array(arr2)) => {
            let next_idx = arr1
                .iter()
                .filter_map(|(k, _)| match k {
                    ArrayKey::Integer(n) => Some(*n),
                    _ => None,
                })
                .max()
                .unwrap_or(-1)
                + 1;

            for (i, (_, value)) in arr2.into_iter().enumerate() {
                arr1.push((ArrayKey::Integer(next_idx + i as i64), value));
            }
            vm.stack.push(Value::Array(arr1));
        }
        _ => return Err("Cannot merge non-arrays".to_string()),
    }
    Ok(())
}

pub fn execute_array_count<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let array = vm.stack.pop().unwrap();
    match array {
        Value::Array(arr) => {
            vm.stack.push(Value::Integer(arr.len() as i64));
        }
        _ => vm.stack.push(Value::Integer(0)),
    }
}

pub fn execute_array_get_key_at<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let index = vm.stack.pop().unwrap();
    let array = vm.stack.pop().unwrap();
    match (array, index) {
        (Value::Array(arr), Value::Integer(idx)) => {
            if idx >= 0 && (idx as usize) < arr.len() {
                let (key, _) = &arr[idx as usize];
                vm.stack.push(key.to_value());
            } else {
                vm.stack.push(Value::Null);
            }
        }
        _ => vm.stack.push(Value::Null),
    }
}

pub fn execute_array_get_value_at<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let index = vm.stack.pop().unwrap();
    let array = vm.stack.pop().unwrap();
    match (array, index) {
        (Value::Array(arr), Value::Integer(idx)) => {
            if idx >= 0 && (idx as usize) < arr.len() {
                let (_, value) = &arr[idx as usize];
                vm.stack.push(value.clone());
            } else {
                vm.stack.push(Value::Null);
            }
        }
        _ => vm.stack.push(Value::Null),
    }
}

pub fn execute_array_unpack<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let array = vm.stack.pop().unwrap();
    if let Value::Array(elements) = array {
        for (_, value) in elements {
            vm.stack.push(value);
        }
    }
}

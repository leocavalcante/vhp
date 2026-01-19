use crate::runtime::Value;

pub fn execute_concat<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;

    let left_str = vm.value_to_string(left)?;
    let right_str = vm.value_to_string(right)?;

    let result = Value::String(format!("{}{}", left_str, right_str));
    vm.stack.push(result);
    Ok(())
}

/// Execute heredoc interpolation
///
/// Stack contains: [str1, var1, str2, var2, ..., strN]
/// where there are N string parts and N-1 variables
/// We pop all values and concatenate them into one string
pub fn execute_heredoc_interpolate<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_count: u16,
) -> Result<(), String> {
    // Total items on stack = (var_count + 1) strings + var_count variables = 2*var_count + 1
    let total_items = (var_count as usize) * 2 + 1;

    // Collect all values from the stack
    let mut values: Vec<Value> = Vec::with_capacity(total_items);
    for _ in 0..total_items {
        match vm.stack.pop() {
            Some(v) => values.push(v),
            None => return Err("Stack underflow in heredoc interpolation".to_string()),
        }
    }

    // Reverse because we popped in reverse order
    values.reverse();

    // Convert all values to strings and concatenate
    let mut result = String::new();
    for value in values {
        let s = vm.value_to_string(value)?;
        result.push_str(&s);
    }

    vm.stack.push(Value::String(result));
    Ok(())
}

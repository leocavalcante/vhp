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

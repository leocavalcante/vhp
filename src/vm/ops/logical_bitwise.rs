use crate::runtime::Value;

pub fn execute_not<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    vm.stack.push(Value::Bool(!value.to_bool()));
    Ok(())
}

pub fn execute_and<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    vm.stack
        .push(Value::Bool(left.to_bool() && right.to_bool()));
    Ok(())
}

pub fn execute_or<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    vm.stack
        .push(Value::Bool(left.to_bool() || right.to_bool()));
    Ok(())
}

pub fn execute_xor<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    vm.stack.push(Value::Bool(left.to_bool() ^ right.to_bool()));
    Ok(())
}

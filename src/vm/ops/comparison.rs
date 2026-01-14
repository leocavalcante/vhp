use crate::runtime::Value;

pub fn execute_eq<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(left.loose_equals(&right)));
}

pub fn execute_ne<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(!left.loose_equals(&right)));
}

pub fn execute_identical<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(left.type_equals(&right)));
}

pub fn execute_not_identical<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(!left.type_equals(&right)));
}

pub fn execute_lt<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.compare_values(&left, &right)? < 0;
    vm.stack.push(Value::Bool(result));
    Ok(())
}

pub fn execute_le<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.compare_values(&left, &right)? <= 0;
    vm.stack.push(Value::Bool(result));
    Ok(())
}

pub fn execute_gt<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.compare_values(&left, &right)? > 0;
    vm.stack.push(Value::Bool(result));
    Ok(())
}

pub fn execute_ge<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.compare_values(&left, &right)? >= 0;
    vm.stack.push(Value::Bool(result));
    Ok(())
}

pub fn execute_spaceship<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.compare_values(&left, &right)?;
    vm.stack.push(Value::Integer(result));
    Ok(())
}

use crate::runtime::Value;

pub fn execute_not<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let value = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(!value.to_bool()));
}

pub fn execute_and<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(left.to_bool() && right.to_bool()));
}

pub fn execute_or<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(left.to_bool() || right.to_bool()));
}

pub fn execute_xor<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    vm.stack.push(Value::Bool(left.to_bool() ^ right.to_bool()));
}

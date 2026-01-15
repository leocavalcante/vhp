use crate::runtime::Value;
use crate::vm::VM;

pub fn execute_set_current_fiber<W: std::io::Write>(vm: &mut VM<W>) -> Result<(), String> {
    let fiber = vm.stack.pop().ok_or("Stack underflow")?;
    vm.current_fiber = Some(fiber);
    Ok(())
}

pub fn execute_get_current_fiber<W: std::io::Write>(vm: &mut VM<W>) -> Result<(), String> {
    let current = vm.current_fiber.clone().unwrap_or(Value::Null);
    vm.stack.push(current);
    Ok(())
}

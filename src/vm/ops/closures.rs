use crate::runtime::{Closure, ClosureBody, Value};

pub fn execute_create_closure<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name: String,
    capture_count: u8,
) -> Result<(), String> {
    let mut captured_vars: Vec<(String, Value)> = Vec::new();
    for _ in 0..capture_count {
        let value = vm.stack.pop().ok_or("Stack underflow")?;
        let var_name = vm.stack.pop().ok_or("Stack underflow")?;
        if let Value::String(name) = var_name {
            captured_vars.push((name, value));
        } else {
            return Err("CaptureVar expects variable name as string".to_string());
        }
    }

    let closure = Closure {
        params: Vec::new(),
        body: ClosureBody::FunctionRef(func_name),
        captured_vars,
    };
    vm.stack.push(Value::Closure(Box::new(closure)));
    Ok(())
}

pub fn execute_capture_var<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_name: String,
) -> Result<(), String> {
    let frame = vm.current_frame();
    let slot = frame
        .function
        .local_names
        .iter()
        .position(|name| name == &var_name)
        .map(|i| i as u16);

    let value = if let Some(slot) = slot {
        frame.locals[slot as usize].clone()
    } else {
        vm.globals.get(&var_name).cloned().unwrap_or(Value::Null)
    };

    vm.stack.push(Value::String(var_name));
    vm.stack.push(value);
    Ok(())
}

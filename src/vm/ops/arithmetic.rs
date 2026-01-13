use crate::runtime::Value;

pub fn execute_add<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = vm.add_values(left, right)?;
    vm.stack.push(result);
    Ok(())
}

pub fn execute_sub<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = match (&left, &right) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
        (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
        (Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
        _ => {
            let a = left.to_float();
            let b = right.to_float();
            Value::Float(a - b)
        }
    };
    vm.stack.push(result);
    Ok(())
}

pub fn execute_mul<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = match (&left, &right) {
        (Value::Integer(a), Value::Integer(b)) => Value::Integer(a * b),
        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
        (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 * b),
        (Value::Float(a), Value::Integer(b)) => Value::Float(a * *b as f64),
        _ => {
            let a = left.to_float();
            let b = right.to_float();
            Value::Float(a * b)
        }
    };
    vm.stack.push(result);
    Ok(())
}

pub fn execute_div<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let a = left.to_float();
    let b = right.to_float();
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    vm.stack.push(Value::Float(a / b));
    Ok(())
}

pub fn execute_mod<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let a = left.to_int();
    let b = right.to_int();
    if b == 0 {
        return Err("Division by zero".to_string());
    }
    vm.stack.push(Value::Integer(a % b));
    Ok(())
}

pub fn execute_pow<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let base = left.to_float();
    let exp = right.to_float();
    vm.stack.push(Value::Float(base.powf(exp)));
    Ok(())
}

pub fn execute_neg<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let result = match value {
        Value::Integer(n) => Value::Integer(-n),
        Value::Float(f) => Value::Float(-f),
        _ => Value::Integer(-value.to_int()),
    };
    vm.stack.push(result);
    Ok(())
}

pub fn execute_push_null<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.stack.push(Value::Null);
}

pub fn execute_push_true<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.stack.push(Value::Bool(true));
}

pub fn execute_push_false<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.stack.push(Value::Bool(false));
}

pub fn execute_push_int<W: std::io::Write>(vm: &mut super::super::VM<W>, n: i64) {
    vm.stack.push(Value::Integer(n));
}

pub fn execute_push_float<W: std::io::Write>(vm: &mut super::super::VM<W>, f: f64) {
    vm.stack.push(Value::Float(f));
}

pub fn execute_push_string<W: std::io::Write>(vm: &mut super::super::VM<W>, s: String) {
    vm.stack.push(Value::String(s));
}

pub fn execute_load_const<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    constant: crate::vm::opcode::Constant,
) -> Result<(), String> {
    let value = match constant {
        crate::vm::opcode::Constant::Null => Value::Null,
        crate::vm::opcode::Constant::Bool(b) => Value::Bool(b),
        crate::vm::opcode::Constant::Int(n) => Value::Integer(n),
        crate::vm::opcode::Constant::Float(f) => Value::Float(f),
        crate::vm::opcode::Constant::String(s) => Value::String(s),
    };
    vm.stack.push(value);
    Ok(())
}

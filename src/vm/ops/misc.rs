use crate::ast::Visibility;
use crate::runtime::{ArrayKey, Value};
use crate::vm::frame::{ExceptionHandler, LoopContext, ThisSource};
use crate::vm::opcode::CastType;

pub fn execute_load_var<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.globals.get(&name).cloned().unwrap_or(Value::Null);
    vm.stack.push(value);
}

pub fn execute_store_var<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.stack.pop().unwrap();
    vm.globals.insert(name.clone(), value.clone());
    vm.stack.push(value);
}

pub fn execute_load_fast<W: std::io::Write>(vm: &mut super::super::VM<W>, slot: u16) {
    let value = vm.current_frame().get_local(slot).clone();
    vm.stack.push(value);
}

pub fn execute_store_fast<W: std::io::Write>(vm: &mut super::super::VM<W>, slot: u16) {
    let value = vm.stack.pop().unwrap();
    vm.current_frame_mut().set_local(slot, value.clone());
    vm.stack.push(value);
}

pub fn execute_load_global<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.globals.get(&name).cloned().unwrap_or(Value::Null);
    vm.stack.push(value);
}

pub fn execute_store_global<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.stack.pop().unwrap();
    vm.globals.insert(name.clone(), value.clone());
    vm.stack.push(value);
}

pub fn execute_pop<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.stack.pop();
}

pub fn execute_dup<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let value = vm.stack.last().unwrap().clone();
    vm.stack.push(value);
}

pub fn execute_swap<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let len = vm.stack.len();
    if len < 2 {
        return Err("Stack underflow".to_string());
    }
    vm.stack.swap(len - 1, len - 2);
    Ok(())
}

pub fn execute_cast<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    cast_type: CastType,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let result = match cast_type {
        CastType::Int => Value::Integer(value.to_int()),
        CastType::Float => Value::Float(value.to_float()),
        CastType::String => Value::String(value.to_string_val()),
        CastType::Bool => Value::Bool(value.to_bool()),
        CastType::Array => match value {
            Value::Array(_) => value,
            _ => return Err("Cannot cast to array".to_string()),
        },
        CastType::Object => match value {
            Value::Object(_) => value,
            _ => return Err("Cannot cast to object".to_string()),
        },
    };
    vm.stack.push(result);
    Ok(())
}

pub fn execute_null_coalesce<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let right = vm.stack.pop().unwrap();
    let left = vm.stack.pop().unwrap();
    let result = if matches!(left, Value::Null) {
        right
    } else {
        left
    };
    vm.stack.push(result);
}

pub fn execute_echo<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    if let Value::Object(ref instance) = value {
        if let Some(to_string_method) = vm.find_method_in_chain(&instance.class_name, "__toString")
        {
            let result = vm.call_method_sync(instance.clone(), to_string_method)?;
            match result {
                Value::String(s) => write!(vm.output, "{}", s).map_err(|e| e.to_string())?,
                _ => {
                    return Err(format!(
                        "Return value must be of type string, {} returned",
                        result.get_type()
                    ))
                }
            }
        } else {
            return Err(format!(
                "Object of class {} could not be converted to string",
                instance.class_name
            ));
        }
    } else {
        write!(vm.output, "{}", value.to_output_string()).map_err(|e| e.to_string())?
    }
    Ok(())
}

pub fn execute_print<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    if let Value::Object(ref instance) = value {
        if let Some(to_string_method) = vm.find_method_in_chain(&instance.class_name, "__toString")
        {
            let result = vm.call_method_sync(instance.clone(), to_string_method)?;
            match result {
                Value::String(s) => write!(vm.output, "{}", s).map_err(|e| e.to_string())?,
                _ => {
                    return Err(format!(
                        "Return value must be of type string, {} returned",
                        result.get_type()
                    ))
                }
            }
        } else {
            return Err(format!(
                "Object of class {} could not be converted to string",
                instance.class_name
            ));
        }
    } else {
        write!(vm.output, "{}", value.to_output_string()).map_err(|e| e.to_string())?
    }
    vm.stack.push(Value::Integer(1));
    Ok(())
}

pub fn execute_unset_var<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    vm.globals.remove(&name);
}

pub fn execute_unset_array_element<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let key = vm.stack.pop().ok_or("Stack underflow")?;
    let array = vm.stack.pop().ok_or("Stack underflow")?;

    match array {
        Value::Array(mut arr) => {
            let array_key = match key {
                Value::Integer(n) => ArrayKey::Integer(n),
                Value::String(s) => ArrayKey::String(s),
                _ => return Err(format!("Invalid array key type: {:?}", key)),
            };
            arr.retain(|(k, _)| k != &array_key);
        }
        _ => return Err("Cannot unset element of non-array".to_string()),
    }
    Ok(())
}

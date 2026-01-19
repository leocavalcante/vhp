use crate::runtime::{ArrayKey, Closure, ClosureBody, Value};
use crate::vm::opcode::CastType;

pub fn execute_load_var<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.globals.get(&name).cloned().unwrap_or(Value::Null);
    vm.stack.push(value);
}

pub fn execute_store_var<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    name: String,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    vm.globals.insert(name.clone(), value.clone());
    vm.stack.push(value);
    Ok(())
}

pub fn execute_load_fast<W: std::io::Write>(vm: &mut super::super::VM<W>, slot: u16) {
    let value = vm.current_frame().get_local(slot).clone();
    vm.stack.push(value);
}

pub fn execute_store_fast<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    slot: u16,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    vm.current_frame_mut().set_local(slot, value.clone());
    vm.stack.push(value);
    Ok(())
}

pub fn execute_load_global<W: std::io::Write>(vm: &mut super::super::VM<W>, name: String) {
    let value = vm.globals.get(&name).cloned().unwrap_or(Value::Null);
    vm.stack.push(value);
}

pub fn execute_store_global<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    name: String,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    vm.globals.insert(name.clone(), value.clone());
    vm.stack.push(value);
    Ok(())
}

pub fn execute_pop<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.stack.pop();
}

pub fn execute_dup<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.last().ok_or("Stack is empty")?.clone();
    vm.stack.push(value);
    Ok(())
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

pub fn execute_null_coalesce<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let right = vm.stack.pop().ok_or("Stack underflow")?;
    let left = vm.stack.pop().ok_or("Stack underflow")?;
    let result = if matches!(left, Value::Null) {
        right
    } else {
        left
    };
    vm.stack.push(result);
    Ok(())
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

pub fn execute_capture_var<W: std::io::Write>(vm: &mut super::super::VM<W>, var_name: String) {
    let value = {
        let frame = vm.current_frame();
        let slot = frame
            .function
            .local_names
            .iter()
            .position(|name| name == &var_name)
            .map(|i| i as u16);

        if let Some(slot) = slot {
            frame.locals[slot as usize].clone()
        } else {
            vm.globals.get(&var_name).cloned().unwrap_or(Value::Null)
        }
    };

    vm.stack.push(Value::String(var_name));
    vm.stack.push(value);
}

pub fn execute_new_fiber<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let callback = vm.stack.pop().ok_or("Stack underflow")?;

    let fiber_class = vm
        .classes
        .get("Fiber")
        .ok_or("Fiber class not found")?
        .clone();

    let mut instance = crate::runtime::ObjectInstance::with_hierarchy(
        "Fiber".to_string(),
        fiber_class.parent.clone(),
        fiber_class.interfaces.clone(),
    );

    for prop in &fiber_class.properties {
        let default_val = prop.default.clone().unwrap_or(Value::Null);
        instance
            .properties
            .insert(prop.name.clone(), default_val.clone());
        if prop.readonly {
            instance.readonly_properties.insert(prop.name.clone());
            if prop.default.is_some() {
                instance.initialized_readonly.insert(prop.name.clone());
            }
        }
    }

    instance
        .properties
        .insert("__callback".to_string(), callback);

    vm.stack.push(Value::Object(instance));
    Ok(())
}

use crate::runtime::Value;
use crate::vm::frame::ThisSource;

pub fn execute_load_property<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    prop_name: String,
) -> Result<(), String> {
    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(instance) => {
            if let Some(class) = vm.classes.get(&instance.class_name).cloned() {
                if let Some(prop_def) = class.properties.iter().find(|p| p.name == prop_name) {
                    if let Some(ref hook_method_name) = prop_def.get_hook {
                        if let Some(hook_method) = class.methods.get(hook_method_name).cloned() {
                            let stack_base = vm.stack.len();
                            let mut frame =
                                super::super::frame::CallFrame::new(hook_method, stack_base);
                            frame.locals[0] = Value::Object(instance);
                            vm.frames.push(frame);
                            return Ok(());
                        }
                    }
                }
            }

            if let Some(value) = instance.properties.get(&prop_name).cloned() {
                vm.stack.push(value);
            } else if let Some(get_method) = vm.find_method_in_chain(&instance.class_name, "__get")
            {
                vm.stack.push(Value::String(prop_name));
                let stack_base = vm.stack.len();
                let mut frame = super::super::frame::CallFrame::new(get_method, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.locals[1] = vm.stack.pop().unwrap();
                vm.frames.push(frame);
            } else {
                vm.stack.push(Value::Null);
            }
        }
        Value::EnumCase {
            enum_name,
            case_name,
            backing_value,
        } => {
            let value = match prop_name.as_str() {
                "name" => Value::String(case_name),
                "value" => {
                    if let Some(bv) = backing_value {
                        *bv
                    } else {
                        return Err(format!(
                            "Pure enum case {}::{} does not have a 'value' property",
                            enum_name, case_name
                        ));
                    }
                }
                _ => return Err(format!("Undefined property: {}::{}", enum_name, prop_name)),
            };
            vm.stack.push(value);
        }
        _ => {
            return Err(format!(
                "Cannot access property of non-object: {:?}",
                object
            ))
        }
    }
    Ok(())
}

pub fn execute_unset_property<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    prop_name: String,
) -> Result<(), String> {
    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(mut instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if !prop_defined_in_class {
                if let Some(unset_method) = vm.find_method_in_chain(&instance.class_name, "__unset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(unset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.this_source = ThisSource::PropertySetHook;
                    vm.frames.push(frame);
                    return Ok(());
                }
            }

            if instance.properties.contains_key(&prop_name) {
                instance.properties.remove(&prop_name);
            }
        }
        _ => return Err("Cannot unset property on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_isset_property<W: std::io::Write>(vm: &mut super::super::VM<W>, prop_name: String) {
    let object = vm.stack.pop().unwrap();

    match object {
        Value::Object(instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if let Some(value) = instance.properties.get(&prop_name) {
                let is_set = !matches!(value, Value::Null);
                vm.stack.push(Value::Bool(is_set));
            } else if !prop_defined_in_class {
                if let Some(isset_method) = vm.find_method_in_chain(&instance.class_name, "__isset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(isset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    vm.frames.push(frame);
                } else {
                    vm.stack.push(Value::Bool(false));
                }
            } else {
                vm.stack.push(Value::Bool(false));
            }
        }
        Value::Null => {
            vm.stack.push(Value::Bool(false));
        }
        _ => {
            vm.stack.push(Value::Bool(false));
        }
    }
}

pub fn execute_unset_property_on_local<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    slot: u16,
    prop_name: String,
) -> Result<(), String> {
    let object = vm.current_frame().locals[slot as usize].clone();

    match object {
        Value::Object(mut instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if !prop_defined_in_class {
                if let Some(unset_method) = vm.find_method_in_chain(&instance.class_name, "__unset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(unset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.this_source = ThisSource::LocalSlot(slot);
                    vm.frames.push(frame);
                    return Ok(());
                }
            }

            if instance.properties.contains_key(&prop_name) {
                instance.properties.remove(&prop_name);
                if let Some(frame) = vm.frames.last_mut() {
                    frame.set_local(slot, Value::Object(instance));
                }
            }
        }
        _ => return Err("Cannot unset property on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_isset_property_on_local<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    slot: u16,
    prop_name: String,
) {
    let object = vm.current_frame().locals[slot as usize].clone();

    match object {
        Value::Object(instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if let Some(value) = instance.properties.get(&prop_name) {
                let is_set = !matches!(value, Value::Null);
                vm.stack.push(Value::Bool(is_set));
            } else if !prop_defined_in_class {
                if let Some(isset_method) = vm.find_method_in_chain(&instance.class_name, "__isset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(isset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.this_source = ThisSource::LocalSlot(slot);
                    vm.frames.push(frame);
                } else {
                    vm.stack.push(Value::Bool(false));
                }
            } else {
                vm.stack.push(Value::Bool(false));
            }
        }
        Value::Null => {
            vm.stack.push(Value::Bool(false));
        }
        _ => {
            vm.stack.push(Value::Bool(false));
        }
    }
}

pub fn execute_unset_property_on_global<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_name: String,
    prop_name: String,
) -> Result<(), String> {
    let object = vm.globals.get(&var_name).cloned().unwrap_or(Value::Null);

    match object {
        Value::Object(mut instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if !prop_defined_in_class {
                if let Some(unset_method) = vm.find_method_in_chain(&instance.class_name, "__unset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(unset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.this_source = ThisSource::GlobalVar(var_name);
                    vm.frames.push(frame);
                    return Ok(());
                }
            }

            if instance.properties.contains_key(&prop_name) {
                instance.properties.remove(&prop_name);
                vm.globals.insert(var_name, Value::Object(instance));
            }
        }
        _ => return Err("Cannot unset property on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_isset_property_on_global<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_name: String,
    prop_name: String,
) {
    let object = vm.globals.get(&var_name).cloned().unwrap_or(Value::Null);

    match object {
        Value::Object(instance) => {
            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if let Some(value) = instance.properties.get(&prop_name) {
                let is_set = !matches!(value, Value::Null);
                vm.stack.push(Value::Bool(is_set));
            } else if !prop_defined_in_class {
                if let Some(isset_method) = vm.find_method_in_chain(&instance.class_name, "__isset")
                {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(isset_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.this_source = ThisSource::GlobalVar(var_name);
                    vm.frames.push(frame);
                } else {
                    vm.stack.push(Value::Bool(false));
                }
            } else {
                vm.stack.push(Value::Bool(false));
            }
        }
        Value::Null => {
            vm.stack.push(Value::Bool(false));
        }
        _ => {
            vm.stack.push(Value::Bool(false));
        }
    }
}

use crate::runtime::Value;
use crate::vm::frame::ThisSource;

fn normalize_class_name(name: &str) -> String {
    if let Some(stripped) = name.strip_prefix('\\') {
        stripped.to_string()
    } else {
        name.to_string()
    }
}

pub fn execute_new_object<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
) -> Result<(), String> {
    let class_name = normalize_class_name(&class_name);
    let class_def = vm
        .classes
        .get(&class_name)
        .ok_or_else(|| format!("Class '{}' not found", class_name))?
        .clone();

    if class_def.is_abstract {
        return Err(format!("Cannot instantiate abstract class {}", class_name));
    }

    let mut instance = crate::runtime::ObjectInstance::with_hierarchy(
        class_name.clone(),
        class_def.parent.clone(),
        class_def.interfaces.clone(),
    );

    let mut parent_chain = Vec::new();
    let mut current_parent = class_def.parent.as_ref();
    while let Some(parent_name) = current_parent {
        if let Some(parent_def) = vm.classes.get(parent_name) {
            parent_chain.push(parent_def.clone());
            current_parent = parent_def.parent.as_ref();
        } else {
            break;
        }
    }

    for parent_def in parent_chain.iter().rev() {
        for prop in &parent_def.properties {
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
    }

    for prop in &class_def.properties {
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

    vm.stack.push(Value::Object(instance));
    Ok(())
}

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
            } else {
                if let Some(get_method) = vm.find_method_in_chain(&instance.class_name, "__get") {
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

pub fn execute_load_this<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let frame = vm.current_frame();
    let this = frame
        .locals
        .first()
        .cloned()
        .ok_or("No $this available in current context")?;
    vm.stack.push(this);
    Ok(())
}

pub fn execute_instance_of<W: std::io::Write>(vm: &mut super::super::VM<W>, class_name: String) {
    let class_name = normalize_class_name(&class_name);
    let object = vm.stack.pop().unwrap();

    let result = match object {
        Value::Object(instance) => {
            instance.class_name == class_name
                || instance.parent_class.as_ref() == Some(&class_name)
                || instance.interfaces.contains(&class_name)
        }
        _ => false,
    };
    vm.stack.push(Value::Bool(result));
}

pub fn execute_clone<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let object = vm.stack.pop().ok_or("Stack underflow")?;
    match object {
        Value::Object(instance) => {
            let cloned = instance.clone();
            vm.stack.push(Value::Object(cloned));
        }
        _ => return Err("__clone method called on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_load_enum_case<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    enum_name: String,
    case_name: String,
) -> Result<(), String> {
    let enum_name = normalize_class_name(&enum_name);
    let enum_def = vm
        .enums
        .get(&enum_name)
        .ok_or_else(|| format!("Enum '{}' not found", enum_name))?
        .clone();

    let backing_value = enum_def
        .cases
        .get(&case_name)
        .ok_or_else(|| format!("Undefined case '{}' for enum '{}'", case_name, enum_name))?
        .clone()
        .map(Box::new);

    vm.stack.push(Value::EnumCase {
        enum_name,
        case_name,
        backing_value,
    });
    Ok(())
}

pub fn execute_store_this_property<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    prop_name: String,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;

    let this = vm.current_frame().get_local(0).clone();
    match this {
        Value::Object(mut instance) => {
            if instance.readonly_properties.contains(&prop_name)
                && instance.initialized_readonly.contains(&prop_name)
            {
                return Err(format!("Cannot modify readonly property {}", prop_name));
            }
            instance.properties.insert(prop_name.clone(), value.clone());
            if instance.readonly_properties.contains(&prop_name) {
                instance.initialized_readonly.insert(prop_name);
            }
            vm.current_frame_mut().set_local(0, Value::Object(instance));
            vm.stack.push(value);
        }
        _ => return Err("$this is not an object".to_string()),
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

pub fn execute_store_property<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    prop_name: String,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(instance) => {
            if let Some(class) = vm.classes.get(&instance.class_name).cloned() {
                if let Some(prop_def) = class.properties.iter().find(|p| p.name == prop_name) {
                    if let Some(write_vis) = &prop_def.write_visibility {
                        let current_class = vm.get_current_class();
                        let can_write = match write_vis {
                            crate::ast::Visibility::Private => {
                                current_class.as_ref() == Some(&instance.class_name)
                            }
                            crate::ast::Visibility::Protected => {
                                if let Some(ref curr) = current_class {
                                    curr == &instance.class_name
                                        || vm.is_subclass_of(curr, &instance.class_name)
                                } else {
                                    false
                                }
                            }
                            crate::ast::Visibility::Public => true,
                        };
                        if !can_write {
                            let vis_str = match write_vis {
                                crate::ast::Visibility::Private => "private",
                                crate::ast::Visibility::Protected => "protected",
                                crate::ast::Visibility::Public => "public",
                            };
                            return Err(format!(
                                "Cannot modify {} property {}",
                                vis_str, prop_name
                            ));
                        }
                    }

                    if prop_def.get_hook.is_some() && prop_def.set_hook.is_none() {
                        return Err(format!("Cannot write to read-only property {}", prop_name));
                    }

                    if let Some(ref hook_method_name) = prop_def.set_hook {
                        if let Some(hook_method) = class.methods.get(hook_method_name).cloned() {
                            let stack_base = vm.stack.len();
                            let mut frame =
                                super::super::frame::CallFrame::new(hook_method, stack_base);
                            frame.locals[0] = Value::Object(instance);
                            frame.locals[1] = value;
                            frame.this_source = ThisSource::PropertySetHook;
                            vm.frames.push(frame);
                            return Ok(());
                        }
                    }
                }
            }

            let prop_defined_in_class = if let Some(class) = vm.classes.get(&instance.class_name) {
                class.properties.iter().any(|p| p.name == prop_name)
            } else {
                false
            };

            if !prop_defined_in_class && !instance.properties.contains_key(&prop_name) {
                if let Some(set_method) = vm.find_method_in_chain(&instance.class_name, "__set") {
                    let stack_base = vm.stack.len();
                    let mut frame = super::super::frame::CallFrame::new(set_method, stack_base);
                    frame.locals[0] = Value::Object(instance);
                    frame.locals[1] = Value::String(prop_name);
                    frame.locals[2] = value;
                    frame.this_source = ThisSource::PropertySetHook;
                    vm.frames.push(frame);
                    return Ok(());
                }
            }

            let mut instance = instance;
            if instance.readonly_properties.contains(&prop_name)
                && instance.initialized_readonly.contains(&prop_name)
            {
                return Err(format!("Cannot modify readonly property {}", prop_name));
            }
            instance.properties.insert(prop_name.clone(), value.clone());
            if instance.readonly_properties.contains(&prop_name) {
                instance.initialized_readonly.insert(prop_name);
            }
            vm.stack.push(Value::Object(instance));
        }
        _ => return Err("Cannot set property on non-object".to_string()),
    }
    Ok(())
}

pub fn execute_store_clone_property<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    prop_idx: u32,
) -> Result<(), String> {
    let prop_name = vm.current_frame().get_string(prop_idx).to_string();
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(mut instance) => {
            if !instance.properties.contains_key(&prop_name) {
                return Err(format!(
                    "Property '{}' does not exist on class '{}'",
                    prop_name, instance.class_name
                ));
            }

            instance.properties.insert(prop_name.clone(), value.clone());
            if instance.readonly_properties.contains(&prop_name) {
                instance.initialized_readonly.insert(prop_name);
            }
            vm.stack.push(Value::Object(instance));
        }
        _ => return Err("Cannot set property on non-object".to_string()),
    }
    Ok(())
}

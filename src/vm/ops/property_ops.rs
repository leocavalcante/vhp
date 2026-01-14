use crate::runtime::Value;
use crate::vm::frame::ThisSource;

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

use crate::runtime::{ArrayKey, Closure, ClosureBody, Value};
use crate::vm::frame::CallFrame;

pub fn execute_call_static_method<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
    method_name: String,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    let resolved_class = vm.resolve_class_keyword(&class_name)?;

    if let Some((method, is_instance_method)) =
        vm.find_static_method_in_chain(&resolved_class, &method_name)
    {
        let stack_base = vm.stack.len();
        let mut frame = CallFrame::new(method, stack_base);
        frame.called_class = Some(resolved_class.clone());

        let param_start = if is_instance_method { 1 } else { 0 };
        for (i, arg) in args.into_iter().enumerate() {
            let slot = param_start + i;
            if slot < frame.locals.len() {
                frame.locals[slot] = arg;
            }
        }

        vm.frames.push(frame);
    } else if let Some((magic_call_static, _)) =
        vm.find_static_method_in_chain(&resolved_class, "__callStatic")
    {
        let stack_base = vm.stack.len();
        let mut frame = CallFrame::new(magic_call_static, stack_base);
        frame.called_class = Some(resolved_class.clone());
        frame.locals[0] = Value::String(method_name);
        let args_array: Vec<(ArrayKey, Value)> = args
            .into_iter()
            .enumerate()
            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
            .collect();
        frame.locals[1] = Value::Array(args_array);

        vm.frames.push(frame);
    } else if let Some(enum_def) = vm.enums.get(&resolved_class).cloned() {
        match method_name.as_str() {
            "cases" => {
                let cases: Vec<(ArrayKey, Value)> = enum_def
                    .case_order
                    .iter()
                    .enumerate()
                    .filter_map(|(i, name)| {
                        enum_def.cases.get(name).map(|value| {
                            (
                                ArrayKey::Integer(i as i64),
                                Value::EnumCase {
                                    enum_name: resolved_class.clone(),
                                    case_name: name.clone(),
                                    backing_value: value.clone().map(Box::new),
                                },
                            )
                        })
                    })
                    .collect();
                vm.stack.push(Value::Array(cases));
            }
            "from" => {
                if args.is_empty() {
                    return Err("from() requires exactly one argument".to_string());
                }
                let search_value = &args[0];
                let mut found = None;
                for (name, backing) in &enum_def.cases {
                    if let Some(bv) = backing {
                        if bv == search_value {
                            found = Some((name.clone(), backing.clone()));
                            break;
                        }
                    }
                }
                if let Some((name, backing)) = found {
                    vm.stack.push(Value::EnumCase {
                        enum_name: resolved_class.clone(),
                        case_name: name,
                        backing_value: backing.map(Box::new),
                    });
                } else {
                    let value_str = match &search_value {
                        Value::Integer(n) => n.to_string(),
                        Value::String(s) => format!("'{}'", s),
                        other => format!("{:?}", other),
                    };
                    return Err(format!(
                        "Value '{}' is not a valid backing value for enum {}",
                        value_str, resolved_class
                    ));
                }
            }
            "tryFrom" => {
                if args.is_empty() {
                    return Err("tryFrom() requires exactly one argument".to_string());
                }
                let search_value = &args[0];
                let mut found = None;
                for (name, backing) in &enum_def.cases {
                    if let Some(bv) = backing {
                        if bv == search_value {
                            found = Some((name.clone(), backing.clone()));
                            break;
                        }
                    }
                }
                if let Some((name, backing)) = found {
                    vm.stack.push(Value::EnumCase {
                        enum_name: resolved_class.clone(),
                        case_name: name,
                        backing_value: backing.map(Box::new),
                    });
                } else {
                    vm.stack.push(Value::Null);
                }
            }
            _ => {
                if let Some(method) = enum_def.methods.get(&method_name) {
                    let stack_base = vm.stack.len();
                    let mut frame = CallFrame::new(method.clone(), stack_base);
                    for (i, arg) in args.into_iter().enumerate() {
                        if i < frame.locals.len() {
                            frame.locals[i] = arg;
                        }
                    }
                    vm.frames.push(frame);
                } else {
                    return Err(format!(
                        "Static method '{}' not found on enum '{}'",
                        method_name, resolved_class
                    ));
                }
            }
        }
    } else {
        return Err(format!(
            "Static method '{}' not found on class '{}'",
            method_name, resolved_class
        ));
    }

    Ok(())
}

pub fn execute_call_static_method_named<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
    method_name: String,
) -> Result<(), String> {
    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let resolved_class = vm.resolve_class_keyword(&class_name)?;

    let args_map = if let Value::Array(arr) = args_array {
        arr
    } else {
        return Err("Named static method args must be an array".to_string());
    };

    let mut positional = Vec::new();
    let mut named = std::collections::HashMap::new();

    for (key, value) in args_map {
        match key {
            ArrayKey::Integer(idx) => {
                positional.push((idx as usize, value));
            }
            ArrayKey::String(name) => {
                named.insert(name, value);
            }
        }
    }

    positional.sort_by_key(|(idx, _)| *idx);

    if let Some((method, is_instance_method)) =
        vm.find_static_method_in_chain(&resolved_class, &method_name)
    {
        let param_count = method.param_count as usize;
        let mut final_args = vec![Value::Null; param_count];

        for (i, (_, value)) in positional.into_iter().enumerate() {
            if i < param_count {
                final_args[i] = value;
            }
        }

        for (param_idx, param) in method.parameters.iter().enumerate() {
            if let Some(value) = named.get(&param.name) {
                if param_idx < param_count {
                    final_args[param_idx] = value.clone();
                }
            }
        }

        let stack_base = vm.stack.len();
        let mut frame = CallFrame::new(method, stack_base);
        frame.called_class = Some(resolved_class.clone());

        let param_start = if is_instance_method { 1 } else { 0 };
        for (i, arg) in final_args.into_iter().enumerate() {
            let slot = param_start + i;
            if slot < frame.locals.len() {
                frame.locals[slot] = arg;
            }
        }

        vm.frames.push(frame);
    } else {
        return Err(format!(
            "Static method '{}' not found on class '{}'",
            method_name, resolved_class
        ));
    }

    Ok(())
}

pub fn execute_load_static_prop<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
    prop_name: String,
) -> Result<(), String> {
    let resolved_class = vm.resolve_class_keyword(&class_name)?;

    let class_def = vm
        .classes
        .get(&resolved_class)
        .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;

    let value = class_def
        .static_properties
        .get(&prop_name)
        .cloned()
        .ok_or_else(|| {
            format!(
                "Access to undeclared static property {}::${}",
                resolved_class, prop_name
            )
        })?;
    vm.stack.push(value);

    Ok(())
}

pub fn execute_store_static_prop<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    class_name: String,
    prop_name: String,
) -> Result<(), String> {
    let value = vm.stack.pop().ok_or("Stack underflow")?;
    let resolved_class = vm.resolve_class_keyword(&class_name)?;

    if let Some(class_def) = vm.classes.get(&resolved_class) {
        if class_def.readonly_static_properties.contains(&prop_name) {
            return Err(format!(
                "Cannot modify readonly property {}::${}",
                resolved_class, prop_name
            ));
        }

        if let Some(prop_def) = class_def
            .properties
            .iter()
            .find(|p| p.name == prop_name && p.is_static)
        {
            if let Some(write_vis) = &prop_def.write_visibility {
                let current_class = vm.get_current_class();
                let can_write = match write_vis {
                    crate::ast::Visibility::Private => {
                        current_class.as_ref() == Some(&resolved_class)
                    }
                    crate::ast::Visibility::Protected => {
                        if let Some(ref curr) = current_class {
                            curr == &resolved_class || vm.is_subclass_of(curr, &resolved_class)
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
                        "Cannot modify {} property {}::${}",
                        vis_str, resolved_class, prop_name
                    ));
                }
            }
        }
    }

    let class_def = vm
        .classes
        .get_mut(&resolved_class)
        .ok_or_else(|| format!("Class '{}' not found", resolved_class))?;
    std::sync::Arc::make_mut(class_def)
        .static_properties
        .insert(prop_name, value.clone());

    vm.stack.push(value);

    Ok(())
}

pub fn execute_call_constructor<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(instance) => {
            let class_name = instance.class_name.clone();

            if let Some(constructor) = vm.find_method_in_chain(&class_name, "__construct") {
                let constructor = constructor.clone();

                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(constructor, stack_base);

                frame.locals[0] = Value::Object(instance);

                for (i, arg) in args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        let coerced_arg = if i < frame.function.param_types.len() {
                            if let Some(ref type_hint) = frame.function.param_types[i] {
                                if !vm.requires_strict_type_check(type_hint) {
                                    if !vm.value_matches_type(&arg, type_hint) {
                                        let type_name = vm.format_type_hint(type_hint);
                                        let given_type = vm.get_value_type_name(&arg);
                                        return Err(format!(
                                            "must be of type {}, {} given",
                                            type_name, given_type
                                        ));
                                    }
                                    vm.coerce_value_to_type(arg, type_hint)
                                } else {
                                    if !vm.value_matches_type_strict(&arg, type_hint) {
                                        let type_name = vm.format_type_hint(type_hint);
                                        let given_type = vm.get_value_type_name(&arg);
                                        return Err(format!(
                                            "must be of type {}, {} given",
                                            type_name, given_type
                                        ));
                                    }
                                    arg
                                }
                            } else {
                                arg
                            }
                        } else {
                            arg
                        };
                        frame.locals[i + 1] = coerced_arg;
                    }
                }

                frame.is_constructor = true;

                vm.frames.push(frame);
            } else {
                vm.stack.push(Value::Object(instance));
            }
        }
        _ => return Err("Cannot call constructor on non-object".to_string()),
    }

    Ok(())
}

pub fn execute_call_constructor_named<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let object = vm.stack.pop().ok_or("Stack underflow")?;

    match object {
        Value::Object(instance) => {
            let class_name = instance.class_name.clone();

            if let Some(constructor) = vm.find_method_in_chain(&class_name, "__construct") {
                let constructor = constructor.clone();

                let args_map = if let Value::Array(arr) = args_array {
                    arr
                } else {
                    return Err("Named constructor args must be an array".to_string());
                };

                let mut positional = Vec::new();
                let mut named = std::collections::HashMap::new();

                for (key, value) in args_map {
                    match key {
                        ArrayKey::Integer(idx) => {
                            positional.push((idx as usize, value));
                        }
                        ArrayKey::String(name) => {
                            named.insert(name, value);
                        }
                    }
                }

                positional.sort_by_key(|(idx, _)| *idx);

                let param_count = constructor.param_count as usize;
                let mut final_args = vec![Value::Null; param_count];

                for (i, (_, value)) in positional.into_iter().enumerate() {
                    if i < param_count {
                        final_args[i] = value;
                    }
                }

                for (param_idx, param) in constructor.parameters.iter().enumerate() {
                    if let Some(value) = named.get(&param.name) {
                        if param_idx < param_count {
                            final_args[param_idx] = value.clone();
                        }
                    }
                }

                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(constructor, stack_base);

                frame.locals[0] = Value::Object(instance);

                for (i, arg) in final_args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        frame.locals[i + 1] = arg;
                    }
                }

                frame.is_constructor = true;

                vm.frames.push(frame);
            } else {
                vm.stack.push(Value::Object(instance));
            }
        }
        _ => return Err("Cannot call constructor on non-object".to_string()),
    }

    Ok(())
}

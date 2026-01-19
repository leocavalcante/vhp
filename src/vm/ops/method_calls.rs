use crate::runtime::{ArrayKey, Value};
use crate::vm::frame::{CallFrame, ThisSource};

pub fn execute_call_method<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    method_name: String,
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

            if let Some(method) = vm.find_method_in_chain(&class_name, &method_name) {
                for (i, arg) in args.iter().enumerate() {
                    if i < method.param_types.len() {
                        if let Some(ref type_hint) = method.param_types[i] {
                            if vm.requires_strict_type_check(type_hint)
                                && !vm.value_matches_type(arg, type_hint)
                            {
                                let type_name = vm.format_type_hint(type_hint);
                                let given_type = vm.get_value_type_name(arg);
                                return Err(format!(
                                    "Argument {} passed to {}::{}() must be of type {}, {} given",
                                    i + 1,
                                    class_name,
                                    method_name,
                                    type_name,
                                    given_type
                                ));
                            }
                        }
                    }
                }

                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(method, stack_base);
                frame.locals[0] = Value::Object(instance);

                for (i, arg) in args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        frame.locals[i + 1] = arg;
                    }
                }

                vm.frames.push(frame);
            } else if let Some(magic_call) = vm.find_method_in_chain(&class_name, "__call") {
                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(magic_call, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.locals[1] = Value::String(method_name);
                let args_array: Vec<(ArrayKey, Value)> = args
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                    .collect();
                frame.locals[2] = Value::Array(args_array);

                vm.frames.push(frame);
            } else {
                return Err(format!(
                    "Method '{}' not found on class '{}'",
                    method_name, class_name
                ));
            }
        }
        Value::Generator(_) => {
            let mut gen = if let Value::Generator(g) = object {
                g
            } else {
                unreachable!()
            };
            match method_name.as_str() {
                "current" => {
                    let current = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(current);
                }
                "key" => {
                    let key = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .0
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(key);
                }
                "next" => {
                    gen.current_index += 1;
                    let valid = gen.current_index < gen.yielded_values.len() && !gen.finished;
                    if !valid {
                        gen.finished = true;
                    }
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(Value::Bool(valid));
                }
                "rewind" => {
                    // Rewind is handled by resetting current_index
                    gen.current_index = 0;
                    vm.stack.push(Value::Generator(gen));
                }
                "valid" => {
                    let valid = gen.current_index < gen.yielded_values.len() && !gen.finished;
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(Value::Bool(valid));
                }
                "getReturn" => {
                    let ret = gen.return_value.clone().unwrap_or(Value::Null);
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(ret);
                }
                "send" => {
                    let sent = args.first().cloned().unwrap_or(Value::Null);
                    gen.sent_value = Some(sent.clone());

                    // Advance to next yield and return its value
                    gen.current_index += 1;

                    if gen.current_index >= gen.yielded_values.len() {
                        // Generator is exhausted
                        gen.finished = true;
                        let ret = gen.return_value.clone().unwrap_or(Value::Null);
                        vm.stack.push(Value::Generator(gen));
                        vm.stack.push(ret);
                    } else {
                        let result = gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null);
                        vm.stack.push(Value::Generator(gen));
                        vm.stack.push(result);
                    }
                }
                "throw" => {
                    // Generator::throw() throws an exception into the generator
                    // For stub implementation, we mark it as finished and return null
                    gen.finished = true;
                    vm.stack.push(Value::Generator(gen));
                    vm.stack.push(Value::Null);
                }
                _ => {
                    return Err(format!("Method '{}' not found on Generator", method_name));
                }
            }
        }
        _ => return Err("Cannot call method on non-object".to_string()),
    }

    Ok(())
}

pub fn execute_call_method_on_local<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_slot: u16,
    method_name: String,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    let object = vm.current_frame().get_local(var_slot).clone();

    match object {
        Value::Object(instance) => {
            let class_name = instance.class_name.clone();

            if let Some(method) = vm.find_method_in_chain(&class_name, &method_name) {
                for (i, arg) in args.iter().enumerate() {
                    if i < method.param_types.len() {
                        if let Some(ref type_hint) = method.param_types[i] {
                            if vm.requires_strict_type_check(type_hint)
                                && !vm.value_matches_type(arg, type_hint)
                            {
                                let type_name = vm.format_type_hint(type_hint);
                                let given_type = vm.get_value_type_name(arg);
                                return Err(format!(
                                    "Argument {} passed to {}::{}() must be of type {}, {} given",
                                    i + 1,
                                    class_name,
                                    method_name,
                                    type_name,
                                    given_type
                                ));
                            }
                        }
                    }
                }

                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(method, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.this_source = ThisSource::LocalSlot(var_slot);

                for (i, arg) in args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        frame.locals[i + 1] = arg;
                    }
                }

                vm.frames.push(frame);
            } else if let Some(magic_call) = vm.find_method_in_chain(&class_name, "__call") {
                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(magic_call, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.this_source = ThisSource::LocalSlot(var_slot);
                frame.locals[1] = Value::String(method_name);
                let args_array: Vec<(ArrayKey, Value)> = args
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                    .collect();
                frame.locals[2] = Value::Array(args_array);

                vm.frames.push(frame);
            } else {
                return Err(format!(
                    "Method '{}' not found on class '{}'",
                    method_name, class_name
                ));
            }
        }
        Value::Generator(_) => {
            let mut gen = if let Value::Generator(g) = object {
                g
            } else {
                unreachable!()
            };
            match method_name.as_str() {
                "current" => {
                    let current = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(current);
                }
                "key" => {
                    let key = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .0
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(key);
                }
                "next" => {
                    gen.current_index += 1;
                    vm.stack
                        .push(Value::Bool(gen.current_index < gen.yielded_values.len()));
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                "rewind" => {
                    gen.current_index = 0;
                    gen.is_rewound = true;
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                "valid" => {
                    let valid = gen.current_index < gen.yielded_values.len() && !gen.finished;
                    vm.stack.push(Value::Bool(valid));
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                "getReturn" => {
                    let ret = gen.return_value.clone().unwrap_or(Value::Null);
                    vm.stack.push(ret);
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                "send" => {
                    let _sent = args.first().cloned().unwrap_or(Value::Null);
                    gen.current_index += 1;
                    let result = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(result);
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                "throw" => {
                    vm.stack.push(Value::Null);
                    vm.current_frame_mut()
                        .set_local(var_slot, Value::Generator(gen));
                }
                _ => {
                    return Err(format!("Method '{}' not found on Generator", method_name));
                }
            }
        }
        _ => return Err("Cannot call method on non-object".to_string()),
    }

    Ok(())
}

pub fn execute_call_method_on_global<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    var_name: String,
    method_name: String,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    let object = vm.globals.get(&var_name).cloned().unwrap_or(Value::Null);

    match object {
        Value::Object(instance) => {
            let class_name = instance.class_name.clone();

            if let Some(method) = vm.find_method_in_chain(&class_name, &method_name) {
                for (i, arg) in args.iter().enumerate() {
                    if i < method.param_types.len() {
                        if let Some(ref type_hint) = method.param_types[i] {
                            if vm.requires_strict_type_check(type_hint)
                                && !vm.value_matches_type(arg, type_hint)
                            {
                                let type_name = vm.format_type_hint(type_hint);
                                let given_type = vm.get_value_type_name(arg);
                                return Err(format!(
                                    "Argument {} passed to {}::{}() must be of type {}, {} given",
                                    i + 1,
                                    class_name,
                                    method_name,
                                    type_name,
                                    given_type
                                ));
                            }
                        }
                    }
                }

                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(method, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.this_source = ThisSource::GlobalVar(var_name.clone());

                for (i, arg) in args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        frame.locals[i + 1] = arg;
                    }
                }

                vm.frames.push(frame);
            } else if let Some(magic_call) = vm.find_method_in_chain(&class_name, "__call") {
                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(magic_call, stack_base);
                frame.locals[0] = Value::Object(instance);
                frame.this_source = ThisSource::GlobalVar(var_name.clone());
                frame.locals[1] = Value::String(method_name);
                let args_array: Vec<(ArrayKey, Value)> = args
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                    .collect();
                frame.locals[2] = Value::Array(args_array);

                vm.frames.push(frame);
            } else {
                return Err(format!(
                    "Method '{}' not found on class '{}'",
                    method_name, class_name
                ));
            }
        }
        Value::Generator(_) => {
            let mut gen = if let Value::Generator(g) = object {
                g
            } else {
                unreachable!()
            };
            match method_name.as_str() {
                "current" => {
                    let current = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(current);
                }
                "key" => {
                    let key = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .0
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(key);
                }
                "next" => {
                    gen.current_index += 1;
                    vm.stack
                        .push(Value::Bool(gen.current_index < gen.yielded_values.len()));
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                "rewind" => {
                    gen.current_index = 0;
                    gen.is_rewound = true;
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                "valid" => {
                    let valid = gen.current_index < gen.yielded_values.len() && !gen.finished;
                    vm.stack.push(Value::Bool(valid));
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                "getReturn" => {
                    let ret = gen.return_value.clone().unwrap_or(Value::Null);
                    vm.stack.push(ret);
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                "send" => {
                    let _sent = args.first().cloned().unwrap_or(Value::Null);
                    gen.current_index += 1;
                    let result = if gen.current_index < gen.yielded_values.len() {
                        gen.yielded_values[gen.current_index]
                            .1
                            .clone()
                            .unwrap_or(Value::Null)
                    } else {
                        Value::Null
                    };
                    vm.stack.push(result);
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                "throw" => {
                    vm.stack.push(Value::Null);
                    vm.globals.insert(var_name.clone(), Value::Generator(gen));
                }
                _ => {
                    return Err(format!("Method '{}' not found on Generator", method_name));
                }
            }
        }
        _ => return Err("Cannot call method on non-object".to_string()),
    }

    Ok(())
}

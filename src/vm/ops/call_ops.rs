use crate::runtime::{ArrayKey, Value};
use crate::vm::frame::CallFrame;

pub fn execute_call<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name: String,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    if let Some(func) = vm.get_function(&func_name) {
        if func.is_generator {
            let mut args_vec = Vec::with_capacity(arg_count as usize);
            for _ in 0..arg_count {
                args_vec.push(vm.stack.pop().ok_or("Stack underflow")?);
            }
            args_vec.reverse();
            return execute_generator_call(vm, func_name, args_vec);
        }

        if arg_count < func.required_param_count {
            return Err(format!(
                "Too few arguments to function {}(), {} passed in, at least {} expected",
                func.name, arg_count, func.required_param_count
            ));
        }

        for (i, arg) in args.iter().enumerate() {
            if i < func.param_types.len() {
                if let Some(ref type_hint) = func.param_types[i] {
                    let use_strict = func.strict_types || vm.requires_strict_type_check(type_hint);
                    if use_strict {
                        if !vm.value_matches_type_strict(arg, type_hint) {
                            let type_name = vm.format_type_hint(type_hint);
                            let given_type = vm.get_value_type_name(arg);
                            return Err(format!(
                                "Argument {} passed to {}() must be of type {}, {} given",
                                i + 1,
                                func.name,
                                type_name,
                                given_type
                            ));
                        }
                    } else if !vm.value_matches_type(arg, type_hint) {
                        let type_name = vm.format_type_hint(type_hint);
                        let given_type = vm.get_value_type_name(arg);
                        return Err(format!(
                            "Argument {} passed to {}() must be of type {}, {} given",
                            i + 1,
                            func.name,
                            type_name,
                            given_type
                        ));
                    }
                }
            }
        }

        let stack_base = vm.stack.len();
        let mut frame = CallFrame::new(func.clone(), stack_base);

        if func.is_variadic && func.param_count > 0 {
            let variadic_slot = (func.param_count - 1) as usize;
            for i in 0..variadic_slot {
                if i < args.len() {
                    let coerced_arg = if i < func.param_types.len() {
                        if let Some(ref type_hint) = func.param_types[i] {
                            let use_strict =
                                func.strict_types || vm.requires_strict_type_check(type_hint);
                            if !use_strict {
                                let coerced = vm.coerce_value_to_type(args[i].clone(), type_hint);
                                if !vm.value_matches_type(&coerced, type_hint) {
                                    let type_name = vm.format_type_hint(type_hint);
                                    let given_type = vm.get_value_type_name(&args[i]);
                                    return Err(format!(
                                        "must be of type {}, {} given",
                                        type_name, given_type
                                    ));
                                }
                                coerced
                            } else {
                                args[i].clone()
                            }
                        } else {
                            args[i].clone()
                        }
                    } else {
                        args[i].clone()
                    };
                    frame.locals[i] = coerced_arg;
                }
            }
            let variadic_args: Vec<(ArrayKey, Value)> = args
                .into_iter()
                .skip(variadic_slot)
                .enumerate()
                .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
                .collect();
            frame.locals[variadic_slot] = Value::Array(variadic_args);
        } else {
            for (i, arg) in args.into_iter().enumerate() {
                if i < frame.locals.len() {
                    let coerced_arg = if i < func.param_types.len() {
                        if let Some(ref type_hint) = func.param_types[i] {
                            let use_strict =
                                func.strict_types || vm.requires_strict_type_check(type_hint);
                            if !use_strict {
                                let coerced = vm.coerce_value_to_type(arg.clone(), type_hint);
                                if !vm.value_matches_type(&coerced, type_hint) {
                                    let type_name = vm.format_type_hint(type_hint);
                                    let given_type = vm.get_value_type_name(&arg);
                                    return Err(format!(
                                        "must be of type {}, {} given",
                                        type_name, given_type
                                    ));
                                }
                                coerced
                            } else {
                                arg
                            }
                        } else {
                            arg
                        }
                    } else {
                        arg
                    };
                    frame.locals[i] = coerced_arg;
                }
            }
        }

        vm.frames.push(frame);
    } else if super::super::builtins::is_builtin(&func_name) {
        let result = vm.call_reflection_or_builtin(&func_name, &args)?;
        vm.stack.push(result);
    } else {
        return Err(format!("undefined function: {}", func_name));
    }
    Ok(())
}

fn execute_generator_call<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name: String,
    args: Vec<Value>,
) -> Result<(), String> {
    use crate::runtime::YIELD_COLLECTOR;

    if let Some(func) = vm.get_function(&func_name) {
        let func = func.clone();

        YIELD_COLLECTOR.with(|collector| {
            let saved_yields = collector.borrow().yielded_values.clone();
            let saved_return = collector.borrow().return_value.clone();
            collector.borrow_mut().clear();

            let stack_base = vm.stack.len();
            let mut frame = CallFrame::new(func.clone(), stack_base);

            for (i, arg) in args.into_iter().enumerate() {
                if i < frame.locals.len() {
                    frame.locals[i] = arg;
                }
            }

            vm.frames.push(frame);

            while let Some(current_frame) = vm.frames.last_mut() {
                if current_frame.ip >= current_frame.function.bytecode.len() {
                    vm.frames.pop();
                    break;
                }

                let opcode = current_frame.function.bytecode[current_frame.ip].clone();
                current_frame.ip += 1;

                match vm.execute_opcode(opcode) {
                    Ok(()) => {}
                    Err(e) => {
                        if e == "__GENERATOR__" {
                            continue;
                        }
                        if e.starts_with("__RETURN__") {
                            let return_value = if e == "__RETURN__null" {
                                Value::Null
                            } else {
                                vm.stack.pop().unwrap_or(Value::Null)
                            };
                            collector.borrow_mut().return_value = Some(return_value);
                            vm.frames.pop();
                            break;
                        }
                        vm.frames.pop();
                        break;
                    }
                }
            }

            let yielded_values = collector.borrow().yielded_values.clone();
            let return_value = collector.borrow().return_value.clone();

            collector.borrow_mut().yielded_values = saved_yields;
            collector.borrow_mut().return_value = saved_return;

            let gen = crate::runtime::GeneratorInstance {
                id: 0,
                function_name: func_name.clone(),
                yielded_values,
                current_index: 0,
                is_rewound: false,
                finished: false,
                return_value,
                sent_value: None,
            };
            vm.stack
                .push(crate::runtime::Value::Generator(Box::new(gen)));
            Ok(())
        })
    } else {
        Err(format!("undefined function: {}", func_name))
    }
}

pub fn execute_call_builtin<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name: String,
    arg_count: u8,
) -> Result<(), String> {
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    let result = vm.call_reflection_or_builtin(&func_name, &args)?;
    vm.stack.push(result);
    Ok(())
}

pub fn execute_call_builtin_spread<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name_idx: u32,
) -> Result<(), String> {
    let func_name = vm.current_frame().get_string(func_name_idx).to_string();

    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let args = match args_array {
        Value::Array(arr) => arr.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
        _ => return Err("CallBuiltinSpread expects an array of arguments".to_string()),
    };

    let result = vm.call_reflection_or_builtin(&func_name, &args)?;
    vm.stack.push(result);
    Ok(())
}

pub fn execute_call_builtin_named<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name_idx: u32,
) -> Result<(), String> {
    let func_name = vm.current_frame().get_string(func_name_idx).to_string();

    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let named_args = match args_array {
        Value::Array(arr) => arr
            .into_iter()
            .filter_map(|(k, v)| {
                if let ArrayKey::String(name) = k {
                    Some((name, v))
                } else {
                    None
                }
            })
            .collect::<Vec<(String, Value)>>(),
        _ => return Err("CallBuiltinNamed expects an associative array of arguments".to_string()),
    };

    let args: Vec<Value> = named_args.into_iter().map(|(_, v)| v).collect();

    let result = vm.call_reflection_or_builtin(&func_name, &args)?;
    vm.stack.push(result);
    Ok(())
}

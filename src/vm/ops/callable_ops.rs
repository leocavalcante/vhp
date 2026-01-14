use crate::runtime::{ArrayKey, Closure, ClosureBody, Value};
use crate::vm::frame::CallFrame;

pub fn execute_call_callable<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    arg_count: u8,
) -> Result<(), String> {
    let callable = vm.stack.pop().ok_or("Stack underflow")?;

    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        args.push(vm.stack.pop().ok_or("Stack underflow")?);
    }
    args.reverse();

    match callable {
        Value::String(func_name) => {
            if let Some(func) = vm.get_function(&func_name) {
                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(func.clone(), stack_base);

                if func.is_variadic && func.param_count > 0 {
                    let variadic_slot = (func.param_count - 1) as usize;
                    for i in 0..variadic_slot {
                        if i < args.len() {
                            frame.locals[i] = args[i].clone();
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
                            frame.locals[i] = arg;
                        }
                    }
                }

                vm.frames.push(frame);
            } else if super::super::builtins::is_builtin(&func_name) {
                let result =
                    super::super::builtins::call_builtin(&func_name, &args, &mut vm.output)?;
                vm.stack.push(result);
            } else {
                return Err(format!("undefined function: {}", func_name));
            }
        }
        Value::Closure(closure) => match &closure.body {
            ClosureBody::FunctionRef(func_name) => {
                if let Some(func) = vm.get_function(func_name) {
                    let stack_base = vm.stack.len();
                    let mut frame = CallFrame::new(func, stack_base);

                    let mut next_slot = 0;
                    for (var_name, value) in &closure.captured_vars {
                        if let Some(slot) = frame
                            .function
                            .local_names
                            .iter()
                            .position(|n| n == var_name)
                        {
                            frame.locals[slot] = value.clone();
                            next_slot = next_slot.max(slot + 1);
                        }
                    }

                    for (i, arg) in args.into_iter().enumerate() {
                        if i + next_slot < frame.locals.len() {
                            frame.locals[i + next_slot] = arg;
                        }
                    }

                    vm.frames.push(frame);
                } else if super::super::builtins::is_builtin(func_name) {
                    let result =
                        super::super::builtins::call_builtin(func_name, &args, &mut vm.output)?;
                    vm.stack.push(result);
                } else {
                    return Err(format!("undefined function: {}", func_name));
                }
            }
            ClosureBody::Expression(_body_expr) => {
                return Err(
                    "Arrow function expression evaluation not yet supported in VM".to_string(),
                );
            }
            _ => return Err("Unsupported closure type".to_string()),
        },
        Value::Object(instance) => {
            let class_name = instance.class_name.clone();
            if let Some(method) = vm.find_method_in_chain(&class_name, "__invoke") {
                let stack_base = vm.stack.len();
                let mut frame = CallFrame::new(method, stack_base);
                frame.locals[0] = Value::Object(instance.clone());
                for (i, arg) in args.into_iter().enumerate() {
                    if i + 1 < frame.locals.len() {
                        frame.locals[i + 1] = arg;
                    }
                }
                vm.frames.push(frame);
            } else {
                return Err(format!("Object of class {} is not callable", class_name));
            }
        }
        _ => return Err(format!("Value is not callable: {:?}", callable)),
    }
    Ok(())
}

pub fn execute_call_spread<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name_idx: u32,
) -> Result<(), String> {
    let func_name = vm.current_frame().get_string(func_name_idx).to_string();

    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let args = match args_array {
        Value::Array(arr) => arr.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
        _ => return Err("CallSpread expects an array of arguments".to_string()),
    };

    let arg_count = args.len();

    if let Some(func) = vm.get_function(&func_name) {
        if (arg_count as u8) < func.required_param_count {
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
                                vm.coerce_value_to_type(args[i].clone(), type_hint)
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
                                vm.coerce_value_to_type(arg.clone(), type_hint)
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

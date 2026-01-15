use crate::runtime::{ArrayKey, Value};
use crate::vm::frame::CallFrame;

pub fn execute_call_named_args<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    func_name_idx: u32,
) -> Result<(), String> {
    let func_name = vm.current_frame().get_string(func_name_idx).to_string();

    let args_array = vm.stack.pop().ok_or("Stack underflow")?;
    let (positional_args, named_args) = match args_array {
        Value::Array(arr) => {
            let mut positional = Vec::new();
            let mut named = std::collections::HashMap::new();

            for (k, v) in arr {
                match k {
                    ArrayKey::Integer(i) => {
                        positional.push((i as usize, v));
                    }
                    ArrayKey::String(name) => {
                        named.insert(name, v);
                    }
                }
            }

            positional.sort_by_key(|(i, _)| *i);
            let positional: Vec<Value> = positional.into_iter().map(|(_, v)| v).collect();

            (positional, named)
        }
        _ => return Err("CallNamed expects an array of arguments".to_string()),
    };

    if let Some(func) = vm.get_function(&func_name) {
        let mut args = Vec::with_capacity(func.param_count as usize);

        for i in 0..func.param_count as usize {
            if i < func.parameters.len() {
                let param_name = &func.parameters[i].name;

                if i < positional_args.len() {
                    args.push(positional_args[i].clone());
                } else if let Some(value) = named_args.get(param_name) {
                    args.push(value.clone());
                } else if func.parameters[i].default.is_some() {
                    args.push(Value::Null);
                } else if i < func.required_param_count as usize {
                    return Err(format!(
                        "Missing required argument '{}' for function {}()",
                        param_name, func.name
                    ));
                } else {
                    args.push(Value::Null);
                }
            }
        }

        for name in named_args.keys() {
            if !func.parameters.iter().any(|p| &p.name == name) {
                return Err(format!(
                    "Unknown named parameter '{}' for function {}()",
                    name, func.name
                ));
            }
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
        let mut args = positional_args;
        for (_, v) in named_args {
            args.push(v);
        }
        let result = vm.call_reflection_or_builtin(&func_name, &args)?;
        vm.stack.push(result);
    } else {
        return Err(format!("undefined function: {}", func_name));
    }
    Ok(())
}

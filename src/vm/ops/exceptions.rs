use crate::runtime::Value;
use crate::vm::frame::ExceptionHandler;

/// Capture backtrace information from the current call stack
pub fn capture_backtrace(frames: &[super::super::CallFrame]) -> Value {
    let mut trace_array: Vec<(crate::runtime::ArrayKey, Value)> = Vec::new();

    // Capture call stack in reverse order (most recent first)
    for (idx, frame) in frames.iter().rev().enumerate() {
        let func_name = frame.function.name.clone();
        let (class, type_sep, function) = if let Some(pos) = func_name.rfind("::") {
            let class_part = &func_name[..pos];
            let method_part = &func_name[pos + 2..];
            let type_sep = if frame.this.is_some() { "->" } else { "::" };
            (
                Some(class_part.to_string()),
                type_sep.to_string(),
                method_part.to_string(),
            )
        } else {
            (None, "".to_string(), func_name.clone())
        };

        // Create trace frame array with file, line, function, class, type, args
        let mut frame_array: Vec<(crate::runtime::ArrayKey, Value)> = Vec::new();

        // File - use placeholder for now (would need debug info to get actual file)
        frame_array.push((
            crate::runtime::ArrayKey::String("file".to_string()),
            Value::String("".to_string()),
        ));

        // Line - use placeholder for now
        frame_array.push((
            crate::runtime::ArrayKey::String("line".to_string()),
            Value::Integer(0),
        ));

        // Function name
        frame_array.push((
            crate::runtime::ArrayKey::String("function".to_string()),
            Value::String(function),
        ));

        // Class name (for method calls)
        if let Some(cls) = class {
            frame_array.push((
                crate::runtime::ArrayKey::String("class".to_string()),
                Value::String(cls),
            ));
        }

        // Type separator (-> for instance, :: for static)
        frame_array.push((
            crate::runtime::ArrayKey::String("type".to_string()),
            Value::String(type_sep),
        ));

        // Args - empty array for now (capturing actual args is more complex)
        frame_array.push((
            crate::runtime::ArrayKey::String("args".to_string()),
            Value::Array(Vec::new()),
        ));

        trace_array.push((
            crate::runtime::ArrayKey::Integer(idx as i64),
            Value::Array(frame_array),
        ));
    }

    Value::Array(trace_array)
}

pub fn execute_throw<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let mut exception = vm.stack.pop().ok_or("Stack underflow")?;

    // Capture backtrace before modifying frames
    let backtrace = capture_backtrace(&vm.frames);

    // Store backtrace in the exception object
    if let Value::Object(ref mut obj) = &mut exception {
        obj.properties.insert("trace".to_string(), backtrace);
    }

    let current_frame_depth = vm.frames.len();
    let current_ip = vm.current_frame().ip;

    let mut handler_info: Option<(usize, usize, usize)> = None;

    for (handler_idx, handler) in vm.handlers.iter().enumerate().rev() {
        if handler.frame_depth > current_frame_depth {
            continue;
        }

        let handler_is_active = if handler.frame_depth == current_frame_depth {
            current_ip >= handler.try_start as usize
                && (handler.try_end == 0 || current_ip < handler.try_end as usize)
        } else {
            handler.try_end == 0 || handler.try_end > handler.try_start
        };

        if handler_is_active {
            handler_info = Some((
                handler.catch_offset as usize,
                handler.frame_depth,
                handler_idx,
            ));
            break;
        }
    }

    if let Some((catch_offset, target_frame_depth, handler_idx)) = handler_info {
        while vm.frames.len() > target_frame_depth {
            vm.frames.pop();
        }

        if let Some(handler) = vm.handlers.get_mut(handler_idx) {
            if handler.try_end == 0 {
                handler.try_end = current_ip as u32;
            }
        }

        vm.stack.push(exception);
        if let Some(frame) = vm.frames.last_mut() {
            frame.jump_to(catch_offset);
        }
    } else {
        let (error_msg, trace_output) = if let Value::Object(ref obj) = exception {
            let message = if let Some(msg_value) = obj.properties.get("message") {
                match msg_value {
                    Value::String(s) if !s.is_empty() => s.clone(),
                    _ => String::new(),
                }
            } else {
                String::new()
            };

            let file = if let Some(file_value) = obj.properties.get("__file") {
                match file_value {
                    Value::String(s) if !s.is_empty() => s.clone(),
                    _ => "unknown".to_string(),
                }
            } else {
                "unknown".to_string()
            };

            let line = if let Some(line_value) = obj.properties.get("__line") {
                match line_value {
                    Value::Integer(n) => *n,
                    _ => 0,
                }
            } else {
                0
            };

            let class_name = obj.class_name.clone();

            // Get the call stack - format each frame
            let mut trace_lines: Vec<String> = Vec::new();
            for (i, frame) in vm.frames.iter().enumerate() {
                let func_name = frame.function.name.clone();
                let loc = if let Some(pos) = func_name.rfind("::") {
                    let class_part = &func_name[..pos];
                    let method_part = &func_name[pos + 2..];
                    format!("{}->{}", class_part, method_part)
                } else {
                    func_name
                };
                trace_lines.push(format!("#{} [{}:{}] {}", i, file, line, loc));
            }

            let trace_output = trace_lines.join("\n");

            let base_msg = if !message.is_empty() {
                format!("{}: {} in {} on line {}", class_name, message, file, line)
            } else {
                format!("Uncaught {}", class_name)
            };

            (base_msg, Some(trace_output))
        } else {
            (format!("Uncaught exception: {:?}", exception), None)
        };

        if let Some(trace) = trace_output {
            return Err(format!("{}\n\nStack trace:\n{}", error_msg, trace));
        } else {
            return Err(error_msg);
        }
    }
    Ok(())
}

pub fn execute_try_start<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    catch_offset: u32,
    finally_offset: u32,
) {
    let try_start = vm.current_frame().ip as u32;
    let frame_depth = vm.frames.len();
    vm.handlers.push(ExceptionHandler {
        try_start,
        try_end: 0,
        catch_offset,
        catch_class: String::new(),
        catch_var: String::new(),
        finally_offset,
        stack_depth: vm.stack.len(),
        frame_depth,
    });
}

pub fn execute_try_end<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    let current_ip = vm.current_frame().ip as u32;
    if let Some(handler) = vm.handlers.last_mut() {
        handler.try_end = current_ip;
    }
}

pub fn execute_finally_start<W: std::io::Write>(_vm: &mut super::super::VM<W>) {}

pub fn execute_finally_end<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    if vm.pending_return.is_some() {
        return Err("__FINALLY_RETURN__".to_string());
    }
    Ok(())
}

use crate::runtime::Value;
use crate::vm::frame::ExceptionHandler;

pub fn execute_throw<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let exception = vm.stack.pop().ok_or("Stack underflow")?;

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
        let error_msg = if let Value::Object(ref obj) = exception {
            if let Some(msg_value) = obj.properties.get("message") {
                match msg_value {
                    Value::String(s) if !s.is_empty() => {
                        format!("Uncaught {}: {}", obj.class_name, s)
                    }
                    _ => format!("Uncaught {}", obj.class_name),
                }
            } else {
                format!("Uncaught {}", obj.class_name)
            }
        } else {
            format!("Uncaught exception: {:?}", exception)
        };
        return Err(error_msg);
    }
    Ok(())
}

pub fn execute_try_start<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    catch_offset: i32,
    finally_offset: i32,
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

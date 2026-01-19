//! VM Execution Loop
//!
//! This module contains the main execution loop and opcode dispatch logic.

use super::VM;
use crate::runtime::Value;
use crate::vm::frame::{CallFrame, ThisSource};
use crate::vm::opcode::CompiledFunction;
use std::io::Write;
use std::sync::Arc;

pub fn execute_vm<W: Write>(
    vm: &mut VM<W>,
    function: Arc<CompiledFunction>,
) -> Result<Value, String> {
    let frame = CallFrame::new(function, 0);
    vm.frames.push(frame);

    loop {
        let frame = match vm.frames.last_mut() {
            Some(f) => f,
            None => {
                return Ok(vm.stack.pop().unwrap_or(Value::Null));
            }
        };

        if frame.ip >= frame.function.bytecode.len() {
            let returned = vm.stack.pop().unwrap_or(Value::Null);
            vm.frames.pop();

            if vm.frames.is_empty() {
                return Ok(returned);
            }

            vm.stack.push(returned);
            continue;
        }

        let opcode = frame.function.bytecode[frame.ip].clone();
        frame.ip += 1;

        match vm.execute_opcode(opcode) {
            Ok(()) => {}
            Err(e) => {
                if e.starts_with("__RETURN__") {
                    let frame = vm.frames.last().expect("No frame");
                    let current_ip = frame.ip as u32;
                    let is_constructor = frame.is_constructor;
                    let this_source = frame.this_source.clone();
                    let modified_this =
                        if !matches!(this_source, ThisSource::None) && !frame.locals.is_empty() {
                            Some(frame.locals[0].clone())
                        } else {
                            None
                        };

                    let value =
                        if is_constructor || matches!(this_source, ThisSource::PropertySetHook) {
                            frame.locals[0].clone()
                        } else {
                            let value_str = e.strip_prefix("__RETURN__").unwrap();
                            if value_str == "null" {
                                Value::Null
                            } else {
                                vm.stack.pop().unwrap_or(Value::Null)
                            }
                        };

                    let finally_jump = vm.handlers.iter().rev().find_map(|h| {
                        if h.frame_depth == vm.frames.len()
                            && h.finally_offset > 0
                            && current_ip > h.try_start
                            && current_ip <= h.finally_offset
                        {
                            Some(h.finally_offset as usize)
                        } else {
                            None
                        }
                    });

                    if let Some(finally_offset) = finally_jump {
                        vm.pending_return = Some(value);
                        if let Some(frame) = vm.frames.last_mut() {
                            frame.jump_to(finally_offset);
                        }
                        continue;
                    }

                    vm.frames.pop();

                    if let Some(modified) = modified_this {
                        match this_source {
                            ThisSource::LocalSlot(slot) => {
                                if let Some(caller_frame) = vm.frames.last_mut() {
                                    caller_frame.set_local(slot, modified);
                                }
                            }
                            ThisSource::GlobalVar(var_name) => {
                                vm.globals.insert(var_name, modified);
                            }
                            ThisSource::None | ThisSource::PropertySetHook => {}
                        }
                    }

                    if vm.frames.is_empty() {
                        return Ok(value);
                    }
                    vm.stack.push(value);
                    continue;
                } else if e.starts_with("__BREAK__") {
                    if let Some(loop_ctx) = vm.loops.last() {
                        let break_target = loop_ctx.break_target as usize;
                        if let Some(frame) = vm.frames.last_mut() {
                            frame.jump_to(break_target);
                        }
                    } else {
                        return Err("Cannot break outside of loop".to_string());
                    }
                    continue;
                } else if e.starts_with("__CONTINUE__") {
                    if let Some(loop_ctx) = vm.loops.last() {
                        let continue_target = loop_ctx.continue_target as usize;
                        if let Some(frame) = vm.frames.last_mut() {
                            frame.jump_to(continue_target);
                        }
                    } else {
                        return Err("Cannot continue outside of loop".to_string());
                    }
                    continue;
                } else if e.starts_with("__EXCEPTION__") {
                    let parts: Vec<&str> = e.splitn(3, ':').collect();
                    if parts.len() >= 3 {
                        let _class = parts[1];
                        let _message = parts[2];
                        return Err(e);
                    }
                    return Err(e);
                } else if e == "__GENERATOR__" {
                    vm.frames.pop();
                    return Err("__GENERATOR__".to_string());
                } else if e == "__FINALLY_RETURN__" {
                    if let Some(value) = vm.pending_return.take() {
                        let frame = vm.frames.last().expect("No frame");
                        let this_source = frame.this_source.clone();
                        let modified_this = if !matches!(this_source, ThisSource::None)
                            && !frame.locals.is_empty()
                        {
                            Some(frame.locals[0].clone())
                        } else {
                            None
                        };

                        vm.frames.pop();

                        if let Some(modified) = modified_this {
                            match this_source {
                                ThisSource::LocalSlot(slot) => {
                                    if let Some(caller_frame) = vm.frames.last_mut() {
                                        caller_frame.set_local(slot, modified);
                                    }
                                }
                                ThisSource::GlobalVar(var_name) => {
                                    vm.globals.insert(var_name, modified);
                                }
                                ThisSource::None | ThisSource::PropertySetHook => {}
                            }
                        }

                        if vm.frames.is_empty() {
                            return Ok(value);
                        }
                        vm.stack.push(value);
                        continue;
                    }
                } else if e.starts_with("__EXIT__:") {
                    // Handle exit() and die() calls
                    return Err(e);
                } else {
                    return Err(e);
                }
            }
        }
    }
}

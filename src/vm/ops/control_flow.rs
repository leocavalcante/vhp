use crate::ast::TypeHint;
use crate::runtime::Value;

pub fn execute_jump<W: std::io::Write>(vm: &mut super::super::VM<W>, offset: u32) {
    vm.current_frame_mut().jump_to(offset as usize);
}

pub fn execute_jump_if_false<W: std::io::Write>(vm: &mut super::super::VM<W>, offset: u32) {
    let value = vm.stack.pop().unwrap();
    if !value.to_bool() {
        vm.current_frame_mut().jump_to(offset as usize);
    }
}

pub fn execute_jump_if_true<W: std::io::Write>(vm: &mut super::super::VM<W>, offset: u32) {
    let value = vm.stack.pop().unwrap();
    if value.to_bool() {
        vm.current_frame_mut().jump_to(offset as usize);
    }
}

pub fn execute_jump_if_null<W: std::io::Write>(vm: &mut super::super::VM<W>, offset: u32) {
    let value = vm.stack.last().unwrap();
    if matches!(value, Value::Null) {
        vm.current_frame_mut().jump_to(offset as usize);
    }
}

pub fn execute_jump_if_not_null<W: std::io::Write>(vm: &mut super::super::VM<W>, offset: u32) {
    let value = vm.stack.last().unwrap();
    if !matches!(value, Value::Null) {
        vm.current_frame_mut().jump_to(offset as usize);
    }
}

pub fn execute_return<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    if let Some(ref return_type) = vm.current_frame().function.return_type.clone() {
        if matches!(return_type, TypeHint::Void) {
            return Err(format!(
                "{}(): Return value must be of type void",
                vm.current_frame().function.name
            ));
        }
        let return_value = vm.stack.last().cloned().unwrap_or(Value::Null);
        if !vm.value_matches_type_strict(&return_value, return_type) {
            let type_name = vm.format_type_hint(return_type);
            let given_type = vm.get_value_type_name(&return_value);
            return Err(format!(
                "Return value must be of type {}, {} returned",
                type_name, given_type
            ));
        }
    }
    Err("__RETURN__".to_string())
}

pub fn execute_yield<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().unwrap_or(Value::Null);
    let gen = crate::runtime::GeneratorInstance {
        id: 0,
        position: 0,
        values: vec![value.clone()],
        statements: vec![],
        current_statement: 0,
        variables: std::collections::HashMap::new(),
        finished: false,
    };
    vm.stack.push(Value::Generator(Box::new(gen)));
    Err("__GENERATOR__".to_string())
}

pub fn execute_yield_from<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    let value = vm.stack.pop().unwrap_or(Value::Null);
    let gen = crate::runtime::GeneratorInstance {
        id: 0,
        position: 0,
        values: vec![value.clone()],
        statements: vec![],
        current_statement: 0,
        variables: std::collections::HashMap::new(),
        finished: false,
    };
    vm.stack.push(Value::Generator(Box::new(gen)));
    Err("__GENERATOR__".to_string())
}

pub fn execute_return_null<W: std::io::Write>(vm: &mut super::super::VM<W>) -> Result<(), String> {
    if let Some(ref return_type) = vm.current_frame().function.return_type.clone() {
        if !matches!(return_type, TypeHint::Void)
            && !vm.value_matches_type_strict(&Value::Null, return_type)
        {
            let type_name = vm.format_type_hint(return_type);
            return Err(format!(
                "Return value must be of type {}, null returned",
                type_name
            ));
        }
    }
    Err("__RETURN__null".to_string())
}

pub fn execute_break<W: std::io::Write>(_vm: &mut super::super::VM<W>) -> Result<(), String> {
    Err("__BREAK__".to_string())
}

pub fn execute_continue<W: std::io::Write>(_vm: &mut super::super::VM<W>) -> Result<(), String> {
    Err("__CONTINUE__".to_string())
}

pub fn execute_loop_start<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
    continue_target: u32,
    break_target: u32,
) {
    vm.loops.push(super::super::frame::LoopContext {
        continue_target,
        break_target,
        stack_depth: vm.stack.len(),
    });
}

pub fn execute_loop_end<W: std::io::Write>(vm: &mut super::super::VM<W>) {
    vm.loops.pop();
}

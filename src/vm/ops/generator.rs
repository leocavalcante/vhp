use crate::runtime::Value;

pub fn execute_generator_current<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let gen_value = vm.stack.pop().ok_or("Stack underflow")?;
    match gen_value {
        Value::Generator(gen) => {
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
            Ok(())
        }
        _ => Err("Generator::current() requires a Generator object".to_string()),
    }
}

pub fn execute_generator_key<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let gen_value = vm.stack.pop().ok_or("Stack underflow")?;
    match gen_value {
        Value::Generator(gen) => {
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
            Ok(())
        }
        _ => Err("Generator::key() requires a Generator object".to_string()),
    }
}

pub fn execute_generator_next<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let gen_value = vm.stack.pop().ok_or("Stack underflow")?;
    match gen_value {
        Value::Generator(mut gen) => {
            gen.current_index += 1;
            let result = Value::Bool(gen.current_index < gen.yielded_values.len());
            vm.stack.push(Value::Generator(gen));
            vm.stack.push(result);
            Ok(())
        }
        _ => Err("Generator::next() requires a Generator object".to_string()),
    }
}

pub fn execute_generator_rewind<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let gen_value = vm.stack.pop().ok_or("Stack underflow")?;
    match gen_value {
        Value::Generator(mut gen) => {
            gen.current_index = 0;
            gen.is_rewound = true;
            vm.stack.push(Value::Generator(gen));
            Ok(())
        }
        _ => Err("Generator::rewind() requires a Generator object".to_string()),
    }
}

pub fn execute_generator_valid<W: std::io::Write>(
    vm: &mut super::super::VM<W>,
) -> Result<(), String> {
    let gen_value = vm.stack.pop().ok_or("Stack underflow")?;
    match gen_value {
        Value::Generator(gen) => {
            let valid = gen.current_index < gen.yielded_values.len() && !gen.finished;
            vm.stack.push(Value::Generator(gen));
            vm.stack.push(Value::Bool(valid));
            Ok(())
        }
        _ => Err("Generator::valid() requires a Generator object".to_string()),
    }
}

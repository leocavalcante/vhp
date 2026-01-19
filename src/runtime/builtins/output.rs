//! Output built-in functions

use crate::runtime::{ArrayKey, Value};
use std::io::Write;

/// print - Output a string
pub fn print<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("print() expects exactly 1 parameter".to_string());
    }
    write!(output, "{}", args[0].to_output_string()).map_err(|e| e.to_string())?;
    Ok(Value::Integer(1))
}

/// var_dump - Dumps information about a variable
pub fn var_dump<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    for arg in args {
        var_dump_value(output, arg, 0)?;
    }
    Ok(Value::Null)
}

fn var_dump_value<W: Write>(output: &mut W, value: &Value, indent: usize) -> Result<(), String> {
    let prefix = "  ".repeat(indent);
    match value {
        Value::Null => {
            writeln!(output, "{}NULL", prefix).map_err(|e| e.to_string())?;
        }
        Value::Bool(b) => {
            writeln!(output, "{}bool({})", prefix, b).map_err(|e| e.to_string())?;
        }
        Value::Integer(n) => {
            writeln!(output, "{}int({})", prefix, n).map_err(|e| e.to_string())?;
        }
        Value::Float(n) => {
            writeln!(output, "{}float({})", prefix, n).map_err(|e| e.to_string())?;
        }
        Value::String(s) => {
            writeln!(output, "{}string({}) \"{}\"", prefix, s.len(), s)
                .map_err(|e| e.to_string())?;
        }
        Value::Array(arr) => {
            writeln!(output, "{}array({}) {{", prefix, arr.len()).map_err(|e| e.to_string())?;
            for (key, val) in arr {
                match key {
                    ArrayKey::Integer(n) => {
                        writeln!(output, "{}  [{}]=>", prefix, n).map_err(|e| e.to_string())?;
                    }
                    ArrayKey::String(s) => {
                        writeln!(output, "{}  [\"{}\"]=>", prefix, s).map_err(|e| e.to_string())?;
                    }
                }
                var_dump_value(output, val, indent + 1)?;
            }
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
        Value::Object(obj) => {
            writeln!(
                output,
                "{}object({})#1 ({}) {{",
                prefix,
                obj.class_name,
                obj.properties.len()
            )
            .map_err(|e| e.to_string())?;
            for (key, val) in &obj.properties {
                writeln!(output, "{}  [\"{}\"]=>", prefix, key).map_err(|e| e.to_string())?;
                var_dump_value(output, val, indent + 1)?;
            }
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
        Value::Fiber(fiber) => {
            writeln!(output, "{}object(Fiber#{:06}) {{", prefix, fiber.id)
                .map_err(|e| e.to_string())?;
            writeln!(output, "{}  state: {:?}", prefix, fiber.state).map_err(|e| e.to_string())?;
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
        Value::Closure(_) => {
            writeln!(output, "{}object(Closure)#1 {{", prefix).map_err(|e| e.to_string())?;
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
        Value::Generator(gen) => {
            writeln!(output, "{}object(Generator)#{:06} {{", prefix, gen.id)
                .map_err(|e| e.to_string())?;
            writeln!(
                output,
                "{}  yielded_values: {}",
                prefix,
                gen.yielded_values.len()
            )
            .map_err(|e| e.to_string())?;
            writeln!(output, "{}  current_index: {}", prefix, gen.current_index)
                .map_err(|e| e.to_string())?;
            writeln!(output, "{}  finished: {}", prefix, gen.finished)
                .map_err(|e| e.to_string())?;
            if let Some(ref ret) = gen.return_value {
                writeln!(output, "{}  return_value: {:?}", prefix, ret)
                    .map_err(|e| e.to_string())?;
            }
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
        Value::EnumCase {
            enum_name,
            case_name,
            backing_value,
        } => {
            if let Some(val) = backing_value {
                writeln!(output, "{}enum({}::{}): ", prefix, enum_name, case_name)
                    .map_err(|e| e.to_string())?;
                var_dump_value(output, val, indent)?;
            } else {
                writeln!(output, "{}enum({}::{})", prefix, enum_name, case_name)
                    .map_err(|e| e.to_string())?;
            }
        }
        Value::Exception(exc) => {
            writeln!(output, "{}object({})#1 (2) {{", prefix, exc.class_name)
                .map_err(|e| e.to_string())?;
            writeln!(
                output,
                "{}  [\"message\"]=>\n{}  string({}) \"{}\"",
                prefix,
                prefix,
                exc.message.len(),
                exc.message
            )
            .map_err(|e| e.to_string())?;
            writeln!(
                output,
                "{}  [\"code\"]=>\n{}  int({})",
                prefix, prefix, exc.code
            )
            .map_err(|e| e.to_string())?;
            writeln!(output, "{}}}", prefix).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// print_r - Prints human-readable information about a variable
pub fn print_r<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("print_r() expects at least 1 parameter".to_string());
    }
    let return_output = args.len() >= 2 && args[1].to_bool();

    let out = print_r_value(&args[0], 0);

    if return_output {
        Ok(Value::String(out))
    } else {
        write!(output, "{}", out).map_err(|e| e.to_string())?;
        Ok(Value::Bool(true))
    }
}

fn print_r_value(value: &Value, indent: usize) -> String {
    let prefix = "    ".repeat(indent);
    match value {
        Value::Array(arr) => {
            let mut result = String::from("Array\n");
            result.push_str(&format!("{}(\n", prefix));
            for (key, val) in arr {
                let key_str = match key {
                    ArrayKey::Integer(n) => n.to_string(),
                    ArrayKey::String(s) => s.clone(),
                };
                let val_str = print_r_value(val, indent + 1);
                result.push_str(&format!(
                    "{}    [{}] => {}\n",
                    prefix,
                    key_str,
                    val_str.trim_start()
                ));
            }
            result.push_str(&format!("{})\n", prefix));
            result
        }
        Value::Object(obj) => {
            let mut result = format!("{} Object\n", obj.class_name);
            result.push_str(&format!("{}(\n", prefix));
            for (key, val) in &obj.properties {
                let val_str = print_r_value(val, indent + 1);
                result.push_str(&format!(
                    "{}    [{}] => {}\n",
                    prefix,
                    key,
                    val_str.trim_start()
                ));
            }
            result.push_str(&format!("{})\n", prefix));
            result
        }
        _ => value.to_string_val(),
    }
}

/// printf - Output a formatted string
pub fn printf<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    let result = super::string::sprintf(args)?;
    write!(output, "{}", result.to_string_val()).map_err(|e| e.to_string())?;
    Ok(Value::Integer(result.to_string_val().len() as i64))
}

/// exit - Terminate script execution
///
/// Behavior:
/// - No arguments: terminate with exit code 0
/// - String argument: output the string and terminate with exit code 0
/// - Integer argument: terminate with that exit code
pub fn exit<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    let exit_code = match args.len() {
        0 => 0,
        1 => {
            match &args[0] {
                Value::String(s) => {
                    // Output the string and exit with code 0
                    write!(output, "{}", s).map_err(|e| e.to_string())?;
                    0
                }
                Value::Integer(n) => {
                    // Use the integer as exit code (clamp to 0-255 range like PHP)
                    let mut code = *n;
                    if code < 0 {
                        code = 0;
                    } else if code > 255 {
                        code = 255;
                    }
                    code
                }
                Value::Float(f) => {
                    // Convert to int and clamp
                    let mut code = *f as i64;
                    if code < 0 {
                        code = 0;
                    } else if code > 255 {
                        code = 255;
                    }
                    code
                }
                Value::Bool(b) => {
                    // true -> 1, false -> 0
                    if *b {
                        1
                    } else {
                        0
                    }
                }
                _ => {
                    // Other types: convert to string and output, exit with code 0
                    write!(output, "{}", args[0].to_output_string()).map_err(|e| e.to_string())?;
                    0
                }
            }
        }
        _ => {
            return Err("exit() expects at most 1 parameter".to_string());
        }
    };

    // Return special error to signal script termination
    Err(format!("__EXIT__:{}", exit_code))
}

/// die - Alias for exit()
pub fn die<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    exit(output, args)
}

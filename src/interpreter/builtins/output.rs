//! Output built-in functions

use crate::interpreter::value::Value;
use std::io::Write;

/// print - Output a string
pub fn print<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("print() expects exactly 1 parameter".to_string());
    }
    write!(output, "{}", args[0].to_output_string())
        .map_err(|e| e.to_string())?;
    Ok(Value::Integer(1))
}

/// var_dump - Dumps information about a variable
pub fn var_dump<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    for arg in args {
        let dump = match arg {
            Value::Null => "NULL\n".to_string(),
            Value::Bool(b) => format!("bool({})\n", b),
            Value::Integer(n) => format!("int({})\n", n),
            Value::Float(n) => format!("float({})\n", n),
            Value::String(s) => format!("string({}) \"{}\"\n", s.len(), s),
        };
        write!(output, "{}", dump).map_err(|e| e.to_string())?;
    }
    Ok(Value::Null)
}

/// print_r - Prints human-readable information about a variable
pub fn print_r<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("print_r() expects at least 1 parameter".to_string());
    }
    let return_output = args.len() >= 2 && args[1].to_bool();
    let out = args[0].to_string_val();

    if return_output {
        Ok(Value::String(out))
    } else {
        write!(output, "{}", out).map_err(|e| e.to_string())?;
        Ok(Value::Bool(true))
    }
}

/// printf - Output a formatted string
pub fn printf<W: Write>(output: &mut W, args: &[Value]) -> Result<Value, String> {
    let result = super::string::sprintf(args)?;
    write!(output, "{}", result.to_string_val())
        .map_err(|e| e.to_string())?;
    Ok(Value::Integer(result.to_string_val().len() as i64))
}

//! Type built-in functions

use crate::runtime::Value;

/// intval - Get the integer value of a variable
pub fn intval(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("intval() expects at least 1 parameter".to_string());
    }
    Ok(Value::Integer(args[0].to_int()))
}

/// floatval - Get the float value of a variable
pub fn floatval(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("floatval() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float()))
}

/// strval - Get the string value of a variable
pub fn strval(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strval() expects exactly 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val()))
}

/// boolval - Get the boolean value of a variable
pub fn boolval(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("boolval() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(args[0].to_bool()))
}

/// gettype - Get the type of a variable
pub fn gettype(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("gettype() expects exactly 1 parameter".to_string());
    }
    Ok(Value::String(args[0].get_type().to_string()))
}

/// is_null - Finds whether a variable is null
pub fn is_null(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_null() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Null)))
}

/// is_bool - Finds out whether a variable is a boolean
pub fn is_bool(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_bool() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

/// is_int - Finds whether the type of a variable is integer
pub fn is_int(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_int() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Integer(_))))
}

/// is_float - Finds whether the type of a variable is float
pub fn is_float(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_float() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::Float(_))))
}

/// is_string - Find whether the type of a variable is string
pub fn is_string(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_string() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}

/// is_array - Finds whether a variable is an array
pub fn is_array(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_array() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(args[0].is_array()))
}

/// is_numeric - Finds whether a variable is a number or numeric string
pub fn is_numeric(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_numeric() expects exactly 1 parameter".to_string());
    }
    let is_numeric = match &args[0] {
        Value::Integer(_) | Value::Float(_) => true,
        Value::String(s) => s.parse::<f64>().is_ok(),
        _ => false,
    };
    Ok(Value::Bool(is_numeric))
}

/// isset - Determine if a variable is declared and is different than null
pub fn isset(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("isset() expects at least 1 parameter".to_string());
    }
    Ok(Value::Bool(!matches!(
        args.first(),
        Some(Value::Null) | None
    )))
}

/// empty - Determine whether a variable is empty
pub fn empty(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("empty() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(
        !args.first().map(|v| v.to_bool()).unwrap_or(false),
    ))
}

/// unset - Unset a given variable
/// Note: This is a simplified implementation. In PHP, unset() is a language construct
/// that removes variables from the symbol table. For now, this just returns null.
/// The actual removal should be handled at the VM level.
pub fn unset(_args: &[Value]) -> Result<Value, String> {
    // unset() doesn't return a value in PHP, but for VM compatibility we return Null
    Ok(Value::Null)
}

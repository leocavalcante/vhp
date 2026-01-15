//! Additional Math built-in functions

#![allow(clippy::manual_range_contains)]

use crate::runtime::Value;

/// deg2rad - Converts degrees to radians
pub fn deg2rad(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("deg2rad() expects exactly 1 parameter".to_string());
    }
    let degrees = args[0].to_float();
    Ok(Value::Float(degrees * std::f64::consts::PI / 180.0))
}

/// rad2deg - Converts radians to degrees
pub fn rad2deg(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("rad2deg() expects exactly 1 parameter".to_string());
    }
    let radians = args[0].to_float();
    Ok(Value::Float(radians * 180.0 / std::f64::consts::PI))
}

/// asin - Arc sine of a number
pub fn asin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("asin() expects exactly 1 parameter".to_string());
    }
    let val = args[0].to_float();
    if val < -1.0 || val > 1.0 {
        return Err("asin(): Argument must be in the range [-1, 1]".to_string());
    }
    Ok(Value::Float(val.asin()))
}

/// acos - Arc cosine of a number
pub fn acos(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("acos() expects exactly 1 parameter".to_string());
    }
    let val = args[0].to_float();
    if val < -1.0 || val > 1.0 {
        return Err("acos(): Argument must be in the range [-1, 1]".to_string());
    }
    Ok(Value::Float(val.acos()))
}

/// atan - Arc tangent of a number
pub fn atan(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("atan() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().atan()))
}

/// atan2 - Arc tangent of two numbers
pub fn atan2(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("atan2() expects exactly 2 parameters".to_string());
    }
    let y = args[0].to_float();
    let x = args[1].to_float();
    Ok(Value::Float(y.atan2(x)))
}

/// sinh - Hyperbolic sine
pub fn sinh(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sinh() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().sinh()))
}

/// cosh - Hyperbolic cosine
pub fn cosh(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("cosh() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().cosh()))
}

/// tanh - Hyperbolic tangent
pub fn tanh(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("tanh() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().tanh()))
}

/// hypot - Calculate the length of the hypotenuse
pub fn hypot(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("hypot() expects exactly 2 parameters".to_string());
    }
    let x = args[0].to_float();
    let y = args[1].to_float();
    Ok(Value::Float((x * x + y * y).sqrt()))
}

/// fmod - Returns the floating point remainder of a division
pub fn fmod(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("fmod() expects exactly 2 parameters".to_string());
    }
    let x = args[0].to_float();
    let y = args[1].to_float();
    if y == 0.0 {
        return Err("fmod(): Division by zero".to_string());
    }
    Ok(Value::Float(x - (x / y).floor() * y))
}

/// intdiv - Integer division
pub fn intdiv(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("intdiv() expects exactly 2 parameters".to_string());
    }
    let dividend = args[0].to_int();
    let divisor = args[1].to_int();
    if divisor == 0 {
        return Err("intdiv(): Division by zero".to_string());
    }
    Ok(Value::Integer(dividend / divisor))
}

/// is_finite - Checks if a float is finite
pub fn is_finite(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_finite() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(args[0].to_float().is_finite()))
}

/// is_infinite - Checks if a float is infinite
pub fn is_infinite(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_infinite() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(args[0].to_float().is_infinite()))
}

/// is_nan - Checks if a float is not a number
pub fn is_nan(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_nan() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Bool(args[0].to_float().is_nan()))
}

/// bindec - Binary to decimal
pub fn bindec(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("bindec() expects exactly 1 parameter".to_string());
    }
    let binary_str = args[0].to_string_val();
    match i64::from_str_radix(&binary_str, 2) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(_) => Ok(Value::Integer(0)),
    }
}

/// decbin - Decimal to binary
pub fn decbin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("decbin() expects exactly 1 parameter".to_string());
    }
    let n = args[0].to_int();
    Ok(Value::String(format!("{:b}", n)))
}

/// decoct - Decimal to octal
pub fn decoct(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("decoct() expects exactly 1 parameter".to_string());
    }
    let n = args[0].to_int();
    Ok(Value::String(format!("{:o}", n)))
}

/// dechex - Decimal to hexadecimal
pub fn dechex(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("dechex() expects exactly 1 parameter".to_string());
    }
    let n = args[0].to_int();
    Ok(Value::String(format!("{:x}", n)))
}

/// hexdec - Hexadecimal to decimal
pub fn hexdec(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("hexdec() expects exactly 1 parameter".to_string());
    }
    let hex_str = args[0].to_string_val();
    match i64::from_str_radix(&hex_str, 16) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(_) => Ok(Value::Integer(0)),
    }
}

/// octdec - Octal to decimal
pub fn octdec(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("octdec() expects exactly 1 parameter".to_string());
    }
    let oct_str = args[0].to_string_val();
    match i64::from_str_radix(&oct_str, 8) {
        Ok(n) => Ok(Value::Integer(n)),
        Err(_) => Ok(Value::Integer(0)),
    }
}

/// base_convert - Convert a number between arbitrary bases
pub fn base_convert(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("base_convert() expects at least 3 parameters".to_string());
    }
    let num = args[0].to_string_val();
    let from_base = args[1].to_int() as u32;
    let to_base = args[2].to_int() as u32;

    if from_base < 2 || from_base > 36 || to_base < 2 || to_base > 36 {
        return Err("base_convert(): Invalid base (must be between 2 and 36)".to_string());
    }

    match i64::from_str_radix(&num, from_base) {
        Ok(n) => Ok(Value::String(format!("{:x}", n))),
        Err(_) => Ok(Value::String("0".to_string())),
    }
}

/// getrandmax - Returns the maximum value that can be returned by rand()
pub fn getrandmax(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Integer(i32::MAX as i64))
}

/// getrandseed - Get random seed (returns 0 in VHP)
#[allow(dead_code)]
pub fn getrandseed(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Integer(0))
}

/// srand - Seed the random number generator (no-op in VHP)
pub fn srand(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Null)
}

/// mt_srand - Seed the mt random number generator (no-op in VHP)
pub fn mt_srand(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Null)
}

/// mt_getrandmax - Returns the maximum value that can be returned by mt_rand()
pub fn mt_getrandmax(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Integer(i32::MAX as i64))
}

/// lcg_value - Linear congruential generator value
pub fn lcg_value(_args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let val = (seed % 1000000) as f64 / 1000000.0;
    Ok(Value::Float(val))
}

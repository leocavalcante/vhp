//! Math built-in functions

use crate::runtime::Value;

/// abs - Absolute value
pub fn abs(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("abs() expects exactly 1 parameter".to_string());
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        v => Ok(Value::Float(v.to_float().abs())),
    }
}

/// ceil - Round fractions up
pub fn ceil(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ceil() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().ceil()))
}

/// floor - Round fractions down
pub fn floor(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("floor() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().floor()))
}

/// round - Rounds a float
pub fn round(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("round() expects at least 1 parameter".to_string());
    }
    let val = args[0].to_float();
    let precision = if args.len() >= 2 {
        args[1].to_int() as i32
    } else {
        0
    };
    let factor = 10_f64.powi(precision);
    Ok(Value::Float((val * factor).round() / factor))
}

/// max - Find highest value
pub fn max(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("max() expects at least 1 parameter".to_string());
    }
    let mut max_val = args[0].to_float();
    for arg in args.iter().skip(1) {
        let val = arg.to_float();
        if val > max_val {
            max_val = val;
        }
    }
    if args.iter().all(|a| matches!(a, Value::Integer(_))) {
        Ok(Value::Integer(max_val as i64))
    } else {
        Ok(Value::Float(max_val))
    }
}

/// min - Find lowest value
pub fn min(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("min() expects at least 1 parameter".to_string());
    }
    let mut min_val = args[0].to_float();
    for arg in args.iter().skip(1) {
        let val = arg.to_float();
        if val < min_val {
            min_val = val;
        }
    }
    if args.iter().all(|a| matches!(a, Value::Integer(_))) {
        Ok(Value::Integer(min_val as i64))
    } else {
        Ok(Value::Float(min_val))
    }
}

/// pow - Exponential expression
pub fn pow(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("pow() expects exactly 2 parameters".to_string());
    }
    let base = args[0].to_float();
    let exp = args[1].to_float();
    let result = base.powf(exp);
    if result.fract() == 0.0 && result.abs() < i64::MAX as f64 {
        Ok(Value::Integer(result as i64))
    } else {
        Ok(Value::Float(result))
    }
}

/// sqrt - Square root
pub fn sqrt(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sqrt() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().sqrt()))
}

/// rand - Generate a random integer
pub fn rand(args: &[Value]) -> Result<Value, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    let (min, max) = if args.len() >= 2 {
        (args[0].to_int(), args[1].to_int())
    } else if args.len() == 1 {
        (0, args[0].to_int())
    } else {
        (0, i32::MAX as i64)
    };

    let range = (max - min + 1) as u128;
    let random = if range > 0 {
        min + ((seed % range) as i64)
    } else {
        min
    };

    Ok(Value::Integer(random))
}

/// sin - Sine of an angle in radians
pub fn sin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sin() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().sin()))
}

/// cos - Cosine of an angle in radians
pub fn cos(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("cos() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().cos()))
}

/// tan - Tangent of an angle in radians
pub fn tan(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("tan() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().tan()))
}

/// log - Natural logarithm
#[allow(dead_code)]
pub fn log(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("log() expects exactly 1 parameter".to_string());
    }
    let val = args[0].to_float();
    if val <= 0.0 {
        return Err("log() argument must be greater than 0".to_string());
    }
    Ok(Value::Float(val.ln()))
}

/// log10 - Base-10 logarithm
pub fn log10(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("log10() expects exactly 1 parameter".to_string());
    }
    let val = args[0].to_float();
    if val <= 0.0 {
        return Err("log10() argument must be greater than 0".to_string());
    }
    Ok(Value::Float(val.log10()))
}

/// exp - Exponential function
pub fn exp(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("exp() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Float(args[0].to_float().exp()))
}

/// pi - Mathematical constant
pub fn pi(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Float(std::f64::consts::PI))
}

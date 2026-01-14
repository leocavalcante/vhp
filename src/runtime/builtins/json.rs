//! JSON encoding and decoding functions

use crate::runtime::{ArrayKey, Value};

/// json_encode - Returns a JSON representation of a value
pub fn json_encode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("json_encode() expects exactly 1 parameter, 0 given".to_string());
    }

    let result = value_to_json(&args[0], 0)?;
    Ok(Value::String(result))
}

/// json_decode - Decodes a JSON string
pub fn json_decode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("json_decode() expects exactly 1 parameter, 0 given".to_string());
    }

    let json_str = args[0].to_string_val();

    // For now, implement basic JSON parsing
    // This is a simplified version - full JSON parsing would be more complex
    match parse_json(&json_str) {
        Ok(value) => Ok(value),
        Err(_e) => Ok(Value::Null), // On parse error, return null
    }
}

fn value_to_json(value: &Value, depth: u32) -> Result<String, String> {
    // Prevent stack overflow from circular references
    if depth > 64 {
        return Err("Maximum nesting level of 64 reached".to_string());
    }

    match value {
        Value::Null => Ok("null".to_string()),
        Value::Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
        Value::Integer(n) => Ok(n.to_string()),
        Value::Float(f) => {
            // Handle special float values
            if f.is_nan() || f.is_infinite() {
                Ok("null".to_string())
            } else {
                // Format float without unnecessary decimal places
                let s = f.to_string();
                if s.contains('.') {
                    Ok(s)
                } else {
                    Ok(format!("{}.0", s))
                }
            }
        }
        Value::String(s) => {
            let mut escaped = String::new();
            escaped.push('"');
            for c in s.chars() {
                match c {
                    '"' => escaped.push_str("\\\""),
                    '\\' => escaped.push_str("\\\\"),
                    '/' => escaped.push_str("\\/"),
                    '\x08' => escaped.push_str("\\b"),
                    '\x0C' => escaped.push_str("\\f"),
                    '\n' => escaped.push_str("\\n"),
                    '\r' => escaped.push_str("\\r"),
                    '\t' => escaped.push_str("\\t"),
                    _ if c < ' ' => {
                        escaped.push_str(&format!("\\u{:04x}", c as u32));
                    }
                    _ => escaped.push(c),
                }
            }
            escaped.push('"');
            Ok(escaped)
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Ok("[]".to_string());
            }

            // Check if this is an associative array or indexed array
            let is_associative = arr.iter().any(|(k, _)| !matches!(k, ArrayKey::Integer(_)));

            if is_associative {
                // Encode as JSON object
                let mut result = String::new();
                result.push('{');
                let mut first = true;
                for (k, v) in arr {
                    if !first {
                        result.push(',');
                    }
                    first = false;

                    let key_str = match k {
                        ArrayKey::Integer(n) => n.to_string(),
                        ArrayKey::String(s) => s.clone(),
                    };

                    let key_json = value_to_json(&Value::String(key_str), depth + 1)?;
                    let value_json = value_to_json(v, depth + 1)?;

                    result.push_str(&key_json);
                    result.push(':');
                    result.push_str(&value_json);
                }
                result.push('}');
                Ok(result)
            } else {
                // Encode as JSON array
                let mut result = String::new();
                result.push('[');
                let mut first = true;
                for (_, v) in arr {
                    if !first {
                        result.push(',');
                    }
                    first = false;
                    result.push_str(&value_to_json(v, depth + 1)?);
                }
                result.push(']');
                Ok(result)
            }
        }
        Value::Object(_) => {
            // For now, encode objects as associative arrays
            // A full implementation would iterate over object properties
            Ok("{}".to_string())
        }
        Value::Closure(_) => Ok("null".to_string()),
        Value::Fiber(_) => Ok("null".to_string()),
        Value::Generator(_) => Ok("null".to_string()),
        Value::EnumCase { .. } => Ok("null".to_string()),
        Value::Exception(_) => Ok("null".to_string()),
    }
}

fn parse_value(input: &str) -> Result<(Value, &str), String> {
    let trimmed = input.trim_start();

    if trimmed.is_empty() {
        return Err("Unexpected end of input".to_string());
    }

    match trimmed.chars().next() {
        Some('"') => parse_string(trimmed),
        Some('{') => parse_object(trimmed),
        Some('[') => parse_array(trimmed),
        Some('t') => parse_literal(trimmed, "true", Value::Bool(true)),
        Some('f') => parse_literal(trimmed, "false", Value::Bool(false)),
        Some('n') => parse_literal(trimmed, "null", Value::Null),
        Some('-') | Some('0'..='9') => parse_number(trimmed),
        _ => Err(format!(
            "Unexpected character: {}",
            trimmed.chars().next().unwrap()
        )),
    }
}

fn parse_json(json_str: &str) -> Result<Value, String> {
    let trimmed = json_str.trim();

    if trimmed.is_empty() {
        return Err("Empty JSON string".to_string());
    }

    let (value, remaining) = parse_value(trimmed)?;

    if !remaining.trim().is_empty() {
        return Err("Extra characters after JSON value".to_string());
    }

    Ok(value)
}

fn parse_string(input: &str) -> Result<(Value, &str), String> {
    if !input.starts_with('"') {
        return Err("Expected string".to_string());
    }

    let mut result = String::new();
    let chars: Vec<char> = input[1..].chars().collect();
    let mut pos = 0;
    let mut escaped = false;

    while pos < chars.len() {
        let c = chars[pos];
        if escaped {
            match c {
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0C'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'u' => {
                    pos += 1;
                    // Parse 4-digit hex code
                    if pos + 4 > chars.len() {
                        return Err("Invalid Unicode escape sequence".to_string());
                    }
                    let hex: String = chars[pos..pos + 4].iter().collect();
                    let code_point = u32::from_str_radix(&hex, 16)
                        .map_err(|_| "Invalid Unicode escape sequence".to_string())?;
                    if let Some(ch) = std::char::from_u32(code_point) {
                        result.push(ch);
                    } else {
                        return Err("Invalid Unicode code point".to_string());
                    }
                    pos += 3;
                }
                _ => return Err(format!("Invalid escape sequence: \\{}", c)),
            }
            escaped = false;
        } else if c == '"' {
            return Ok((Value::String(result), &input[pos + 2..]));
        } else if c == '\\' {
            escaped = true;
        } else {
            result.push(c);
        }
        pos += 1;
    }

    Err("Unterminated string".to_string())
}

fn parse_object(input: &str) -> Result<(Value, &str), String> {
    if !input.starts_with('{') {
        return Err("Expected object".to_string());
    }

    let mut arr: Vec<(ArrayKey, Value)> = Vec::new();
    let mut rest = &input[1..];

    // Skip whitespace
    let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
    rest = &rest[whitespace_count..];

    if let Some(stripped) = rest.strip_prefix('}') {
        return Ok((Value::Array(arr), stripped));
    }

    loop {
        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Parse key
        let (key_value, key_remaining) = parse_string(rest)?;
        rest = key_remaining;

        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Expect colon
        if !rest.starts_with(':') {
            return Err("Expected ':' after object key".to_string());
        }
        rest = &rest[1..];

        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Parse value
        let (value, value_remaining) = parse_value(rest)?;
        rest = value_remaining;

        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Add to array
        if let Value::String(key) = key_value {
            arr.push((ArrayKey::String(key), value));
        }

        // Check for comma or closing brace
        if let Some(stripped) = rest.strip_prefix(',') {
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix('}') {
            return Ok((Value::Array(arr), stripped));
        } else {
            return Err("Expected ',' or '}' in object".to_string());
        }
    }
}

fn parse_array(input: &str) -> Result<(Value, &str), String> {
    if !input.starts_with('[') {
        return Err("Expected array".to_string());
    }

    let mut arr: Vec<(ArrayKey, Value)> = Vec::new();
    let mut rest = &input[1..];

    // Skip whitespace
    let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
    rest = &rest[whitespace_count..];

    if let Some(stripped) = rest.strip_prefix(']') {
        return Ok((Value::Array(arr), stripped));
    }

    let mut index: i64 = 0;

    loop {
        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Parse value
        let (value, value_remaining) = parse_value(rest)?;
        rest = value_remaining;

        // Skip whitespace
        let whitespace_count = rest.chars().take_while(|c| c.is_ascii_whitespace()).count();
        rest = &rest[whitespace_count..];

        // Add to array
        arr.push((ArrayKey::Integer(index), value));
        index += 1;

        // Check for comma or closing bracket
        if let Some(stripped) = rest.strip_prefix(',') {
            rest = stripped;
        } else if let Some(stripped) = rest.strip_prefix(']') {
            return Ok((Value::Array(arr), stripped));
        } else {
            return Err("Expected ',' or ']' in array".to_string());
        }
    }
}

fn parse_literal<'a>(
    input: &'a str,
    literal: &str,
    value: Value,
) -> Result<(Value, &'a str), String> {
    if let Some(stripped) = input.strip_prefix(literal) {
        Ok((value, stripped))
    } else {
        Err(format!("Expected literal: {}", literal))
    }
}

fn parse_number(input: &str) -> Result<(Value, &str), String> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;

    // Parse sign
    if pos < chars.len() && chars[pos] == '-' {
        pos += 1;
    }

    // Parse integer part
    let mut has_digits = false;
    while pos < chars.len() && chars[pos].is_ascii_digit() {
        pos += 1;
        has_digits = true;
    }

    if !has_digits {
        return Err("Expected digit in number".to_string());
    }

    // Parse fraction part
    if pos < chars.len() && chars[pos] == '.' {
        pos += 1;
        let mut has_frac_digits = false;
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
            has_frac_digits = true;
        }
        if !has_frac_digits {
            return Err("Expected digits after decimal point".to_string());
        }
    }

    // Parse exponent part
    if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
        pos += 1;
        if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
            pos += 1;
        }
        let mut has_exp_digits = false;
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
            has_exp_digits = true;
        }
        if !has_exp_digits {
            return Err("Expected digits in exponent".to_string());
        }
    }

    let num_str: String = chars[..pos].iter().collect();

    if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
        let value: f64 = num_str.parse().map_err(|_| "Invalid float".to_string())?;
        Ok((Value::Float(value), &input[pos..]))
    } else {
        let value: i64 = num_str.parse().map_err(|_| "Invalid integer".to_string())?;
        Ok((Value::Integer(value), &input[pos..]))
    }
}

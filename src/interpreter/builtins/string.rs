//! String built-in functions

use crate::interpreter::value::Value;

/// strlen - Get string length
pub fn strlen(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strlen() expects exactly 1 parameter".to_string());
    }
    Ok(Value::Integer(args[0].to_string_val().len() as i64))
}

/// substr - Return part of a string
pub fn substr(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("substr() expects at least 2 parameters".to_string());
    }
    let s = args[0].to_string_val();
    let start = args[1].to_int();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;

    let start_idx = if start < 0 {
        (len + start).max(0) as usize
    } else {
        start.min(len) as usize
    };

    let result = if args.len() >= 3 {
        let length = args[2].to_int();
        if length < 0 {
            let end_idx = ((len + length) as usize).max(start_idx);
            chars[start_idx..end_idx].iter().collect()
        } else {
            chars[start_idx..].iter().take(length as usize).collect()
        }
    } else {
        chars[start_idx..].iter().collect()
    };

    Ok(Value::String(result))
}

/// strtoupper - Make a string uppercase
pub fn strtoupper(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strtoupper() expects exactly 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val().to_uppercase()))
}

/// strtolower - Make a string lowercase
pub fn strtolower(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strtolower() expects exactly 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val().to_lowercase()))
}

/// trim - Strip whitespace from beginning and end
pub fn trim(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("trim() expects at least 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val().trim().to_string()))
}

/// ltrim - Strip whitespace from beginning
pub fn ltrim(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ltrim() expects at least 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val().trim_start().to_string()))
}

/// rtrim - Strip whitespace from end
pub fn rtrim(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("rtrim() expects at least 1 parameter".to_string());
    }
    Ok(Value::String(args[0].to_string_val().trim_end().to_string()))
}

/// str_repeat - Repeat a string
pub fn str_repeat(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("str_repeat() expects exactly 2 parameters".to_string());
    }
    let s = args[0].to_string_val();
    let times = args[1].to_int().max(0) as usize;
    Ok(Value::String(s.repeat(times)))
}

/// str_replace - Replace all occurrences of search with replace
pub fn str_replace(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("str_replace() expects at least 3 parameters".to_string());
    }
    let search = args[0].to_string_val();
    let replace = args[1].to_string_val();
    let subject = args[2].to_string_val();
    Ok(Value::String(subject.replace(&search, &replace)))
}

/// strpos - Find position of first occurrence
pub fn strpos(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("strpos() expects at least 2 parameters".to_string());
    }
    let haystack = args[0].to_string_val();
    let needle = args[1].to_string_val();
    match haystack.find(&needle) {
        Some(pos) => Ok(Value::Integer(pos as i64)),
        None => Ok(Value::Bool(false)),
    }
}

/// str_contains - Check if string contains substring
pub fn str_contains(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("str_contains() expects exactly 2 parameters".to_string());
    }
    let haystack = args[0].to_string_val();
    let needle = args[1].to_string_val();
    Ok(Value::Bool(haystack.contains(&needle)))
}

/// str_starts_with - Check if string starts with substring
pub fn str_starts_with(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("str_starts_with() expects exactly 2 parameters".to_string());
    }
    let haystack = args[0].to_string_val();
    let needle = args[1].to_string_val();
    Ok(Value::Bool(haystack.starts_with(&needle)))
}

/// str_ends_with - Check if string ends with substring
pub fn str_ends_with(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("str_ends_with() expects exactly 2 parameters".to_string());
    }
    let haystack = args[0].to_string_val();
    let needle = args[1].to_string_val();
    Ok(Value::Bool(haystack.ends_with(&needle)))
}

/// ucfirst - Make first character uppercase
pub fn ucfirst(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ucfirst() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let mut chars = s.chars();
    let result = match chars.next() {
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    };
    Ok(Value::String(result))
}

/// lcfirst - Make first character lowercase
pub fn lcfirst(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("lcfirst() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let mut chars = s.chars();
    let result = match chars.next() {
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    };
    Ok(Value::String(result))
}

/// ucwords - Uppercase first character of each word
pub fn ucwords(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ucwords() expects at least 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let result: String = s
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Value::String(result))
}

/// strrev - Reverse a string
pub fn strrev(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strrev() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    Ok(Value::String(s.chars().rev().collect()))
}

/// str_pad - Pad a string to a certain length
pub fn str_pad(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("str_pad() expects at least 2 parameters".to_string());
    }
    let s = args[0].to_string_val();
    let length = args[1].to_int() as usize;
    let pad_string = if args.len() >= 3 {
        args[2].to_string_val()
    } else {
        " ".to_string()
    };
    let pad_type = if args.len() >= 4 {
        args[3].to_int()
    } else {
        1 // STR_PAD_RIGHT
    };

    if s.len() >= length || pad_string.is_empty() {
        return Ok(Value::String(s));
    }

    let pad_needed = length - s.len();
    let pad_chars: Vec<char> = pad_string.chars().collect();

    match pad_type {
        0 => {
            // STR_PAD_LEFT
            let mut result = String::new();
            for i in 0..pad_needed {
                result.push(pad_chars[i % pad_chars.len()]);
            }
            result.push_str(&s);
            Ok(Value::String(result))
        }
        2 => {
            // STR_PAD_BOTH
            let left_pad = pad_needed / 2;
            let right_pad = pad_needed - left_pad;
            let mut result = String::new();
            for i in 0..left_pad {
                result.push(pad_chars[i % pad_chars.len()]);
            }
            result.push_str(&s);
            for i in 0..right_pad {
                result.push(pad_chars[i % pad_chars.len()]);
            }
            Ok(Value::String(result))
        }
        _ => {
            // STR_PAD_RIGHT (default)
            let mut result = s;
            for i in 0..pad_needed {
                result.push(pad_chars[i % pad_chars.len()]);
            }
            Ok(Value::String(result))
        }
    }
}

/// explode - Split a string by delimiter (stub - requires arrays)
pub fn explode(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("explode() expects at least 2 parameters".to_string());
    }
    let _delimiter = args[0].to_string_val();
    let string = args[1].to_string_val();
    // For now, just return the original string
    // Full implementation requires array support
    Ok(Value::String(string))
}

/// implode - Join array elements (stub - requires arrays)
pub fn implode(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("implode() expects at least 1 parameter".to_string());
    }
    // Since we don't have arrays yet, return empty string
    Ok(Value::String(String::new()))
}

/// sprintf - Return a formatted string
pub fn sprintf(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sprintf() expects at least 1 parameter".to_string());
    }
    let format = args[0].to_string_val();
    let mut arg_idx = 1;

    let chars: Vec<char> = format.chars().collect();
    let mut i = 0;
    let mut output = String::new();

    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            match chars[i + 1] {
                '%' => {
                    output.push('%');
                    i += 2;
                }
                's' => {
                    if arg_idx < args.len() {
                        output.push_str(&args[arg_idx].to_string_val());
                        arg_idx += 1;
                    }
                    i += 2;
                }
                'd' | 'i' => {
                    if arg_idx < args.len() {
                        output.push_str(&args[arg_idx].to_int().to_string());
                        arg_idx += 1;
                    }
                    i += 2;
                }
                'f' => {
                    if arg_idx < args.len() {
                        output.push_str(&format!("{:.6}", args[arg_idx].to_float()));
                        arg_idx += 1;
                    }
                    i += 2;
                }
                _ => {
                    output.push(chars[i]);
                    i += 1;
                }
            }
        } else {
            output.push(chars[i]);
            i += 1;
        }
    }

    Ok(Value::String(output))
}

/// chr - Generate a single-byte string from a number
pub fn chr(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("chr() expects exactly 1 parameter".to_string());
    }
    let code = args[0].to_int() as u8;
    Ok(Value::String((code as char).to_string()))
}

/// ord - Convert first byte of string to value
pub fn ord(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("ord() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    match s.chars().next() {
        Some(c) => Ok(Value::Integer(c as i64)),
        None => Ok(Value::Integer(0)),
    }
}

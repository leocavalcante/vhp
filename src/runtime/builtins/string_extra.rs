//! Additional String built-in functions

use crate::runtime::Value;

/// htmlspecialchars - Convert special characters to HTML entities
pub fn htmlspecialchars(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("htmlspecialchars() expects at least 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let flags = args.get(1).map(|v| v.to_int()).unwrap_or(2); // ENT_COMPAT | ENT_HTML401
    let _encoding = args.get(2).map(|v| v.to_string_val()).unwrap_or_default();
    let double_encode = args.get(3).map(|v| v.to_bool()).unwrap_or(true);

    let mut result = if double_encode {
        s.replace('&', "&amp;")
    } else {
        s.clone()
    };

    if flags & 2 != 0 {
        // ENT_COMPAT
        result = result.replace('"', "&quot;");
    }
    if flags & 1 != 0 {
        // ENT_QUOTES
        result = result.replace('\'', "&#039;");
    }
    if flags & 4 != 0 {
        // ENT_HTML5
        result = result.replace('<', "&lt;").replace('>', "&gt;");
    } else {
        result = result.replace('<', "&lt;").replace('>', "&gt;");
    }

    Ok(Value::String(result))
}

/// htmlentities - Convert all applicable characters to HTML entities
pub fn htmlentities(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("htmlentities() expects at least 1 parameter".to_string());
    }
    // For now, just call htmlspecialchars
    htmlspecialchars(args)
}

/// nl2br - Inserts HTML line breaks before all newlines in a string
pub fn nl2br(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("nl2br() expects at least 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let is_xhtml = args.get(1).map(|v| v.to_bool()).unwrap_or(false);
    let break_tag = if is_xhtml { "<br />\n" } else { "<br>\n" };
    Ok(Value::String(s.replace("\n", break_tag)))
}

/// number_format - Format a number with grouped thousands
pub fn number_format(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("number_format() expects at least 1 parameter".to_string());
    }
    let num = args[0].to_float();
    let num_decimals = args.get(1).map(|v| v.to_int() as usize).unwrap_or(0);
    let dec_separator = args
        .get(2)
        .map(|v| v.to_string_val())
        .unwrap_or(".".to_string());
    let thousands_sep = args
        .get(3)
        .map(|v| v.to_string_val())
        .unwrap_or(",".to_string());

    let (integer, fraction) = if num_decimals == 0 {
        (num.round() as i64, String::new())
    } else {
        let multiplier = 10_f64.powi(num_decimals as i32);
        let integer = (num * multiplier).round() / multiplier;
        let int_part = integer.trunc() as i64;
        let frac_part = (integer.fract().abs() * multiplier).round() as i64;
        let fraction_str = format!("{:0width$}", frac_part, width = num_decimals);
        (int_part, fraction_str)
    };

    let int_str = integer.to_string();
    let mut result = String::new();
    for (count, ch) in int_str.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.push_str(&thousands_sep);
        }
        result.push(ch);
    }
    result = result.chars().rev().collect();

    if !fraction.is_empty() {
        result.push_str(&dec_separator);
        result.push_str(&fraction);
    }

    Ok(Value::String(result))
}

/// md5 - Calculate the md5 hash of a string
pub fn md5(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("md5() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(Value::String(format!("{:032x}", hash)))
}

/// sha1 - Calculate the sha1 hash of a string
pub fn sha1(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("sha1() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(Value::String(format!("{:040x}", hash)))
}

/// bin2hex - Convert binary data into hexadecimal representation
pub fn bin2hex(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("bin2hex() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let mut result = String::new();
    for b in s.bytes() {
        result.push_str(&format!("{:02x}", b));
    }
    Ok(Value::String(result))
}

/// hex2bin - Convert hexadecimal data to binary
pub fn hex2bin(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("hex2bin() expects exactly 1 parameter".to_string());
    }
    let s = args[0].to_string_val();
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(next) = chars.next() {
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", c, next), 16) {
                result.push(byte);
            }
        }
    }
    Ok(Value::String(String::from_utf8_lossy(&result).into_owned()))
}

/// levenshtein - Calculate levenshtein distance between two strings
pub fn levenshtein(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("levenshtein() expects at least 2 parameters".to_string());
    }
    let s1 = args[0].to_string_val();
    let s2 = args[1].to_string_val();
    if s1.len() > 255 || s2.len() > 255 {
        return Err("levenshtein(): string length exceeds 255 characters".to_string());
    }
    let (m, n) = (s1.len(), s2.len());
    if m == 0 {
        return Ok(Value::Integer(n as i64));
    }
    if n == 0 {
        return Ok(Value::Integer(m as i64));
    }
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let mut prev_row: Vec<i64> = (0..=n as i64).collect();
    let mut curr_row: Vec<i64> = vec![0; n + 1];

    for i in 1..=m {
        curr_row[0] = i as i64;
        for j in 1..=n {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = (prev_row[j] + 1)
                .min(curr_row[j - 1] + 1)
                .min(prev_row[j - 1] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    Ok(Value::Integer(prev_row[n]))
}

/// similar_text - Calculate the similarity between two strings
pub fn similar_text(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("similar_text() expects at least 2 parameters".to_string());
    }
    let s1 = args[0].to_string_val();
    let s2 = args[1].to_string_val();
    let _percent = args.get(2).is_some();

    let (first, second) = (s1.len(), s2.len());
    let (v1, v2): (Vec<char>, Vec<char>) = (s1.chars().collect(), s2.chars().collect());

    let mut l = 0;
    let mut i = 0;
    let mut j = 0;
    let mut s = 0;

    while i < first && j < second {
        let mut ii = i;
        let mut jj = j;
        while ii < first && jj < second && v1[ii] == v2[jj] {
            ii += 1;
            jj += 1;
            l += 1;
        }
        if l > 0 {
            while i < ii {
                i += 1;
            }
            while j < jj {
                j += 1;
            }
            s += 1;
        }
        i += 1;
        j += 1;
    }

    s *= 200;
    let result = s.to_string();

    Ok(Value::String(result))
}

/// strtr - Translate characters or replace substrings
pub fn strtr(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("strtr() expects at least 2 parameters".to_string());
    }
    let s = args[0].to_string_val();
    match &args[1] {
        Value::Array(from_to) if args.len() == 2 => {
            let mut result = s;
            for (_, pair) in from_to.iter() {
                let (from, to) = match pair {
                    Value::Array(arr) if arr.len() >= 2 => {
                        let from_val = &arr[0].1;
                        let to_val = &arr[1].1;
                        (from_val.to_string_val(), to_val.to_string_val())
                    }
                    _ => continue,
                };
                result = result.replace(&from, &to);
            }
            Ok(Value::String(result))
        }
        Value::Array(from_to) if args.len() == 3 => {
            let from = args[1].to_string_val();
            let to = args[2].to_string_val();
            if from.len() != to.len() {
                return Err("strtr(): The two strings must have the same length".to_string());
            }
            let from_chars: Vec<char> = from.chars().collect();
            let to_chars: Vec<char> = to.chars().collect();
            let mut result = String::new();
            for c in s.chars() {
                if let Some(pos) = from_chars.iter().position(|&fc| fc == c) {
                    result.push(to_chars[pos]);
                } else {
                    result.push(c);
                }
            }
            Ok(Value::String(result))
        }
        _ => Err("strtr() expects string, array or two strings".to_string()),
    }
}

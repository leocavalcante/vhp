use crate::runtime::value::array_key::ArrayKey;
use crate::runtime::Value;

fn value_to_string_val(v: &Value) -> String {
    v.to_string_val()
}

pub fn preg_quote(args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("preg_quote() expects at least 1 parameter".to_string());
    }
    let str = value_to_string_val(&args[0]);
    let delimiter = if args.len() > 1 {
        value_to_string_val(&args[1])
    } else {
        String::new()
    };

    let mut result = String::with_capacity(str.len() * 2);
    for c in str.chars() {
        match c {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                result.push('\\');
                result.push(c);
            }
            _ => result.push(c),
        }
    }

    if !delimiter.is_empty() {
        for c in delimiter.chars() {
            match c {
                '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
                | '\\' => {
                    result.push('\\');
                    result.push(c);
                }
                _ => result.push(c),
            }
        }
    }

    Ok(Value::String(result))
}

pub fn preg_match(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("preg_match() expects at least 2 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let subject = value_to_string_val(&args[1]);
    let _flags = if args.len() > 3 { args[3].to_int() } else { 0 };
    let offset = if args.len() > 4 { args[4].to_int() } else { 0 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    let start = if offset < 0 {
        if offset.abs() as usize > subject.len() {
            return Ok(Value::Integer(0));
        }
        subject.len() - offset.abs() as usize
    } else {
        offset as usize
    };

    if start > subject.len() {
        return Ok(Value::Integer(0));
    }

    let subject_sub = &subject[start..];

    if let Some(m) = re.find(subject_sub) {
        if args.len() > 2 && args[2] != Value::Null {
            let mut matches_array: Vec<(ArrayKey, Value)> = Vec::new();
            matches_array.push((ArrayKey::Integer(0), Value::String(m.as_str().to_string())));
            matches_array.push((
                ArrayKey::Integer(1),
                Value::String((m.start() as i64 + start as i64).to_string()),
            ));
            matches_array.push((
                ArrayKey::Integer(2),
                Value::String((m.end() as i64 + start as i64).to_string()),
            ));
            Ok(Value::Array(matches_array))
        } else {
            Ok(Value::Integer(1))
        }
    } else {
        Ok(Value::Integer(0))
    }
}

pub fn preg_match_all(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("preg_match_all() expects at least 2 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let subject = value_to_string_val(&args[1]);
    let _flags = if args.len() > 3 { args[3].to_int() } else { 0 };
    let offset = if args.len() > 4 { args[4].to_int() } else { 0 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    let start = if offset < 0 {
        if offset.abs() as usize > subject.len() {
            return Ok(Value::Integer(0));
        }
        subject.len() - offset.abs() as usize
    } else {
        offset as usize
    };

    if start > subject.len() {
        return Ok(Value::Integer(0));
    }

    let subject_sub = &subject[start..];
    let matches_vec: Vec<_> = re.find_iter(subject_sub).collect();
    let count = matches_vec.len();

    if args.len() > 2 && args[2] != Value::Null {
        let mut result: Vec<(ArrayKey, Value)> = Vec::new();
        for m in &matches_vec {
            let mut match_data: Vec<(ArrayKey, Value)> = Vec::new();
            match_data.push((ArrayKey::Integer(0), Value::String(m.as_str().to_string())));
            match_data.push((
                ArrayKey::Integer(1),
                Value::String((m.start() as i64 + start as i64).to_string()),
            ));
            match_data.push((
                ArrayKey::Integer(2),
                Value::String((m.end() as i64 + start as i64).to_string()),
            ));
            result.push((
                ArrayKey::Integer(result.len() as i64),
                Value::Array(match_data),
            ));
        }
        Ok(Value::Array(result))
    } else {
        Ok(Value::Integer(count as i64))
    }
}

pub fn preg_split(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("preg_split() expects at least 2 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let subject = value_to_string_val(&args[1]);
    let limit = if args.len() > 2 { args[2].to_int() } else { -1 };
    let _flags = if args.len() > 3 { args[3].to_int() } else { 0 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    let parts: Vec<Value> = if limit == 0 {
        Vec::new()
    } else if limit < 0 {
        re.split(&subject)
            .map(|s: &str| Value::String(s.to_string()))
            .collect()
    } else {
        re.splitn(&subject, limit as usize)
            .map(|s: &str| Value::String(s.to_string()))
            .collect()
    };

    Ok(Value::Array(
        parts
            .into_iter()
            .enumerate()
            .map(|(i, v)| (ArrayKey::Integer(i as i64), v))
            .collect(),
    ))
}

pub fn preg_replace(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("preg_replace() expects at least 3 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let replacement = value_to_string_val(&args[1]);
    let subject = value_to_string_val(&args[2]);
    let limit = if args.len() > 3 { args[3].to_int() } else { -1 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    let result = if limit < 0 {
        re.replace_all(&subject, &replacement).to_string()
    } else if limit == 0 {
        subject
    } else {
        re.replacen(&subject, limit as usize, &replacement)
            .to_string()
    };

    Ok(Value::String(result))
}

pub fn preg_replace_callback(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("preg_replace_callback() expects at least 3 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let subject = value_to_string_val(&args[2]);
    let _limit = if args.len() > 3 { args[3].to_int() } else { -1 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let _re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    Ok(Value::String(subject))
}

pub fn preg_grep(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("preg_grep() expects at least 2 parameters".to_string());
    }
    let pattern = value_to_string_val(&args[0]);
    let _flags = if args.len() > 2 { args[2].to_int() } else { 0 };

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_new_line(regex_flags.dot_all)
        .build()
        .map_err(|_| "Invalid regex pattern")?;

    let input_vec = match &args[1] {
        Value::Array(arr) => arr.clone(),
        _ => return Err("preg_grep() expects parameter 2 to be array".to_string()),
    };

    let result: Vec<(ArrayKey, Value)> = input_vec
        .into_iter()
        .filter(|(_k, item)| {
            let s = value_to_string_val(item);
            re.is_match(&s)
        })
        .enumerate()
        .map(|(i, (_k, v))| (ArrayKey::Integer(i as i64), v.clone()))
        .collect();

    Ok(Value::Array(result))
}

struct RegexFlags {
    ignore_case: bool,
    multi_line: bool,
    dot_all: bool,
    extended: bool,
    unicode: bool,
    ungreedy: bool,
}

fn parse_pattern(pattern: &str) -> (String, RegexFlags) {
    let mut flags = RegexFlags {
        ignore_case: false,
        multi_line: false,
        dot_all: false,
        extended: false,
        unicode: true,
        ungreedy: false,
    };

    if pattern.len() < 2 {
        return (pattern.to_string(), flags);
    }

    let delimiter = pattern.chars().next().unwrap();
    let end_pos = pattern[1..].find(delimiter).map(|i| i + 1);

    if let Some(pos) = end_pos {
        let pattern_str = &pattern[1..pos];
        let modifier_str = &pattern[pos + 1..];

        let mut new_pattern = String::new();

        let mut chars = pattern_str.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next_c) = chars.next() {
                    match next_c {
                        'd' => new_pattern.push_str("[0-9]"),
                        'D' => new_pattern.push_str("[^0-9]"),
                        'w' => new_pattern.push_str("[a-zA-Z0-9_]"),
                        'W' => new_pattern.push_str("[^a-zA-Z0-9_]"),
                        's' => new_pattern.push_str("\\s"),
                        'S' => new_pattern.push_str("\\S"),
                        _ => {
                            new_pattern.push('\\');
                            new_pattern.push(next_c);
                        }
                    }
                }
            } else {
                new_pattern.push(c);
            }
        }

        for c in modifier_str.chars() {
            match c {
                'i' => flags.ignore_case = true,
                'm' => flags.multi_line = true,
                's' => flags.dot_all = true,
                'x' => flags.extended = true,
                'u' => flags.unicode = true,
                'U' => flags.ungreedy = true,
                _ => {}
            }
        }

        (new_pattern, flags)
    } else {
        (pattern.to_string(), flags)
    }
}

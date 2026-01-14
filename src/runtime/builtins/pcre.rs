use crate::runtime::Value;

pub fn preg_quote(str: Value, delimiter: Value) -> Value {
    let str = str.to_string();
    let delimiter = delimiter.to_string();

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
                '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                    result.push('\\');
                    result.push(c);
                }
                _ => result.push(c),
            }
        }
    }

    Value::String(result)
}

pub fn preg_match(pattern: Value, subject: Value, matches: Value, flags: Value, offset: Value) -> Value {
    let pattern = pattern.to_string();
    let subject = subject.to_string();
    let _flags = flags.to_int();
    let offset = offset.to_int();

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        Err(_) => return Value::False,
    };

    let start = if offset < 0 {
        if offset.abs() as usize > subject.len() {
            return Value::Int(0);
        }
        subject.len() - offset.abs() as usize
    } else {
        offset as usize
    };

    if start > subject.len() {
        return Value::Int(0);
    }

    let subject_sub = &subject[start..];

    if let Some(m) = re.find(subject_sub) {
        if matches != Value::Null {
            let mut matches_array = Vec::new();
            matches_array.push(Value::String(m.as_str().to_string()));
            matches_array.push(Value::String(m.start() as i64 + start as i64));
            matches_array.push(Value::String(m.end() as i64 + start as i64));
            Value::Array(matches_array)
        } else {
            Value::Int(1)
        }
    } else {
        Value::Int(0)
    }
}

pub fn preg_match_all(pattern: Value, subject: Value, matches: Value, flags: Value, offset: Value) -> Value {
    let pattern = pattern.to_string();
    let subject = subject.to_string();
    let _flags = flags.to_int();
    let offset = offset.to_int();

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        Err(_) => return Value::False,
    };

    let start = if offset < 0 {
        if offset.abs() as usize > subject.len() {
            return Value::Int(0);
        }
        subject.len() - offset.abs() as usize
    } else {
        offset as usize
    };

    if start > subject.len() {
        return Value::Int(0);
    }

    let subject_sub = &subject[start..];
    let matches_vec: Vec<_> = re.find_iter(subject_sub).collect();
    let count = matches_vec.len();

    if matches != Value::Null {
        let mut result = Vec::new();
        for m in &matches_vec {
            let mut match_data = Vec::new();
            match_data.push(Value::String(m.as_str().to_string()));
            match_data.push(Value::String(m.start() as i64 + start as i64));
            match_data.push(Value::String(m.end() as i64 + start as i64));
            result.push(Value::Array(match_data));
        }
        Value::Array(result)
    } else {
        Value::Int(count as i64)
    }
}

pub fn preg_split(pattern: Value, subject: Value, limit: Value, flags: Value) -> Value {
    let pattern = pattern.to_string();
    let subject = subject.to_string();
    let limit_val = limit.to_int();
    let flags_val = flags.to_int();

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        return Value::False,
    };

    let parts: Vec<Value> = if limit_val == 0 {
        Vec::new()
    } else if limit_val < 0 {
        re.split(&subject)
            .map(|s| Value::String(s.to_string()))
            .collect()
    } else {
        re.splitn(limit_val as usize, &subject)
            .map(|s| Value::String(s.to_string()))
            .collect()
    };

    Value::Array(parts)
}

pub fn preg_replace(pattern: Value, replacement: Value, subject: Value, limit: Value, count: Value) -> Value {
    let pattern = pattern.to_string();
    let replacement = replacement.to_string();
    let subject_val = subject.to_string();
    let limit_val = limit.to_int();
    let _count = count;

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        return Value::False,
    };

    let result = if limit_val < 0 {
        re.replace_all(&subject_val, &replacement).to_string()
    } else if limit_val == 0 {
        subject_val
    } else {
        re.replacen(limit_val as usize, &subject_val, &replacement).to_string()
    };

    Value::String(result)
}

pub fn preg_replace_callback(pattern: Value, callback: Value, subject: Value, limit: Value, count: Value) -> Value {
    let pattern = pattern.to_string();
    let subject_val = subject.to_string();
    let limit_val = limit.to_int();
    let _count = count;

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        return Value::False,
    };

    Value::String(subject_val)
}

pub fn preg_grep(pattern: Value, input: Value, flags: Value) -> Value {
    let pattern = pattern.to_string();
    let input_val = input.to_array();
    let _flags = flags.to_int();

    let (php_pattern, regex_flags) = parse_pattern(&pattern);

    let re = match regex::RegexBuilder::new(&php_pattern)
        .case_insensitive(regex_flags.ignore_case)
        .multi_line(regex_flags.multi_line)
        .dot_matches_newline(regex_flags.dot_all)
        .build()
    {
        Ok(re) => re,
        return Value::False,
    };

    let result: Vec<Value> = input_val
        .into_iter()
        .filter(|item| {
            let s = item.to_string();
            re.is_match(&s)
        })
        .collect();

    Value::Array(result)
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

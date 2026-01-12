//! File I/O built-in functions

use crate::runtime::Value;
use std::fs;
use std::io::Write;

/// file_get_contents - Reads entire file into a string
pub fn file_get_contents(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("file_get_contents() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    match fs::read_to_string(&filename) {
        Ok(content) => Ok(Value::String(content)),
        Err(_e) => Ok(Value::Bool(false)),
    }
}

/// file_put_contents - Write data to a file
pub fn file_put_contents<W: Write>(args: &[Value], _output: &mut W) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("file_put_contents() expects at least 2 parameters".to_string());
    }

    let filename = args[0].to_string_val();
    let data = args[1].to_string_val();

    match fs::write(&filename, data) {
        Ok(_) => Ok(Value::Integer(1)),
        Err(_) => Ok(Value::Integer(0)),
    }
}

/// file_exists - Checks whether a file or directory exists
pub fn file_exists(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("file_exists() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    let exists = fs::metadata(&filename).is_ok();
    Ok(Value::Bool(exists))
}

/// is_file - Tells whether a filename is a regular file
pub fn is_file(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_file() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    let is_reg_file = match fs::metadata(&filename) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };

    Ok(Value::Bool(is_reg_file))
}

/// is_dir - Tells whether a filename is a directory
pub fn is_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_dir() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    let is_dir = match fs::metadata(&filename) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => false,
    };

    Ok(Value::Bool(is_dir))
}

/// filemtime - Gets file modification time
pub fn filemtime(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("filemtime() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    match fs::metadata(&filename) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs() as i64)
                })
                .unwrap_or(0);
            Ok(Value::Integer(modified))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// filesize - Gets file size
pub fn filesize(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("filesize() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    match fs::metadata(&filename) {
        Ok(metadata) => {
            let size = metadata.len() as i64;
            Ok(Value::Integer(size))
        }
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// unlink - Deletes a file
pub fn unlink(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("unlink() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    match fs::remove_file(&filename) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(_) => Ok(Value::Bool(false)),
    }
}

/// is_readable - Tells whether a file exists and is readable
pub fn is_readable(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_readable() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    let exists = fs::metadata(&filename).is_ok();
    Ok(Value::Bool(exists))
}

/// is_writable - Tells whether a file exists and is writable
pub fn is_writable(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("is_writable() expects exactly 1 parameter, 0 given".to_string());
    }

    let filename = args[0].to_string_val();

    let exists = fs::metadata(&filename).is_ok();
    Ok(Value::Bool(exists))
}

//! Date formatting functions

use crate::runtime::Value;
use chrono::{DateTime, Datelike, Timelike, Utc};

/// gmdate() - Format GMT/UTC date
///
/// Returns a string formatted according to the given format string
/// using the Greenwich Mean Time (GMT).
///
/// PHP equivalent: gmdate($format, $timestamp)
pub fn gmdate(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("gmdate() expects at least 1 parameter".to_string());
    }

    let format = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("gmdate() expects parameter 1 to be string".to_string()),
    };

    let timestamp = args.get(1).map(|v| v.to_int()).unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let dt = match DateTime::from_timestamp(timestamp, 0) {
        Some(d) => d,
        None => return Err("gmdate(): Invalid timestamp".to_string()),
    };

    let formatted = format_gmdate(&format, dt);
    Ok(Value::String(formatted))
}

fn format_gmdate(format: &str, dt: DateTime<Utc>) -> String {
    let mut result = String::new();
    let bytes = format.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c == '\\' && i + 1 < bytes.len() {
            result.push(bytes[i + 1] as char);
            i += 2;
            continue;
        }

        if c == 'Y' {
            result.push_str(&format!("{:04}", dt.year()));
        } else if c == 'y' {
            result.push_str(&format!("{:02}", dt.year() % 100));
        } else if c == 'm' {
            result.push_str(&format!("{:02}", dt.month()));
        } else if c == 'n' {
            result.push_str(&format!("{}", dt.month()));
        } else if c == 'd' {
            result.push_str(&format!("{:02}", dt.day()));
        } else if c == 'j' {
            result.push_str(&format!("{}", dt.day()));
        } else if c == 'H' {
            result.push_str(&format!("{:02}", dt.hour()));
        } else if c == 'i' {
            result.push_str(&format!("{:02}", dt.minute()));
        } else if c == 's' {
            result.push_str(&format!("{:02}", dt.second()));
        } else if c == 'l' {
            let weekdays = [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ];
            result.push_str(weekdays[dt.weekday().num_days_from_sunday() as usize]);
        } else if c == 'D' {
            let weekday_abbr = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
            result.push_str(weekday_abbr[dt.weekday().num_days_from_sunday() as usize]);
        } else if c == 'F' {
            let months = [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ];
            result.push_str(months[(dt.month0()) as usize]);
        } else if c == 'M' {
            let months_abbr = [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ];
            result.push_str(months_abbr[(dt.month0()) as usize]);
        } else {
            result.push(c);
        }

        i += 1;
    }

    result
}

/// gmstrftime() - Format date/time according to locale
///
/// Format the time/date according to locale settings.
///
/// PHP equivalent: gmstrftime($format, $timestamp)
pub fn gmstrftime(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("gmstrftime() expects at least 1 parameter".to_string());
    }

    let format = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("gmstrftime() expects parameter 1 to be string".to_string()),
    };

    let timestamp = args.get(1).map(|v| v.to_int()).unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let dt = match DateTime::from_timestamp(timestamp, 0) {
        Some(d) => d,
        None => return Err("gmstrftime(): Invalid timestamp".to_string()),
    };

    let formatted = format_gmstrftime(&format, dt);
    Ok(Value::String(formatted))
}

fn format_gmstrftime(format: &str, dt: DateTime<Utc>) -> String {
    let mut result = String::new();
    let bytes = format.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c == '%' && i + 1 < bytes.len() {
            let next = bytes[i + 1] as char;

            match next {
                'Y' => result.push_str(&format!("{:04}", dt.year())),
                'y' => result.push_str(&format!("{:02}", dt.year() % 100)),
                'm' => result.push_str(&format!("{:02}", dt.month())),
                'd' => result.push_str(&format!("{:02}", dt.day())),
                'H' => result.push_str(&format!("{:02}", dt.hour())),
                'M' => result.push_str(&format!("{:02}", dt.minute())),
                'S' => result.push_str(&format!("{:02}", dt.second())),
                'A' => {
                    let weekdays = [
                        "Sunday",
                        "Monday",
                        "Tuesday",
                        "Wednesday",
                        "Thursday",
                        "Friday",
                        "Saturday",
                    ];
                    result.push_str(weekdays[dt.weekday().num_days_from_sunday() as usize]);
                }
                'a' => {
                    let weekday_abbr = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                    result.push_str(weekday_abbr[dt.weekday().num_days_from_sunday() as usize]);
                }
                'B' => {
                    let months = [
                        "January",
                        "February",
                        "March",
                        "April",
                        "May",
                        "June",
                        "July",
                        "August",
                        "September",
                        "October",
                        "November",
                        "December",
                    ];
                    result.push_str(months[(dt.month0()) as usize]);
                }
                'b' => {
                    let months_abbr = [
                        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
                        "Nov", "Dec",
                    ];
                    result.push_str(months_abbr[(dt.month0()) as usize]);
                }
                _ => {
                    result.push('%');
                    result.push(next);
                }
            }

            i += 2;
        } else {
            result.push(c);
            i += 1;
        }
    }

    result
}

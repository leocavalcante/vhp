//! Timestamp functions

use crate::runtime::Value;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Timelike, Utc};

/// time() - Current Unix timestamp
///
/// Returns the current time measured in the number of seconds since
/// the Unix Epoch (January 1 1970 00:00:00 GMT).
///
/// PHP equivalent: time()
pub fn time(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("time() expects exactly 0 parameters".to_string());
    }

    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System time error: {}", e))?;

    Ok(Value::Integer(duration.as_secs() as i64))
}

/// mktime() - Get Unix timestamp from date components
///
/// Returns the Unix timestamp corresponding to the arguments given.
///
/// PHP equivalent: mktime($hour, $min, $sec, $month, $day, $year, $is_dst)
pub fn mktime(args: &[Value]) -> Result<Value, String> {
    let hour = args.get(0).map(|v| v.to_int()).unwrap_or(0) as i32;
    let minute = args.get(1).map(|v| v.to_int()).unwrap_or(0) as i32;
    let second = args.get(2).map(|v| v.to_int()).unwrap_or(0) as i32;
    let month = args
        .get(3)
        .map(|v| v.to_int())
        .unwrap_or_else(|| Utc::now().month() as i64) as i32;
    let day = args
        .get(4)
        .map(|v| v.to_int())
        .unwrap_or_else(|| Utc::now().day() as i64) as i32;
    let year = args
        .get(5)
        .map(|v| v.to_int())
        .unwrap_or_else(|| Utc::now().year() as i64) as i32;

    if month < 1 || month > 12 {
        return Ok(Value::Bool(false));
    }

    let naive_date = match NaiveDate::from_ymd_opt(year, month as u32, day as u32) {
        Some(dt) => dt,
        None => return Ok(Value::Bool(false)),
    };

    let naive_time = match NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32) {
        Some(t) => t,
        None => return Ok(Value::Bool(false)),
    };

    let naive = naive_date.and_time(naive_time);

    Ok(Value::Integer(naive.and_utc().timestamp()))
}

/// strtotime() - Parse date string to Unix timestamp
///
/// Parses an English textual datetime description into a Unix timestamp.
///
/// PHP equivalent: strtotime($time, $now)
pub fn strtotime(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("strtotime() expects at least 1 parameter".to_string());
    }

    let time_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("strtotime() expects parameter 1 to be string".to_string()),
    };

    let base_ts = args.get(1).map(|v| v.to_int()).unwrap_or_else(|| {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    });

    let result = parse_time_string(&time_str, base_ts)?;

    if let Some(ts) = result {
        Ok(Value::Integer(ts))
    } else {
        Ok(Value::Bool(false))
    }
}

fn parse_time_string(input: &str, base_ts: i64) -> Result<Option<i64>, String> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Ok(None);
    }

    let lower = trimmed.to_lowercase();

    match lower.as_str() {
        "now" => return Ok(Some(base_ts)),
        "today" => {
            let dt = match DateTime::from_timestamp(base_ts, 0) {
                Some(d) => d,
                None => return Ok(None),
            };
            let naive = dt
                .with_hour(0)
                .unwrap()
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap();
            return Ok(Some(naive.timestamp()));
        }
        "tomorrow" => {
            let dt = match DateTime::from_timestamp(base_ts, 0) {
                Some(d) => d,
                None => return Ok(None),
            };
            let result = dt + chrono::Duration::days(1);
            return Ok(Some(result.timestamp()));
        }
        "yesterday" => {
            let dt = match DateTime::from_timestamp(base_ts, 0) {
                Some(d) => d,
                None => return Ok(None),
            };
            let result = dt - chrono::Duration::days(1);
            return Ok(Some(result.timestamp()));
        }
        _ => {}
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(Some(dt.timestamp()));
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let datetime = match date.and_hms_opt(0, 0, 0) {
            Some(dt) => dt,
            None => return Ok(None),
        };
        return Ok(Some(datetime.and_utc().timestamp()));
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%d %b %Y") {
        let datetime = match date.and_hms_opt(0, 0, 0) {
            Some(dt) => dt,
            None => return Ok(None),
        };
        return Ok(Some(datetime.and_utc().timestamp()));
    }

    if let Some((sign, num, unit)) = parse_relative_time(&lower) {
        let dt = match DateTime::from_timestamp(base_ts, 0) {
            Some(d) => d,
            None => return Ok(None),
        };

        let duration = match unit.as_str() {
            "day" | "days" => chrono::Duration::days(num),
            "week" | "weeks" => chrono::Duration::weeks(num),
            "month" | "months" => chrono::Duration::days(num * 30),
            "year" | "years" => chrono::Duration::days(num * 365),
            "hour" | "hours" => chrono::Duration::hours(num),
            "minute" | "minutes" => chrono::Duration::minutes(num),
            "second" | "seconds" => chrono::Duration::seconds(num),
            _ => return Ok(None),
        };

        let result = if sign { dt + duration } else { dt - duration };
        return Ok(Some(result.timestamp()));
    }

    if let Ok(ts) = trimmed.parse::<i64>() {
        return Ok(Some(ts));
    }

    Ok(None)
}

fn parse_relative_time(input: &str) -> Option<(bool, i64, String)> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let prefix = parts[0];
    let unit = parts[1];

    let sign = match prefix.chars().next() {
        Some('+') => true,
        Some('-') => false,
        _ => return None,
    };

    let num: i64 = prefix[1..].parse().ok()?;

    Some((sign, num, unit.to_string()))
}

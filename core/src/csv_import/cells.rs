//! Cell classification: missing tokens, numbers, and ISO 8601 dates/times.

use std::str::FromStr;

use jiff::Timestamp;
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;

const MISSING_TOKENS: [&str; 6] = ["nan", "na", "n/a", "#n/a", "null", "none"];

/// What a trimmed cell holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Missing,
    Number(f64),
    DateTime { seconds: f64, has_offset: bool },
    Other,
}

pub fn classify(text: &str) -> Cell {
    if is_missing(text) {
        return Cell::Missing;
    }
    if let Some(number) = parse_number(text) {
        return if number.is_finite() {
            Cell::Number(number)
        } else {
            Cell::Missing
        };
    }
    match parse_datetime(text) {
        Some((seconds, has_offset)) => Cell::DateTime {
            seconds,
            has_offset,
        },
        None => Cell::Other,
    }
}

/// Empty, or one of the missing-value tokens (any letter case).
pub fn is_missing(text: &str) -> bool {
    text.is_empty() || MISSING_TOKENS.iter().any(|t| text.eq_ignore_ascii_case(t))
}

/// Parses a number: optional sign, `.` decimals, scientific notation. No thousands separators.
pub fn parse_number(text: &str) -> Option<f64> {
    let first = *text.as_bytes().first()?;
    if !(first.is_ascii_digit() || matches!(first, b'+' | b'-' | b'.' | b'i' | b'I' | b'n' | b'N'))
    {
        return None;
    }
    f64::from_str(text).ok()
}

/// Parses an ISO 8601 date or date-time into seconds since 1970-01-01. Values with an
/// offset (`Z`, `+02:00`) are converted to UTC; values without one are kept as written.
/// Returns `(seconds, has_offset)`.
pub fn parse_datetime(text: &str) -> Option<(f64, bool)> {
    let bytes = text.as_bytes();
    if bytes.len() < 10
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || !bytes.iter().take(4).all(u8::is_ascii_digit)
    {
        return None;
    }
    let rest = text.get(10..)?;
    if rest.is_empty() {
        let date = Date::from_str(text).ok()?;
        return Some((naive_seconds(date.to_datetime(Time::midnight()))?, false));
    }
    let has_offset = rest.ends_with(['Z', 'z']) || rest.contains(['+', '-']);
    if has_offset {
        let timestamp = Timestamp::from_str(text).ok()?;
        Some((timestamp_seconds(timestamp), true))
    } else {
        let datetime = DateTime::from_str(text).ok()?;
        Some((naive_seconds(datetime)?, false))
    }
}

fn naive_seconds(datetime: DateTime) -> Option<f64> {
    let zoned = datetime.to_zoned(TimeZone::UTC).ok()?;
    Some(timestamp_seconds(zoned.timestamp()))
}

fn timestamp_seconds(timestamp: Timestamp) -> f64 {
    timestamp.as_second() as f64 + f64::from(timestamp.subsec_nanosecond()) * 1e-9
}

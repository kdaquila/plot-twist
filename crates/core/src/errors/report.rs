use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Maximum characters of an offending value kept in reports and warnings.
const MAX_VALUE_CHARS: usize = 64;

/// A failure as shown by the GUI and returned by the local API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorReport {
    pub code: String,
    pub message: String,
    pub hint: String,
    #[ts(type = "number | null")]
    pub line: Option<u64>,
    pub column: Option<String>,
    pub value: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl ErrorReport {
    pub fn new(code: &str, message: String, hint: &str) -> Self {
        Self {
            code: code.to_owned(),
            message,
            hint: hint.to_owned(),
            line: None,
            column: None,
            value: None,
            details: None,
        }
    }
}

/// Truncates `text` to 64 characters, marking truncation with `…`.
pub fn truncate_value(text: &str) -> String {
    match text.char_indices().nth(MAX_VALUE_CHARS) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_owned(),
    }
}

/// Formats an integer with thousands separators (e.g., `1,204`).
pub fn format_count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_counts_and_truncates() {
        assert_eq!(format_count(1204), "1,204");
        assert_eq!(format_count(12), "12");
        assert_eq!(format_count(1_000_000), "1,000,000");
        let long = "x".repeat(100);
        assert_eq!(truncate_value(&long).chars().count(), 65);
        assert_eq!(truncate_value("abc"), "abc");
    }
}

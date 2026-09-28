//! Delimiter detection from a sample of the file.

use crate::dataset::Delimiter;

const CANDIDATES: [Delimiter; 3] = [Delimiter::Comma, Delimiter::Semicolon, Delimiter::Tab];
const SAMPLE_LINES: usize = 20;

/// Picks the delimiter whose per-line count (outside quotes) is most consistent over the
/// first 20 non-blank lines. Returns `None` when no candidate appears (single column).
/// Ties prefer comma, then semicolon, then tab.
pub fn sniff(sample: &[u8], sample_is_whole_file: bool) -> Option<Delimiter> {
    let lines = split_lines(sample, sample_is_whole_file);
    let mut best: Option<(Delimiter, usize, usize)> = None;
    for delimiter in CANDIDATES {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| count(line, delimiter.byte()))
            .collect();
        let Some(mode) = mode(&counts) else { continue };
        if mode == 0 {
            continue;
        }
        let consistent = counts.iter().filter(|&&c| c == mode).count();
        let better = match best {
            None => true,
            Some((_, best_consistent, best_mode)) => {
                consistent > best_consistent || (consistent == best_consistent && mode > best_mode)
            }
        };
        if better {
            best = Some((delimiter, consistent, mode));
        }
    }
    best.map(|(delimiter, _, _)| delimiter)
}

/// Splits into lines, treating newlines inside quotes as part of the line. Drops blank lines
/// and, when the sample was cut short, the possibly-partial last line.
fn split_lines(sample: &[u8], sample_is_whole_file: bool) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut in_quotes = false;
    let mut start = 0;
    for (i, &byte) in sample.iter().enumerate() {
        match byte {
            b'"' => in_quotes = !in_quotes,
            b'\n' if !in_quotes => {
                lines.push(&sample[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        if lines.len() > SAMPLE_LINES * 2 {
            break;
        }
    }
    if sample_is_whole_file && start < sample.len() {
        lines.push(&sample[start..]);
    }
    lines.retain(|line| line.iter().any(|b| !b.is_ascii_whitespace()));
    lines.truncate(SAMPLE_LINES);
    lines
}

fn count(line: &[u8], delimiter: u8) -> usize {
    let mut in_quotes = false;
    let mut n = 0;
    for &byte in line {
        if byte == b'"' {
            in_quotes = !in_quotes;
        } else if byte == delimiter && !in_quotes {
            n += 1;
        }
    }
    n
}

/// Most frequent value; ties resolve to the larger value.
fn mode(values: &[usize]) -> Option<usize> {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mut best: Option<(usize, usize)> = None;
    for run in sorted.chunk_by(|a, b| a == b) {
        let (value, len) = (run[0], run.len());
        if best.is_none_or(|(_, best_len)| len >= best_len) {
            best = Some((value, len));
        }
    }
    best.map(|(value, _)| value)
}

//! Binary view payload for the GUI (layout in contracts/tauri-commands.md).

use super::ViewPayload;

/// Little-endian: `u32 series_count`, then per series `u32 column_index`, `u8 mode`,
/// `f64 y_min`, `f64 y_max` (NaN if no visible values), `u32 point_count`, and
/// `point_count` interleaved `[x, y]` f64 pairs.
pub fn encode(payload: &ViewPayload) -> Vec<u8> {
    let size = 4 + payload
        .series
        .iter()
        .map(|s| 25 + s.points.len() * 8)
        .sum::<usize>();
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&count(payload.series.len()).to_le_bytes());
    for series in &payload.series {
        out.extend_from_slice(&series.column.0.to_le_bytes());
        out.push(series.mode as u8);
        let (y_min, y_max) = series.y_extent.unwrap_or((f64::NAN, f64::NAN));
        out.extend_from_slice(&y_min.to_le_bytes());
        out.extend_from_slice(&y_max.to_le_bytes());
        out.extend_from_slice(&count(series.points.len() / 2).to_le_bytes());
        for value in &series.points {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    out
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

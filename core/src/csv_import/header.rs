//! Header detection and column naming.

use std::collections::HashSet;

use super::cells::{Cell, classify};

/// A first row is data (no header) when every field is a number or ISO date/time.
pub fn looks_like_data(fields: &[&str]) -> bool {
    !fields.is_empty()
        && fields
            .iter()
            .all(|f| matches!(classify(f.trim()), Cell::Number(_) | Cell::DateTime { .. }))
}

/// Unique display names: header text (empty → `Column N`), duplicates suffixed `(2)`, `(3)`…
/// Without a header, `Column 1…N`.
pub fn column_names(header: Option<&[&str]>, count: usize) -> Vec<String> {
    let mut used = HashSet::with_capacity(count);
    (0..count)
        .map(|i| {
            let base = header
                .and_then(|h| h.get(i))
                .map(|name| name.trim())
                .filter(|name| !name.is_empty())
                .map_or_else(|| format!("Column {}", i + 1), str::to_owned);
            let mut name = base.clone();
            let mut n = 2;
            while used.contains(&name) {
                name = format!("{base} ({n})");
                n += 1;
            }
            used.insert(name.clone());
            name
        })
        .collect()
}

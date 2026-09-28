//! Scatter-style reduction: at most one point per occupied pixel cell, so every pixel that
//! would be inked is still inked.

use super::{RAW_LIMIT, ViewMode, Window};

pub fn reduce(xs: &[f64], ys: &[f64], count: usize, window: &Window) -> (ViewMode, Vec<f64>) {
    let visible = |x: f64, y: f64| window.x_visible(x) && window.y_visible(y);
    let points = xs
        .iter()
        .copied()
        .zip(ys.iter().copied())
        .filter(|&(x, y)| visible(x, y));
    let mut out = Vec::with_capacity(count.min(RAW_LIMIT.max(window.width * window.height)) * 2);
    if count <= RAW_LIMIT {
        for (x, y) in points {
            out.push(x);
            out.push(y);
        }
        return (ViewMode::Raw, out);
    }
    let mut occupied = vec![0u64; (window.width * window.height).div_ceil(64)];
    for (x, y) in points {
        let cell = window.row(y) * window.width + window.column(x);
        let (word, bit) = (cell / 64, 1u64 << (cell % 64));
        if occupied[word] & bit == 0 {
            occupied[word] |= bit;
            out.push(x);
            out.push(y);
        }
    }
    (ViewMode::Reduced, out)
}

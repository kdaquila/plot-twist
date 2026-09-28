//! Line-style reduction.

use super::{RAW_LIMIT, ViewMode, Window};

pub fn reduce(
    xs: &[f64],
    ys: &[f64],
    visible: usize,
    monotonic: bool,
    window: &Window,
) -> (ViewMode, Vec<f64>) {
    let n = xs.len().min(ys.len());
    if visible <= RAW_LIMIT {
        (ViewMode::Raw, raw(&xs[..n], &ys[..n], window))
    } else if monotonic {
        (ViewMode::Reduced, min_max(&xs[..n], &ys[..n], window))
    } else {
        (ViewMode::Reduced, envelope(&xs[..n], &ys[..n], window))
    }
}

fn push_point(out: &mut Vec<f64>, x: f64, y: f64) {
    out.push(x);
    out.push(y);
}

/// Adds a break unless the output is empty or already ends with one.
fn push_break(out: &mut Vec<f64>) {
    if out.last().is_some_and(|y| !y.is_nan()) {
        push_point(out, f64::NAN, f64::NAN);
    }
}

/// Visible points in file order, plus each visible point's neighbors so segments crossing
/// the view edge are drawn.
fn raw(xs: &[f64], ys: &[f64], window: &Window) -> Vec<f64> {
    let n = xs.len();
    let in_range = |i: usize| i < n && window.x_visible(xs[i]);
    let mut out = Vec::new();
    let mut last_kept: Option<usize> = None;
    for i in 0..n {
        let keep = in_range(i) || (i > 0 && in_range(i - 1)) || in_range(i + 1);
        if !keep {
            continue;
        }
        let (x, y) = (xs[i], ys[i]);
        let gap = last_kept.is_some_and(|k| k + 1 != i);
        if gap || x.is_nan() || y.is_nan() {
            push_break(&mut out);
        }
        if !x.is_nan() && !y.is_nan() {
            push_point(&mut out, x, y);
        }
        last_kept = Some(i);
    }
    out
}

/// Per pixel column: first, min, max, and last point in file order (M4). Requires
/// non-decreasing X. Missing values break the line.
fn min_max(xs: &[f64], ys: &[f64], window: &Window) -> Vec<f64> {
    let mut out = Vec::with_capacity(window.width * 8 + 8);
    let mut bucket: Option<Bucket> = None;
    let mut before: Option<usize> = None;
    let mut started = false;
    for (i, (&x, &y)) in xs.iter().zip(ys).enumerate() {
        if x.is_nan() {
            continue;
        }
        if x < window.x_min {
            before = (!y.is_nan()).then_some(i);
            continue;
        }
        if x > window.x_max {
            flush(&mut out, bucket.take(), xs, ys);
            if !y.is_nan() {
                push_point(&mut out, x, y);
            }
            break;
        }
        if !started {
            if let Some(b) = before {
                push_point(&mut out, xs[b], ys[b]);
            }
            started = true;
        }
        if y.is_nan() {
            flush(&mut out, bucket.take(), xs, ys);
            push_break(&mut out);
            continue;
        }
        let column = window.column(x);
        match bucket.as_mut() {
            Some(b) if b.column == column => b.add(i, y, ys),
            _ => {
                flush(&mut out, bucket.take(), xs, ys);
                bucket = Some(Bucket::new(column, i));
            }
        }
    }
    flush(&mut out, bucket, xs, ys);
    out
}

struct Bucket {
    column: usize,
    first: usize,
    last: usize,
    min: usize,
    max: usize,
}

impl Bucket {
    fn new(column: usize, i: usize) -> Self {
        Self {
            column,
            first: i,
            last: i,
            min: i,
            max: i,
        }
    }

    fn add(&mut self, i: usize, y: f64, ys: &[f64]) {
        self.last = i;
        if y < ys[self.min] {
            self.min = i;
        }
        if y > ys[self.max] {
            self.max = i;
        }
    }
}

fn flush(out: &mut Vec<f64>, bucket: Option<Bucket>, xs: &[f64], ys: &[f64]) {
    let Some(b) = bucket else { return };
    let mut indices = [b.first, b.min, b.max, b.last];
    indices.sort_unstable();
    let mut previous = None;
    for i in indices {
        if previous != Some(i) {
            push_point(out, xs[i], ys[i]);
            previous = Some(i);
        }
    }
}

/// Non-monotonic X: vertical min–max segment per pixel column.
fn envelope(xs: &[f64], ys: &[f64], window: &Window) -> Vec<f64> {
    let mut lows = vec![f64::INFINITY; window.width];
    let mut highs = vec![f64::NEG_INFINITY; window.width];
    for (&x, &y) in xs.iter().zip(ys) {
        if y.is_nan() || !window.x_visible(x) {
            continue;
        }
        let c = window.column(x);
        lows[c] = lows[c].min(y);
        highs[c] = highs[c].max(y);
    }
    let step = (window.x_max - window.x_min) / window.width as f64;
    let mut out = Vec::new();
    for (c, (&low, &high)) in lows.iter().zip(&highs).enumerate() {
        if low > high {
            continue;
        }
        let x = window.x_min + (c as f64 + 0.5) * step;
        push_point(&mut out, x, low);
        push_point(&mut out, x, high);
        push_point(&mut out, f64::NAN, f64::NAN);
    }
    out
}

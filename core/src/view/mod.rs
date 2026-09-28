//! Reduces series to what the requested view can show (research R4, spec FR-010):
//! raw points when few are visible, otherwise min/max per pixel column (lines) or one point
//! per occupied pixel cell (scatter), so spikes and extremes never disappear.

mod encode;
mod line;
mod scatter;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use encode::encode;

use crate::dataset::{Column, ColumnIndex, Dataset, DatasetId};
use crate::errors::PtError;
use crate::session::{PlotConfig, PlotStyle};

/// Series with at most this many visible points are sent raw.
pub const RAW_LIMIT: usize = 20_000;
const MAX_PIXELS: u32 = 8192;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ViewRequest {
    pub dataset_id: DatasetId,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Raw = 0,
    Reduced = 1,
}

/// One Y series: interleaved `[x0, y0, x1, y1, …]` in draw order. A NaN pair is a break.
/// `y_extent` is the min/max of the series' values within the requested X range.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesView {
    pub column: ColumnIndex,
    /// The column's display name, from the same dataset that produced the points.
    pub name: String,
    pub mode: ViewMode,
    pub y_extent: Option<(f64, f64)>,
    pub points: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewPayload {
    pub series: Vec<SeriesView>,
}

/// The visible window in data units and pixels.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Window {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub width: usize,
    pub height: usize,
}

impl Window {
    pub fn x_visible(&self, x: f64) -> bool {
        x >= self.x_min && x <= self.x_max
    }

    pub fn y_visible(&self, y: f64) -> bool {
        y >= self.y_min && y <= self.y_max
    }

    /// Pixel column of a visible x.
    pub fn column(&self, x: f64) -> usize {
        let t = (x - self.x_min) / (self.x_max - self.x_min);
        ((t * self.width as f64) as usize).min(self.width - 1)
    }

    /// Pixel row of a visible y.
    pub fn row(&self, y: f64) -> usize {
        let t = (y - self.y_min) / (self.y_max - self.y_min);
        ((t * self.height as f64) as usize).min(self.height - 1)
    }
}

fn window(request: &ViewRequest) -> Result<Window, PtError> {
    let bounds = [request.x_min, request.x_max, request.y_min, request.y_max];
    if bounds.iter().any(|v| !v.is_finite()) {
        return Err(invalid("ranges must be finite"));
    }
    if request.x_min >= request.x_max || request.y_min >= request.y_max {
        return Err(invalid("min must be less than max"));
    }
    let pixels = 1..=MAX_PIXELS;
    if !pixels.contains(&request.width_px) || !pixels.contains(&request.height_px) {
        return Err(invalid("width and height must be between 1 and 8192"));
    }
    Ok(Window {
        x_min: request.x_min,
        x_max: request.x_max,
        y_min: request.y_min,
        y_max: request.y_max,
        width: request.width_px as usize,
        height: request.height_px as usize,
    })
}

fn invalid(reason: &str) -> PtError {
    PtError::InvalidView {
        reason: reason.to_owned(),
    }
}

/// Reduces every Y series of `plot` for the requested view, one thread per series.
pub fn reduce(
    dataset: &Dataset,
    plot: &PlotConfig,
    request: &ViewRequest,
) -> Result<ViewPayload, PtError> {
    let window = window(request)?;
    let unknown = |name: &String| PtError::UnknownColumn {
        column: name.clone(),
    };
    let x_column = dataset
        .column_by_name(&plot.x)
        .ok_or_else(|| unknown(&plot.x))?;
    let xs = x_column.values();
    let monotonic = x_column.is_monotonic();
    let columns = plot
        .y
        .iter()
        .map(|name| dataset.column_by_name(name).ok_or_else(|| unknown(name)))
        .collect::<Result<Vec<_>, PtError>>()?;
    let one = |column: &Column| {
        let ys = column.values();
        let scan = scan(xs, ys, &window);
        let (mode, points) = match plot.style {
            PlotStyle::Line => line::reduce(xs, ys, scan.x_visible, monotonic, &window),
            PlotStyle::Scatter => scatter::reduce(xs, ys, scan.xy_visible, &window),
        };
        SeriesView {
            column: column.index,
            name: column.name.clone(),
            mode,
            y_extent: scan.y_extent,
            points,
        }
    };
    let series = std::thread::scope(|scope| {
        let handles: Vec<_> = columns
            .iter()
            .map(|&column| std::thread::Builder::new().spawn_scoped(scope, move || one(column)))
            .collect();
        handles
            .into_iter()
            .zip(&columns)
            .map(|(handle, &column)| match handle {
                Ok(handle) => handle
                    .join()
                    .map_err(|_| PtError::internal(&"view worker failed")),
                // Could not start a thread: do the work here instead.
                Err(_) => Ok(one(column)),
            })
            .collect::<Result<Vec<_>, PtError>>()
    })?;
    Ok(ViewPayload { series })
}

/// One pass over a series: visible counts (by X, and by X and Y) and the Y extent within
/// the X range.
struct Scan {
    x_visible: usize,
    xy_visible: usize,
    y_extent: Option<(f64, f64)>,
}

fn scan(xs: &[f64], ys: &[f64], window: &Window) -> Scan {
    let (mut x_visible, mut xy_visible) = (0, 0);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for (&x, &y) in xs.iter().zip(ys) {
        if y.is_nan() || !window.x_visible(x) {
            continue;
        }
        x_visible += 1;
        if window.y_visible(y) {
            xy_visible += 1;
        }
        lo = lo.min(y);
        hi = hi.max(y);
    }
    Scan {
        x_visible,
        xy_visible,
        y_extent: (x_visible > 0).then_some((lo, hi)),
    }
}

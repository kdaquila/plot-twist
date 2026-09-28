//! Data endpoints. Each calls the matching `Session` operation with `Origin::Api`.

use std::path::PathBuf;

use axum::Json;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use plot_twist_core::dataset::{BadCellPage, DatasetId};
use plot_twist_core::errors::PtError;
use plot_twist_core::session::{LoadResult, Origin, PlotConfig, SessionState};
use plot_twist_core::view::{ViewMode, ViewRequest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::AppState;
use super::errors::{ApiError, bad_request};

type ApiResult<T> = Result<Json<T>, ApiError>;

const DEFAULT_PAGE: u64 = 100;

pub async fn health() -> Json<Value> {
    Json(json!({ "app": "plot-twist", "version": env!("CARGO_PKG_VERSION"), "api_version": 1 }))
}

#[derive(Deserialize)]
pub struct LoadRequest {
    path: PathBuf,
}

pub async fn load(
    State(state): State<AppState>,
    body: Result<Json<LoadRequest>, JsonRejection>,
) -> ApiResult<LoadResult> {
    let Json(LoadRequest { path }) = body.map_err(|e| bad_request(e.body_text()))?;
    if !path.is_absolute() {
        return Err(bad_request("path must be an absolute file path"));
    }
    let session = state.session;
    let loaded = tokio::task::spawn_blocking(move || session.load_file(&path, Origin::Api))
        .await
        .map_err(|e| PtError::internal(&e))??;
    Ok(Json(loaded))
}

pub async fn state(State(state): State<AppState>) -> Json<SessionState> {
    Json(state.session.state())
}

pub async fn plot(
    State(state): State<AppState>,
    body: Result<Json<PlotConfig>, JsonRejection>,
) -> ApiResult<PlotConfig> {
    let Json(config) = body.map_err(|e| bad_request(e.body_text()))?;
    Ok(Json(state.session.set_plot(config, Origin::Api)?))
}

fn dataset_id(text: &str) -> Result<DatasetId, ApiError> {
    text.parse().map_err(|()| {
        ApiError(PtError::UnknownDataset {
            dataset_id: text.to_owned(),
        })
    })
}

#[derive(Deserialize)]
pub struct PageQuery {
    offset: Option<u64>,
    limit: Option<u64>,
}

pub async fn bad_cells(
    State(state): State<AppState>,
    Path(id): Path<String>,
    query: Result<Query<PageQuery>, QueryRejection>,
) -> ApiResult<BadCellPage> {
    let Query(page) = query.map_err(|e| bad_request(e.body_text()))?;
    let page = state.session.bad_cells(
        dataset_id(&id)?,
        page.offset.unwrap_or(0),
        page.limit.unwrap_or(DEFAULT_PAGE),
    )?;
    Ok(Json(page))
}

#[derive(Deserialize)]
pub struct ViewQuery {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
pub struct ViewResponse {
    series: Vec<SeriesResponse>,
}

#[derive(Serialize)]
struct SeriesResponse {
    column: String,
    mode: &'static str,
    y_min: Option<f64>,
    y_max: Option<f64>,
    /// `[x, y]` pairs; `null` marks a line break at a missing value.
    points: Vec<Option<[f64; 2]>>,
}

pub async fn view(
    State(state): State<AppState>,
    Path(id): Path<String>,
    query: Result<Query<ViewQuery>, QueryRejection>,
) -> ApiResult<ViewResponse> {
    let Query(q) = query.map_err(|e| bad_request(e.body_text()))?;
    let request = ViewRequest {
        dataset_id: dataset_id(&id)?,
        x_min: q.x_min,
        x_max: q.x_max,
        y_min: q.y_min,
        y_max: q.y_max,
        width_px: q.width,
        height_px: q.height,
    };
    let payload = state.session.view(&request)?;
    let series = payload
        .series
        .into_iter()
        .map(|s| SeriesResponse {
            column: s.name,
            mode: match s.mode {
                ViewMode::Raw => "raw",
                ViewMode::Reduced => "reduced",
            },
            y_min: s.y_extent.map(|e| e.0),
            y_max: s.y_extent.map(|e| e.1),
            points: s
                .points
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[x, y]| (x.is_finite() && y.is_finite()).then_some([x, y]))
                .collect(),
        })
        .collect();
    Ok(Json(ViewResponse { series }))
}

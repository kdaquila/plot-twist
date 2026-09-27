//! `PtError` → HTTP status with an `ErrorReport` body (contracts/local-api.md).

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use plot_twist_core::errors::PtError;

pub struct ApiError(pub PtError);

impl From<PtError> for ApiError {
    fn from(error: PtError) -> Self {
        Self(error)
    }
}

pub fn bad_request(reason: impl std::fmt::Display) -> ApiError {
    ApiError(PtError::BadRequest {
        reason: reason.to_string(),
    })
}

fn status(error: &PtError) -> StatusCode {
    match error {
        PtError::BadRequest { .. } | PtError::InvalidView { .. } => StatusCode::BAD_REQUEST,
        PtError::UnknownDataset { .. } | PtError::UnknownColumn { .. } => StatusCode::NOT_FOUND,
        PtError::StaleDataset { .. } | PtError::LoadInProgress | PtError::NoPlot => {
            StatusCode::CONFLICT
        }
        PtError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        PtError::FileNotFound { .. }
        | PtError::FileUnreadable { .. }
        | PtError::EmptyFile
        | PtError::NoDataRows
        | PtError::FieldCountMismatch { .. }
        | PtError::InvalidEncoding { .. }
        | PtError::MalformedQuoting { .. }
        | PtError::OutOfMemory { .. }
        | PtError::ColumnNotUsableAsX { .. }
        | PtError::ColumnNotUsableAsY { .. }
        | PtError::XAlsoY { .. }
        | PtError::NoYColumns
        | PtError::DuplicateYColumn { .. } => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (status(&self.0), Json(self.0.report())).into_response()
    }
}

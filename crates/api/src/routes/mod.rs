//! `/v1` routes and request guards.

mod data;
mod errors;

use std::sync::Arc;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use plot_twist_core::session::Session;

use errors::bad_request;

#[derive(Clone)]
pub struct AppState {
    pub session: Arc<Session>,
    port: u16,
}

pub fn router(session: Arc<Session>, port: u16) -> Router {
    let state = AppState { session, port };
    let v1 = Router::new()
        .route("/health", get(data::health))
        .route("/load", post(data::load))
        .route("/state", get(data::state))
        .route("/plot", put(data::plot))
        .route("/datasets/{id}/bad-cells", get(data::bad_cells))
        .route("/datasets/{id}/view", get(data::view));
    Router::new()
        .nest("/v1", v1)
        .fallback(|| async {
            let mut response =
                bad_request("unknown endpoint; see docs/local-api.md").into_response();
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        })
        .layer(middleware::from_fn_with_state(
            state.clone(),
            local_host_only,
        ))
        .with_state(state)
}

/// Rejects requests whose `Host` isn't this loopback server, so web pages can't reach the
/// API through DNS rebinding (FR-017).
async fn local_host_only(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let allowed = [
        format!("127.0.0.1:{}", state.port),
        format!("localhost:{}", state.port),
    ];
    if allowed.iter().any(|a| a.eq_ignore_ascii_case(host)) {
        next.run(request).await
    } else {
        let mut response =
            bad_request("requests must come from this computer (Host must be 127.0.0.1)")
                .into_response();
        *response.status_mut() = StatusCode::FORBIDDEN;
        response
    }
}

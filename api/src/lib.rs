//! plot-twist-api: the local HTTP API (contracts/local-api.md). Every route is a thin
//! adapter over the same `Session` operations the GUI uses, with `Origin::Api`.

mod discovery;
mod routes;

use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use plot_twist_core::errors::PtError;
use plot_twist_core::session::Session;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// A running API server. Call [`ApiHandle::stop`] on exit to remove the discovery file.
pub struct ApiHandle {
    base_url: String,
    discovery_file: PathBuf,
    shutdown: Option<oneshot::Sender<()>>,
}

impl ApiHandle {
    /// e.g. `http://127.0.0.1:47811/v1`.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Stops serving and removes the discovery file.
    pub fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        discovery::remove(&self.discovery_file);
    }
}

impl Drop for ApiHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Binds `127.0.0.1:port` (or an OS-assigned port if that one is busy), writes the
/// discovery file, and serves until stopped. Never binds a non-loopback address (FR-017).
pub async fn start(
    session: Arc<Session>,
    port: u16,
    discovery_file: PathBuf,
) -> Result<ApiHandle, PtError> {
    let listener = match TcpListener::bind(loopback(port)).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::warn!(port, %error, "API port busy; using an OS-assigned port");
            TcpListener::bind(loopback(0))
                .await
                .map_err(|e| PtError::internal(&e))?
        }
    };
    let address = listener.local_addr().map_err(|e| PtError::internal(&e))?;
    let base_url = format!("http://{address}/v1");
    discovery::write(&discovery_file, &base_url).map_err(|e| PtError::internal(&e))?;

    let (shutdown, stopped) = oneshot::channel::<()>();
    let app = routes::router(session, address.port());
    tokio::spawn(async move {
        let served = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await;
        if let Err(error) = served {
            tracing::error!(%error, "local API stopped");
        }
    });
    tracing::info!(%base_url, "local API listening");
    Ok(ApiHandle {
        base_url,
        discovery_file,
        shutdown: Some(shutdown),
    })
}

fn loopback(port: u16) -> SocketAddr {
    SocketAddr::from((Ipv4Addr::LOCALHOST, port))
}

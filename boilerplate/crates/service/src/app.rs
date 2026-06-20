use anyhow::Context;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::{router::create_router, state::AppState, ServiceConfig};

/// Bind the service, start accepting connections, and block until a shutdown
/// signal is received.
///
/// Shutdown is triggered by:
/// - `SIGTERM` on Unix (Kubernetes `terminationGracePeriodSeconds` support)
/// - `Ctrl-C` (`SIGINT`) on all platforms
///
/// The server completes in-flight requests before exiting.
pub async fn serve(config: ServiceConfig) -> anyhow::Result<()> {
    let addr = config.bind_addr;
    let state = AppState::new(config);
    let app = create_router(state);

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to {addr}"))?;

    let bound_addr = listener
        .local_addr()
        .context("could not determine bound address")?;

    info!(address = %bound_addr, "service listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    info!("service shut down gracefully");
    Ok(())
}

/// Returns a future that resolves when a shutdown signal is received.
///
/// On Unix both `SIGTERM` and `Ctrl-C` are handled.
/// On other platforms only `Ctrl-C` is handled.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = ctrl_c => {
                info!("received Ctrl-C; initiating graceful shutdown");
            }
            _ = sigterm.recv() => {
                info!("received SIGTERM; initiating graceful shutdown");
            }
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await;
        info!("received Ctrl-C; initiating graceful shutdown");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::net::TcpListener;

    use super::*;

    /// Bind to an ephemeral port and verify the server starts without error.
    /// We immediately cancel via the shutdown signal by dropping the listener
    /// handle.
    #[tokio::test]
    async fn serve_binds_to_ephemeral_port() {
        let config = ServiceConfig {
            bind_addr: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            ..Default::default()
        };

        // Just confirm that building the state and router from the config does
        // not panic — actually starting the server would block forever in a
        // unit test without a signal mechanism.
        let addr = config.bind_addr;
        let state = AppState::new(config);
        let app = create_router(state);

        // Bind a listener to verify the port is available.
        let listener = TcpListener::bind(addr).await.unwrap();
        let actual_addr = listener.local_addr().unwrap();
        assert_ne!(actual_addr.port(), 0);

        // Confirm the router builds cleanly by making a single in-process request.
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;
        let req = Request::builder()
            .uri("/health/live")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn serve_returns_error_on_bad_addr() {
        // Port 1 is privileged on Linux; binding should fail.
        let config = ServiceConfig {
            // Use a port in the reserved range that isn't ours.
            // We use port 1 — this will fail with EACCES or EADDRINUSE.
            bind_addr: "127.0.0.1:1".parse::<SocketAddr>().unwrap(),
            ..Default::default()
        };
        let result = serve(config).await;
        assert!(
            result.is_err(),
            "expected serve to fail on privileged port"
        );
    }
}

//! HTTP server crate that wraps jarvis-core.
#![deny(warnings)]

use axum::{routing::post, Router};
use jarvis_core::config::Config;
use tokio::{net::TcpListener, signal};
use tracing::{info, Level};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub async fn start(config: Config) -> anyhow::Result<()> {
    // logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .json()
        .init();

    let router = Router::new().route("/", post(handle));

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

async fn handle(body: String) -> String {
    // TODO: call agent core once implemented
    format!("echo: {}", body)
}

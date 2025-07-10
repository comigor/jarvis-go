//! HTTP server crate that wraps jarvis-core.
#![deny(warnings)]

use axum::{routing::post, Router, extract::Json};
use serde::{Deserialize, Serialize};
use jarvis_core::history::{save, list, Message};
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

#[derive(Debug, Deserialize)]
struct SaveReq {
    session_id: String,
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ListResp {
    messages: Vec<Message>,
}

async fn handle(Json(payload): Json<SaveReq>) -> Result<Json<ListResp>, axum::http::StatusCode> {
    let msg = Message {
        id: 0,
        session_id: payload.session_id.clone(),
        role: payload.role.clone(),
        content: payload.content.clone(),
        created_at: chrono::Utc::now(),
    };
    if let Err(e) = save(msg).await {
        tracing::error!(?e, "failed to save message");
        return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    }
    match list(&payload.session_id).await {
        Ok(messages) => Ok(Json(ListResp { messages })),
        Err(e) => {
            tracing::error!(?e, "failed to list messages");
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

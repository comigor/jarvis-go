//! Very thin binary entrypoint.
#![deny(warnings)]



use jarvis_core::config::Config;
use jarvis_web::start;
use std::path::PathBuf;


#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let path = std::env::var("JARVIS_CONFIG").unwrap_or_else(|_| "config.yaml".into());
    let config = Config::load(PathBuf::from(path))?;
    start(config).await?;
    Ok(())
}

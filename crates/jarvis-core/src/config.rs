use std::path::Path;

use figment::{providers::{Env, Format, Serialized, Yaml}, Figment};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Figment(Box<figment::Error>),
}

impl From<figment::Error> for ConfigError {
    fn from(e: figment::Error) -> Self {
        ConfigError::Figment(Box::new(e))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            api_key: None,
            model: Some("gpt-4o".into()),
            system_prompt: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    // TODO: mcp_servers when ported
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let figment = Figment::from(Serialized::defaults(Config::default()))
            .merge(Yaml::file(path))
            .merge(Env::prefixed("JARVIS_").split("__"));
        Ok(figment.extract()?)
    }
}

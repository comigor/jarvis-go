//! LLM client abstraction. Currently only supports OpenAI-compatible endpoints.
#![allow(dead_code)]

use async_openai::{types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestToolMessageArgs, Role, CreateChatCompletionResponse, CreateChatCompletionRequestArgs}, Client, config::OpenAIConfig};
use serde::{Deserialize, Serialize};
use crate::config::LlmConfig;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl From<&ChatMessage> for ChatCompletionRequestMessage {
    fn from(value: &ChatMessage) -> Self {
        match value.role {
            Role::User => ChatCompletionRequestUserMessageArgs::default()
                .content(value.content.clone())
                .build()
                .unwrap()
                .into(),
            Role::System => ChatCompletionRequestSystemMessageArgs::default()
                .content(value.content.clone())
                .build()
                .unwrap()
                .into(),
            Role::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                .content(value.content.clone())
                .build()
                .unwrap()
                .into(),
            _ => ChatCompletionRequestToolMessageArgs::default()
                .content(value.content.clone())
                .build()
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIClient {
inner: Client<OpenAIConfig>,
    
    model: String,
}

impl OpenAIClient {
    pub fn new(cfg: &LlmConfig) -> Self {
        let mut openai_cfg = OpenAIConfig::new();
        if let Some(ref base_url) = cfg.base_url {
            openai_cfg = openai_cfg.with_api_base(base_url);
        }
        if let Some(ref key) = cfg.api_key {
            openai_cfg = openai_cfg.with_api_key(key);
        }
        let inner: Client<OpenAIConfig> = Client::with_config(openai_cfg);
        let model = cfg.model.clone().unwrap_or_else(|| "gpt-4o".into());
        Self { inner, model }
    }

    pub async fn chat_completion(&self, messages: &[ChatMessage]) -> Result<CreateChatCompletionResponse> {
        let req = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages.iter().map(|m| m.into()).collect::<Vec<_>>())
            .build()?;
        let resp = self.inner.chat().create(req).await?;
        Ok(resp)
    }
}

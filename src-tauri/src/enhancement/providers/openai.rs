// OpenAI chat completions.
//
// Reference VoiceInk : AIService.swift (case .openAI) - baseURL
// https://api.openai.com/v1/chat/completions.

use anyhow::Result;
use async_trait::async_trait;

use crate::enhancement::provider::{EnhancementRequest, EnhancementResponse, LLMProvider};

use super::openai_compat;

pub struct OpenAIProvider;

const MODELS: &[&str] = &[
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.2",
    "gpt-5-mini",
    "gpt-5-nano",
    "gpt-4.1",
    "gpt-4.1-mini",
    "gpt-4.1-nano",
];

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn id(&self) -> &'static str {
        "openai"
    }
    fn label(&self) -> &'static str {
        "OpenAI"
    }
    fn default_models(&self) -> &'static [&'static str] {
        MODELS
    }
    fn default_model(&self) -> &'static str {
        "gpt-5.4"
    }
    fn endpoint(&self) -> &'static str {
        "https://api.openai.com/v1/chat/completions"
    }

    async fn chat_completion(
        &self,
        api_key: &str,
        req: &EnhancementRequest,
    ) -> Result<EnhancementResponse> {
        openai_compat::chat_completion(self.endpoint(), api_key, req).await
    }
}

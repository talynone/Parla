// Mistral chat completions.
//
// Reference VoiceInk : AIService.swift (case .mistral) - baseURL
// https://api.mistral.ai/v1/chat/completions.

use anyhow::Result;
use async_trait::async_trait;

use crate::enhancement::provider::{EnhancementRequest, EnhancementResponse, LLMProvider};

use super::openai_compat;

pub struct MistralProvider;

const MODELS: &[&str] = &[
    "mistral-large-latest",
    "mistral-medium-latest",
    "mistral-small-latest",
];

#[async_trait]
impl LLMProvider for MistralProvider {
    fn id(&self) -> &'static str {
        "mistral"
    }
    fn label(&self) -> &'static str {
        "Mistral"
    }
    fn default_models(&self) -> &'static [&'static str] {
        MODELS
    }
    fn default_model(&self) -> &'static str {
        "mistral-large-latest"
    }
    fn endpoint(&self) -> &'static str {
        "https://api.mistral.ai/v1/chat/completions"
    }

    async fn chat_completion(
        &self,
        api_key: &str,
        req: &EnhancementRequest,
    ) -> Result<EnhancementResponse> {
        openai_compat::chat_completion(self.endpoint(), api_key, req).await
    }
}

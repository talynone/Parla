// OpenRouter chat completions.
//
// Reference VoiceInk : AIService.swift (case .openRouter) - baseURL
// https://openrouter.ai/api/v1/chat/completions.
// La liste de modeles est dynamique, fetched depuis /models cote VoiceInk.
// Ici on expose un modele par defaut et on laisse le user entrer librement.

use anyhow::Result;
use async_trait::async_trait;

use crate::enhancement::provider::{EnhancementRequest, EnhancementResponse, LLMProvider};

use super::openai_compat;

pub struct OpenRouterProvider;

const MODELS: &[&str] = &[
    "openai/gpt-oss-120b",
    "anthropic/claude-sonnet-4-6",
    "google/gemini-2.5-pro",
    "meta-llama/llama-3.3-70b-instruct",
];

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    fn id(&self) -> &'static str {
        "openrouter"
    }
    fn label(&self) -> &'static str {
        "OpenRouter"
    }
    fn default_models(&self) -> &'static [&'static str] {
        MODELS
    }
    fn default_model(&self) -> &'static str {
        "openai/gpt-oss-120b"
    }
    fn endpoint(&self) -> &'static str {
        "https://openrouter.ai/api/v1/chat/completions"
    }

    async fn chat_completion(
        &self,
        api_key: &str,
        req: &EnhancementRequest,
    ) -> Result<EnhancementResponse> {
        openai_compat::chat_completion(self.endpoint(), api_key, req).await
    }
}

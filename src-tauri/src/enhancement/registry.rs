// Registry des providers LLM d'enhancement.
//
// Reference VoiceInk : les providers sont un enum `AIProvider` iteres par
// AIService. Ici on les stocke dans un Vec<Arc<dyn LLMProvider>> cree une
// fois au demarrage.

use std::sync::Arc;

use super::provider::LLMProvider;
use super::providers;

pub struct LLMRegistry {
    providers: Vec<Arc<dyn LLMProvider>>,
}

impl LLMRegistry {
    pub fn new() -> Self {
        let providers: Vec<Arc<dyn LLMProvider>> = vec![
            Arc::new(providers::anthropic::AnthropicProvider),
            Arc::new(providers::openai::OpenAIProvider),
            Arc::new(providers::gemini::GeminiProvider),
            Arc::new(providers::mistral::MistralProvider),
            Arc::new(providers::groq::GroqProvider),
            Arc::new(providers::cerebras::CerebrasProvider),
            Arc::new(providers::openrouter::OpenRouterProvider),
            Arc::new(providers::ollama::OllamaProvider),
            Arc::new(providers::llamacpp::LlamaCppProvider),
            Arc::new(providers::local_cli::LocalCLIProvider),
            Arc::new(providers::custom::CustomProvider),
        ];
        Self { providers }
    }

    pub fn list(&self) -> &[Arc<dyn LLMProvider>] {
        &self.providers
    }

    pub fn find(&self, id: &str) -> Option<Arc<dyn LLMProvider>> {
        self.providers.iter().find(|p| p.id() == id).cloned()
    }
}

impl Default for LLMRegistry {
    fn default() -> Self {
        Self::new()
    }
}

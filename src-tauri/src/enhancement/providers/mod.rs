// Adaptateurs concrets de LLMProvider.
//
// Reference VoiceInk : VoiceInk/Services/AIEnhancement/AIService.swift +
// LLMkit (OpenAILLMClient, AnthropicLLMClient, OllamaClient).

pub mod anthropic;
pub mod cerebras;
pub mod custom;
pub mod gemini;
pub mod groq;
pub mod llamacpp;
pub mod local_cli;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod openai_compat;
pub mod openrouter;
pub mod url_validator;

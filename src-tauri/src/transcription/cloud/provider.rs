// Contrats communs a tous les providers cloud.

use std::path::Path;

use async_trait::async_trait;

/// Parametres d'une requete de transcription cloud.
#[derive(Debug, Clone)]
pub struct TranscribeRequest {
    /// ID du modele specifique pour le provider (ex "whisper-large-v3-turbo").
    pub model: String,
    /// Code langue ISO 639-1 ("fr", "en") ou None pour auto.
    pub language: Option<String>,
    /// Prompt initial / biasing (pas toujours supporte).
    pub prompt: Option<String>,
    /// Dictionnaire / termes specialises.
    pub custom_vocabulary: Vec<String>,
}

impl TranscribeRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            language: None,
            prompt: None,
            custom_vocabulary: Vec::new(),
        }
    }
}

#[async_trait]
pub trait CloudTranscriptionProvider: Send + Sync {
    /// Identifiant canonique (lowercase) du provider.
    fn id(&self) -> &'static str;
    /// Verifie que la cle API est acceptee par l'endpoint.
    async fn verify_api_key(&self, api_key: &str) -> anyhow::Result<()>;
    /// Transcrit un WAV 16 kHz mono Int16.
    async fn transcribe(
        &self,
        wav_path: &Path,
        api_key: &str,
        request: &TranscribeRequest,
    ) -> anyhow::Result<String>;
}

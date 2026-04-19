// Commandes Tauri pour les providers cloud de transcription.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::services::api_keys;
use crate::transcription::cloud::{
    catalog::{cloud_providers, CloudProviderInfo, CLOUD_MODELS},
    provider::TranscribeRequest,
    CloudRegistry,
};

pub struct CloudRegistryState(pub std::sync::Arc<CloudRegistry>);

impl Default for CloudRegistryState {
    fn default() -> Self {
        Self(std::sync::Arc::new(CloudRegistry::default()))
    }
}

#[derive(Debug, Serialize)]
pub struct ProviderWithKeyStatus {
    #[serde(flatten)]
    pub info: CloudProviderInfo,
    pub has_api_key: bool,
}

#[tauri::command]
pub fn list_cloud_providers() -> Vec<ProviderWithKeyStatus> {
    cloud_providers()
        .iter()
        .map(|p| ProviderWithKeyStatus {
            info: p.clone(),
            has_api_key: api_keys::has_api_key(p.id),
        })
        .collect()
}

#[tauri::command]
pub fn list_cloud_models() -> &'static [crate::transcription::cloud::catalog::CloudModelInfo] {
    CLOUD_MODELS
}

#[tauri::command]
pub fn set_api_key(provider: String, key: String) -> Result<(), String> {
    api_keys::set_api_key(&provider, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    api_keys::delete_api_key(&provider).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_api_key(provider: String) -> bool {
    api_keys::has_api_key(&provider)
}

#[tauri::command]
pub async fn verify_api_key(
    registry: State<'_, CloudRegistryState>,
    provider: String,
    key: String,
) -> Result<(), String> {
    let p = registry
        .0
        .find(&provider)
        .ok_or_else(|| format!("provider inconnu: {provider}"))?;
    p.verify_api_key(&key).await.map_err(|e| e.to_string())
}

#[derive(Debug, serde::Deserialize)]
pub struct CloudTranscribeArgs {
    pub wav_path: String,
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub custom_vocabulary: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CloudTranscribeResponse {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn cloud_transcribe_wav(
    _app: AppHandle,
    registry: State<'_, CloudRegistryState>,
    args: CloudTranscribeArgs,
) -> Result<CloudTranscribeResponse, String> {
    let provider = registry
        .0
        .find(&args.provider)
        .ok_or_else(|| format!("provider inconnu: {}", args.provider))?;
    let key = api_keys::get_api_key(&args.provider)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("aucune cle API pour {}", args.provider))?;

    let req = TranscribeRequest {
        model: args.model.clone(),
        language: args.language,
        prompt: args.prompt,
        custom_vocabulary: args.custom_vocabulary,
    };

    let wav_path = std::path::PathBuf::from(&args.wav_path);
    let start = std::time::Instant::now();
    let text = provider
        .transcribe(&wav_path, &key, &req)
        .await
        .map_err(|e| e.to_string())?;
    Ok(CloudTranscribeResponse {
        text,
        provider: args.provider,
        model: args.model,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

// Commandes Tauri pour la feature screen context (capture + OCR).

use tauri::{command, AppHandle};

use crate::screen_context::service;

#[command]
pub fn get_screen_context_enabled(app: AppHandle) -> bool {
    service::is_enabled(&app)
}

#[command]
pub fn set_screen_context_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    service::set_enabled(&app, enabled).map_err(|e| e.to_string())
}

#[command]
pub fn get_screen_context_cached(app: AppHandle) -> Option<String> {
    service::cached_text(&app)
}

#[command]
pub fn clear_screen_context(app: AppHandle) {
    service::clear(&app);
}

/// Declenche une capture + OCR synchrone et renvoie le texte. Utile pour
/// l'apercu UI depuis le panneau Enhancement.
#[command]
pub async fn capture_screen_context_preview(app: AppHandle) -> Result<Option<String>, String> {
    let app_inner = app.clone();
    tokio::task::spawn_blocking(move || service::capture_and_ocr(&app_inner))
        .await
        .map_err(|e| format!("join: {e}"))?
        .map_err(|e| e.to_string())
}

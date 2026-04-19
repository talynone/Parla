// Commandes Tauri pour la page Permissions.
//
// Reference VoiceInk Views/PermissionsView.swift : liste de PermissionCards
// avec status dot + bouton action. VoiceInk verifie TCC macOS (micro,
// accessibility, screen recording). Sur Windows les permissions sont :
// - Microphone : peut etre bloque Settings > Privacy > Microphone.
// - Accessibility : pas de TCC, SendInput marche toujours.
// - Screen Recording : pas de TCC, Windows.Media.Ocr fonctionne
//   si un pack de langue OCR est installe.
// - Auto-start : tauri-plugin-autostart.

use serde::Serialize;
use tauri::{command, AppHandle};

#[derive(Debug, Serialize)]
pub struct PermissionStatus {
    pub microphone: PermissionState,
    pub ocr: PermissionState,
    pub autostart: PermissionState,
    pub hotkey: PermissionState,
}

#[derive(Debug, Serialize)]
pub struct PermissionState {
    pub ok: bool,
    pub label: String,
    pub hint: Option<String>,
}

#[command]
pub fn check_permissions(app: AppHandle) -> PermissionStatus {
    // Microphone : on tente de lister les devices audio. Liste vide =
    // micro bloque Privacy ou driver absent.
    let devs = crate::audio::list_input_devices();
    let microphone = if !devs.is_empty() {
        PermissionState {
            ok: true,
            label: format!("{} entree(s) audio detectee(s)", devs.len()),
            hint: None,
        }
    } else {
        PermissionState {
            ok: false,
            label: "Aucun microphone detecte".into(),
            hint: Some(
                "Branche un micro ou verifie Settings > Confidentialite > Microphone."
                    .into(),
            ),
        }
    };

    // OCR : tente de creer un OcrEngine. None si aucun pack langue installe.
    let ocr = match windows::Media::Ocr::OcrEngine::TryCreateFromUserProfileLanguages() {
        Ok(_) => PermissionState {
            ok: true,
            label: "Windows.Media.Ocr disponible".into(),
            hint: None,
        },
        Err(e) => PermissionState {
            ok: false,
            label: "OCR non disponible".into(),
            hint: Some(format!(
                "Ajoute un pack de langue OCR dans Settings > Heure et langue > Langue ({e})"
            )),
        },
    };

    // Autostart : tauri-plugin-autostart expose is_enabled via invoke.
    let autostart = match autostart_enabled(&app) {
        Ok(true) => PermissionState {
            ok: true,
            label: "Auto-demarrage actif".into(),
            hint: None,
        },
        Ok(false) => PermissionState {
            ok: false,
            label: "Auto-demarrage desactive".into(),
            hint: Some("Parla ne se lancera pas au demarrage de Windows.".into()),
        },
        Err(e) => PermissionState {
            ok: false,
            label: "Statut auto-demarrage inconnu".into(),
            hint: Some(e),
        },
    };

    // Hotkey : sur Windows WH_KEYBOARD_LL ne necessite pas de permission
    // speciale. On indique toujours OK.
    let hotkey = PermissionState {
        ok: true,
        label: "Hook clavier actif".into(),
        hint: Some(
            "Par defaut Right Alt. Configure le raccourci dans Settings.".into(),
        ),
    };

    PermissionStatus {
        microphone,
        ocr,
        autostart,
        hotkey,
    }
}

fn autostart_enabled(app: &AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    mgr.is_enabled().map_err(|e| e.to_string())
}

#[command]
pub fn set_autostart_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| e.to_string())
    } else {
        mgr.disable().map_err(|e| e.to_string())
    }
}

#[command]
pub fn open_privacy_microphone(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("ms-settings:privacy-microphone", None::<&str>)
        .map_err(|e| e.to_string())
}

#[command]
pub fn open_language_settings(app: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url("ms-settings:regionlanguage", None::<&str>)
        .map_err(|e| e.to_string())
}

#[command]
pub fn get_recorder_style(app: AppHandle) -> String {
    crate::mini_recorder::get_style(&app).as_str().to_string()
}

const STORE_FILE: &str = "parla.settings.json";
const KEY_ONBOARDING: &str = "onboarding_completed";

#[command]
pub fn get_onboarding_completed(app: AppHandle) -> bool {
    use tauri_plugin_store::StoreExt;
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(KEY_ONBOARDING).and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

#[command]
pub fn set_onboarding_completed(app: AppHandle, completed: bool) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(KEY_ONBOARDING, serde_json::Value::Bool(completed));
    store.save().map_err(|e| e.to_string())
}

#[command]
pub fn set_recorder_style(app: AppHandle, style: String) -> Result<(), String> {
    let s = crate::mini_recorder::RecorderStyle::parse(&style);
    crate::mini_recorder::set_style(&app, s).map_err(|e| e.to_string())?;
    // Re-positionne la fenetre si ouverte.
    if let Some(win) = tauri::Manager::get_webview_window(&app, crate::mini_recorder::LABEL) {
        crate::mini_recorder::ensure_open(&app);
        let _ = win.show();
    }
    Ok(())
}

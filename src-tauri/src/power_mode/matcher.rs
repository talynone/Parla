// Resolution d'une config Power Mode pour un contexte donne.
//
// Reference VoiceInk : ActiveWindowService.swift + PowerModeManager.
// Priorite : URL > App > Default.
// - URL match : substring apres clean_for_match.
// - App match : egalite exacte sur exe_name (VoiceInk utilise bundleIdentifier).
// - Default : premier config avec is_default=true et is_enabled=true.

use super::active_window::ActiveWindow;
use super::browser_url::clean_for_match;
use super::config::PowerModeConfig;

/// Trouve la meilleure config pour la fenetre active + URL optionnelle.
pub fn resolve<'a>(
    configs: &'a [PowerModeConfig],
    active: &ActiveWindow,
    url: Option<&str>,
) -> Option<&'a PowerModeConfig> {
    // URL first.
    if let Some(u) = url {
        let clean = clean_for_match(u);
        if !clean.is_empty() {
            if let Some(hit) = configs.iter().find(|c| {
                c.is_enabled
                    && c.url_triggers
                        .iter()
                        .any(|t| !t.url.is_empty() && clean.contains(&clean_for_match(&t.url)))
            }) {
                return Some(hit);
            }
        }
    }

    // App match.
    let exe = active.exe_name.as_str();
    if !exe.is_empty() {
        if let Some(hit) = configs.iter().find(|c| {
            c.is_enabled
                && c.app_triggers
                    .iter()
                    .any(|t| t.exe_name.eq_ignore_ascii_case(exe))
        }) {
            return Some(hit);
        }
    }

    // Default.
    configs.iter().find(|c| c.is_enabled && c.is_default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::power_mode::config::{AppTrigger, UrlTrigger};

    fn dummy_active(exe: &str) -> ActiveWindow {
        ActiveWindow {
            hwnd: 0,
            pid: 0,
            title: String::new(),
            exe_path: std::path::PathBuf::from(format!("{exe}.exe")),
            exe_name: exe.into(),
        }
    }

    fn url_cfg(id: &str, url: &str) -> PowerModeConfig {
        let mut c = PowerModeConfig::new(id.into(), "".into());
        c.id = id.into();
        c.url_triggers = vec![UrlTrigger {
            id: "t".into(),
            url: url.into(),
        }];
        c
    }

    fn app_cfg(id: &str, exe: &str) -> PowerModeConfig {
        let mut c = PowerModeConfig::new(id.into(), "".into());
        c.id = id.into();
        c.app_triggers = vec![AppTrigger {
            id: "t".into(),
            exe_name: exe.into(),
            app_name: exe.into(),
        }];
        c
    }

    fn default_cfg(id: &str) -> PowerModeConfig {
        let mut c = PowerModeConfig::new(id.into(), "".into());
        c.id = id.into();
        c.is_default = true;
        c
    }

    #[test]
    fn url_priority_over_app() {
        let url = url_cfg("u", "github.com");
        let app = app_cfg("a", "chrome");
        let configs = vec![app.clone(), url.clone()];
        let active = dummy_active("chrome");
        let hit = resolve(&configs, &active, Some("https://github.com/foo")).unwrap();
        assert_eq!(hit.id, "u");
    }

    #[test]
    fn app_when_no_url() {
        let url = url_cfg("u", "github.com");
        let app = app_cfg("a", "chrome");
        let configs = vec![app, url];
        let active = dummy_active("chrome");
        let hit = resolve(&configs, &active, None).unwrap();
        assert_eq!(hit.id, "a");
    }

    #[test]
    fn default_fallback() {
        let d = default_cfg("d");
        let configs = vec![d];
        let active = dummy_active("notepad");
        assert_eq!(resolve(&configs, &active, None).unwrap().id, "d");
    }

    #[test]
    fn disabled_ignored() {
        let mut u = url_cfg("u", "github.com");
        u.is_enabled = false;
        let configs = vec![u];
        let active = dummy_active("chrome");
        assert!(resolve(&configs, &active, Some("https://github.com")).is_none());
    }
}

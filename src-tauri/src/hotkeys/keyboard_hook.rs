// Hook clavier bas niveau (WH_KEYBOARD_LL) pour detecter les modifier-only hotkeys.
//
// Cree un thread dedie qui installe le hook et pompe une boucle de messages.
// Les evenements KeyDown / KeyUp des touches surveillees sont pousses dans un
// channel mpsc consomme par le HotkeyManager. Le hook vit jusqu'a la fin du
// processus (pas de mecanisme d'arret propre - pas necessaire en pratique).
//
// Reference VoiceInk : HotkeyManager.swift L133-144 keyCodes modifiers macOS.
// Equivalents Windows (Virtual Key Codes) :
//   rightOption  -> Right Alt   = VK_RMENU    0xA5
//   leftOption   -> Left Alt    = VK_LMENU    0xA4
//   leftControl  -> Left Ctrl   = VK_LCONTROL 0xA2
//   rightControl -> Right Ctrl  = VK_RCONTROL 0xA3
//   rightCommand -> Right Win   = VK_RWIN     0x5C
//   rightShift   -> Right Shift = VK_RSHIFT   0xA1
//   leftShift    -> Left Shift  = VK_LSHIFT   0xA0

use std::sync::mpsc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

#[cfg(windows)]
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HotkeyOption {
    #[default]
    None,
    RightAlt,
    LeftAlt,
    LeftCtrl,
    RightCtrl,
    RightWin,
    RightShift,
    LeftShift,
}

impl HotkeyOption {
    pub fn vk(self) -> Option<u32> {
        match self {
            HotkeyOption::None => None,
            HotkeyOption::RightAlt => Some(0xA5),
            HotkeyOption::LeftAlt => Some(0xA4),
            HotkeyOption::LeftCtrl => Some(0xA2),
            HotkeyOption::RightCtrl => Some(0xA3),
            HotkeyOption::RightWin => Some(0x5C),
            HotkeyOption::RightShift => Some(0xA1),
            HotkeyOption::LeftShift => Some(0xA0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// Le modificateur surveille a ete presse (KeyDown transition).
    Pressed {
        option: HotkeyOption,
        timestamp: Instant,
    },
    /// Le modificateur surveille a ete relache (KeyUp transition).
    Released {
        option: HotkeyOption,
        timestamp: Instant,
    },
    /// La touche Escape a ete pressee (pour le double-tap cancel).
    EscapePressed { timestamp: Instant },
}

struct WatchedKeys {
    primary: HotkeyOption,
    secondary: HotkeyOption,
    key_state: [bool; 256],
    /// Timestamp du dernier LCtrl DOWN recu - utilise pour filtrer les
    /// RAlt synthetiques injectes par AltGr (FR AZERTY, DE QWERTZ).
    /// Voir `ALTGR_GATE` et le filtre dans `handle_key`.
    last_lctrl_down: Option<Instant>,
    /// Indique que le RAlt DOWN courant a ete ignore comme AltGr, donc
    /// il faut egalement ignorer le RAlt UP correspondant (sinon le
    /// HotkeyManager verrait un Released orphelin).
    altgr_in_progress: bool,
}

#[cfg(windows)]
struct HookContext {
    tx: mpsc::Sender<HotkeyEvent>,
    watched: Mutex<WatchedKeys>,
}

#[cfg(windows)]
static HOOK_CONTEXT: OnceLock<HookContext> = OnceLock::new();

/// Installe le hook clavier bas niveau dans un thread dedie.
/// Retourne le receiver des evenements. Le hook vit jusqu'a la fin du processus.
pub fn install_hook(
    primary: HotkeyOption,
    secondary: HotkeyOption,
) -> mpsc::Receiver<HotkeyEvent> {
    let (tx, rx) = mpsc::channel();

    #[cfg(windows)]
    {
        let _ = std::thread::Builder::new()
            .name("parla-hotkey-hook".into())
            .spawn(move || {
                run_hook_thread(tx, primary, secondary);
            });
    }
    #[cfg(not(windows))]
    {
        let _ = (tx, primary, secondary);
    }

    rx
}

#[cfg(windows)]
fn run_hook_thread(
    tx: mpsc::Sender<HotkeyEvent>,
    primary: HotkeyOption,
    secondary: HotkeyOption,
) {
    let _ = HOOK_CONTEXT.set(HookContext {
        tx,
        watched: Mutex::new(WatchedKeys {
            primary,
            secondary,
            key_state: [false; 256],
            last_lctrl_down: None,
            altgr_in_progress: false,
        }),
    });

    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_proc), None, 0) };
    let hook = match hook {
        Ok(h) => h,
        Err(e) => {
            error!("SetWindowsHookExW a echoue: {e:?}");
            return;
        }
    };

    debug!("Hook WH_KEYBOARD_LL installe");

    unsafe {
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            match ret.0 {
                -1 => {
                    error!("GetMessageW a retourne -1");
                    break;
                }
                0 => break,
                _ => {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        if let Err(e) = UnhookWindowsHookEx(hook) {
            warn!("UnhookWindowsHookEx: {e:?}");
        }
    }

    debug!("Thread hook clavier termine");
}

#[cfg(windows)]
unsafe extern "system" fn low_level_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let info = &*(l_param.0 as *const KBDLLHOOKSTRUCT);
        let vk = info.vkCode;
        let message = w_param.0 as u32;
        let is_down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
        let is_up = message == WM_KEYUP || message == WM_SYSKEYUP;

        if is_down || is_up {
            if let Some(ctx) = HOOK_CONTEXT.get() {
                handle_key(ctx, vk, is_down);
            }
        }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Fenetre de detection pour le LCtrl synthetique d'AltGr. Windows emet un
/// LCtrl DOWN 0 a 1 ms avant chaque RAlt DOWN sur clavier AZERTY / QWERTZ,
/// parce qu'historiquement AltGr = Ctrl + Alt. Aucun flag ne distingue ce
/// LCtrl d'un vrai appui utilisateur - seul le gap temporel le trahit. 5 ms
/// est large par rapport au delai observable (~1 ms) et bien en-dessous de
/// ce qu'un humain peut faire a la main (Ctrl + Alt manuel prend au moins
/// 30-50 ms entre les deux touches).
#[cfg(windows)]
const ALTGR_GATE: Duration = Duration::from_millis(5);

#[cfg(windows)]
fn handle_key(ctx: &HookContext, vk: u32, is_down: bool) {
    let now = Instant::now();
    let mut watched = ctx.watched.lock();
    let idx = vk as usize & 0xFF;
    let was_down = watched.key_state[idx];
    watched.key_state[idx] = is_down;

    // Trace le LCtrl DOWN pour le filtre AltGr ci-dessous. Le LCtrl UP
    // synthetique qui termine l'AltGr arrive APRES le RAlt UP, donc on
    // n'a pas besoin de le gerer specialement - le cycle est deja clos
    // cote RAlt via `altgr_in_progress`.
    if vk == 0xA2 && is_down {
        watched.last_lctrl_down = Some(now);
    }

    // Filtre AltGr : sur clavier FR / DE, chaque frappe AltGr + touche
    // (pour @ # { [ ] | \ etc.) genere une paire LCtrl + RAlt synthetique.
    // Sans filtre, le HotkeyManager voit un cycle Pressed/Released RAlt
    // et demarre un enregistrement en Hybrid hands-free a chaque fois.
    if vk == 0xA5 {
        if is_down {
            let is_altgr = watched
                .last_lctrl_down
                .map(|t| now.duration_since(t) < ALTGR_GATE)
                .unwrap_or(false);
            if is_altgr {
                watched.altgr_in_progress = true;
                return;
            }
        } else if watched.altgr_in_progress {
            // Le RAlt DOWN matching etait un AltGr synthetique, on ignore
            // son UP pour que le HotkeyManager ne voie ni Pressed ni
            // Released pour ce cycle.
            watched.altgr_in_progress = false;
            return;
        }
    }

    let primary = watched.primary;
    let secondary = watched.secondary;
    drop(watched);

    // Escape pour double-tap cancel (matching MiniRecorderShortcutManager).
    if vk == 0x1B && is_down && !was_down {
        let _ = ctx.tx.send(HotkeyEvent::EscapePressed { timestamp: now });
        return;
    }

    let Some(option) = match_option(vk, primary, secondary) else {
        return;
    };

    if is_down && !was_down {
        let _ = ctx.tx.send(HotkeyEvent::Pressed {
            option,
            timestamp: now,
        });
    } else if !is_down && was_down {
        let _ = ctx.tx.send(HotkeyEvent::Released {
            option,
            timestamp: now,
        });
    }
}

#[cfg(windows)]
fn match_option(vk: u32, primary: HotkeyOption, secondary: HotkeyOption) -> Option<HotkeyOption> {
    if primary.vk() == Some(vk) {
        return Some(primary);
    }
    if secondary.vk() == Some(vk) {
        return Some(secondary);
    }
    None
}

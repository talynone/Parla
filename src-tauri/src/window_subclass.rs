// Subclasse la WndProc de la fenetre principale pour intercepter certains
// messages avant que DefWindowProc ne les traite.
//
// Pourquoi :
//   Windows assigne a toute fenetre avec une barre de titre standard un
//   menu systeme accessible via Alt+Space (Restore / Move / Size /
//   Minimize / Maximize / Close). Ce comportement est implemente dans
//   DefWindowProc : a la reception de WM_SYSKEYDOWN(VK_SPACE, Alt), la
//   fonction poste un WM_SYSCOMMAND(SC_KEYMENU, ' ') qui ouvre le menu.
//
//   Cote utilisateur ca pose deux problemes :
//     1. Le menu Windows hijack le raccourci Alt+Space pendant que Parla
//        est focus : impossible d'utiliser un launcher type Raycast /
//        PowerToys Run qui a enregistre Alt+Space comme hotkey global.
//     2. Parla entre dans une boucle de message modale (menu trackaging),
//        ce qui peut empecher d'autres apps de prendre le foreground via
//        SetForegroundWindow au meme moment.
//
//   Navigateurs (Chrome, Edge, Firefox) n'ont pas le probleme parce qu'ils
//   utilisent un frame custom et ne delegent pas Alt+Space a DefWindowProc.
//   Les apps Tauri par defaut gardent la barre de titre Windows standard,
//   donc il faut subclasser pour egaler le comportement.
//
// Comment :
//   On installe via SetWindowSubclass (comctl32). Le proc intercepte
//   WM_SYSCOMMAND(SC_KEYMENU, ' ') et renvoie 0 sans appeler
//   DefSubclassProc. Les autres SC_KEYMENU (F10 seul, Alt+lettre pour
//   ouvrir un menu) ne sont pas affectes : F10 a lParam=0, Alt+letter a
//   lParam = le code de la lettre, seul le Space (0x20) est filtre.
//
//   A WM_NCDESTROY on retire la subclasse pour ne pas laisser un callback
//   pendant sur une HWND liberee.

#![cfg(windows)]

use tracing::{debug, warn};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{SC_KEYMENU, WM_NCDESTROY, WM_SYSCOMMAND};

/// Identifiant de la subclasse (arbitraire, unique par proc attache a la
/// meme HWND - ici on n'en installe qu'une seule).
const SUBCLASS_ID: usize = 0xBEEF;

/// Attache la subclasse a la HWND donnee.
pub fn install_main_window_subclass(hwnd: isize) {
    let hwnd = HWND(hwnd as *mut _);
    // Safety : SetWindowSubclass est thread-safe. La HWND provient du
    // runtime Tauri qui garantit sa validite pendant toute la vie de la
    // fenetre. Le proc lui-meme se desabonne a WM_NCDESTROY.
    let ok = unsafe { SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) };
    if ok.as_bool() {
        debug!("Main window subclass installed (Alt+Space suppression)");
    } else {
        warn!("SetWindowSubclass failed on main window");
    }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    umsg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uid_subclass: usize,
    _dwrefdata: usize,
) -> LRESULT {
    // Alt+Space (SC_KEYMENU avec lParam = space) : on mange. F10 passe
    // (lParam = 0). Alt+letter pour les menus passe aussi (lParam = code
    // de la lettre). Les 4 bits bas de wParam sont reserves par Windows,
    // d'ou le mask 0xFFF0.
    if umsg == WM_SYSCOMMAND {
        let cmd = (wparam.0 as u32) & 0xFFF0;
        if cmd == SC_KEYMENU && (lparam.0 as u32) == 0x20 {
            return LRESULT(0);
        }
    }

    // Auto-cleanup : sans ca, la subclasse reste attachee a une HWND
    // libere si la fenetre est detruite sans qu'on ait appele
    // RemoveWindowSubclass explicitement.
    if umsg == WM_NCDESTROY {
        let _ = unsafe { RemoveWindowSubclass(hwnd, Some(subclass_proc), uid_subclass) };
    }

    unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) }
}

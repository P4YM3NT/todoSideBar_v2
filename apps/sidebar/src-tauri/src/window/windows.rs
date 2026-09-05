//! Windows: kein NSPanel-Äquivalent, also die erweiterten Fensterstile direkt setzen.
//!
//! Tauri respektiert die `focus`-Einstellung auf Windows nicht zuverlässig
//! (tauri#7519), deshalb `WS_EX_NOACTIVATE` von Hand.

use tauri::{Runtime, WebviewWindow};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub fn setup<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let hwnd = HWND(window.hwnd()?.0 as _);
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            current
                | WS_EX_NOACTIVATE.0 as isize   // stiehlt keinen Fokus
                | WS_EX_TOOLWINDOW.0 as isize,  // nicht in Alt-Tab / Taskleiste
        );
    }
    Ok(())
}

pub fn show<R: Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window(super::LABEL) {
        let _ = window.show();
    }
}

//! Win32 Low-Level Keyboard and Mouse Hooks with Atomic Input Injection
//!
//! Features:
//! - Loopback prevention: LLKHF_INJECTED & UNIK (0x554E494B) extra-info tag
//! - Universal browser / search bar autocomplete-cancellation trick:
//!   Whenever replacing characters (delete_count > 0), first dispatches synthetic VK_SPACE + VK_BACK
//!   tagged with UNIKEY_EXTRA_INFO (0x554E494B). This collapses inline autocomplete selections in
//!   Chrome/Edge/Omnibox/search bars, yields 2ms, sends the actual delete_count of VK_BACK events,
//!   and then dispatches the replacement UTF-16 KEYEVENTF_UNICODE characters.
//! - Reset boundaries: WH_MOUSE_LL, Space/Enter/Tab, Backspace, Ctrl/Alt shortcuts, numbers, punctuation
//! - Hotkey: Ctrl + Shift mode toggle

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN,
    VK_END, VK_ESCAPE, VK_HOME, VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LSHIFT, VK_NEXT, VK_OEM_1,
    VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS,
    VK_OEM_PERIOD, VK_OEM_PLUS, VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_SHIFT,
    VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_LBUTTONDOWN,
    WM_MBUTTONDOWN, WM_RBUTTONDOWN, WM_SYSKEYDOWN,
};

use crate::engine::{EngineAction, TelexEngine};

/// Magic tag to mark synthetic keystrokes injected by UniRust / UniKey-rs
pub const UNIKEY_EXTRA_INFO: usize = 0x554E494B; // "UNIK"

/// Lock-free global flag indicating whether a browser Omnibox/address bar is currently focused
pub static IS_OMNIBOX_FOCUSED: AtomicBool = AtomicBool::new(false);

static OMNIBOX_THREAD_SPAWNED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static THREAD_AUTOMATION: std::cell::RefCell<Option<IUIAutomation>> = const { std::cell::RefCell::new(None) };
}

/// Inspect currently focused element via UI Automation to check if it's an Omnibox
unsafe fn is_omnibox_element(element: &IUIAutomationElement) -> bool {
    if let Ok(class_name) = element.CurrentClassName() {
        let class_str = class_name.to_string();
        if class_str.contains("OmniboxViewViews")
            || class_str.contains("Omnibox")
            || class_str.contains("urlbar")
            || class_str.contains("AddressBar")
        {
            return true;
        }
    }
    if let Ok(auto_id) = element.CurrentAutomationId() {
        let id_str = auto_id.to_string();
        if id_str.contains("urlbar-input")
            || id_str.contains("address-bar")
            || id_str.contains("addressEdit")
        {
            return true;
        }
    }
    false
}

unsafe extern "system" fn focus_win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    update_omnibox_focus_state();
}

fn update_omnibox_focus_state() {
    THREAD_AUTOMATION.with(|cell| {
        let borrow = cell.borrow();
        if let Some(automation) = borrow.as_ref() {
            unsafe {
                if let Ok(element) = automation.GetFocusedElement() {
                    let is_omni = is_omnibox_element(&element);
                    IS_OMNIBOX_FOCUSED.store(is_omni, Ordering::Relaxed);
                } else {
                    IS_OMNIBOX_FOCUSED.store(false, Ordering::Relaxed);
                }
            }
        } else {
            IS_OMNIBOX_FOCUSED.store(false, Ordering::Relaxed);
        }
    });
}

/// Spawn a lightweight background thread using SetWinEventHook + IUIAutomation
/// to track browser Omnibox focus asynchronously without ever blocking the keyboard hook.
fn spawn_omnibox_detector() {
    if OMNIBOX_THREAD_SPAWNED.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::Builder::new()
        .name("omnibox-detector".into())
        .spawn(|| {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

                let auto: Option<IUIAutomation> = CoCreateInstance(
                    &CUIAutomation,
                    None,
                    CLSCTX_INPROC_SERVER,
                ).ok();

                THREAD_AUTOMATION.with(|cell| {
                    *cell.borrow_mut() = auto;
                });

                update_omnibox_focus_state();

                const EVENT_OBJECT_FOCUS: u32 = 0x8005;
                const WINEVENT_OUTOFCONTEXT: u32 = 0;

                let hook = SetWinEventHook(
                    EVENT_OBJECT_FOCUS,
                    EVENT_OBJECT_FOCUS,
                    None,
                    Some(focus_win_event_proc),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT,
                );

                let mut msg = MSG::default();
                while windows::Win32::UI::WindowsAndMessaging::GetMessageW(&mut msg, None, 0, 0).into() {
                    let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                    windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
                }

                if !hook.is_invalid() {
                    let _ = UnhookWinEvent(hook);
                }
                CoUninitialize();
            }
        })
        .expect("Failed to spawn omnibox detector thread");
}

/// Global hook handles stored as raw isize to implement Send/Sync
static KBD_HOOK: Mutex<Option<isize>> = Mutex::new(None);
static MOUSE_HOOK: Mutex<Option<isize>> = Mutex::new(None);

/// Global tracking of the foreground window to detect focus/window switches
static LAST_FOREGROUND_HWND: Mutex<Option<isize>> = Mutex::new(None);

/// Global Vietnamese mode toggle flag (true = Vietnamese, false = English)
pub static VIETNAMESE_MODE: AtomicBool = AtomicBool::new(true);

/// Global Telex engine protected by Mutex
static ENGINE: Mutex<Option<TelexEngine>> = Mutex::new(None);

/// Track Shift + Ctrl key states for hotkey toggling
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);

/// Optional HWND stored as isize to notify of mode toggles (updates tray icon)
static NOTIFY_HWND: Mutex<Option<isize>> = Mutex::new(None);

pub const WM_APP_MODE_TOGGLED: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 2;

/// Check foreground window change; if focus switched, trigger hard_reset to avoid desync
fn check_foreground_window() {
    unsafe {
        let fg_hwnd = GetForegroundWindow();
        let fg_val = fg_hwnd.0 as isize;
        if fg_val != 0 {
            let mut last_guard = LAST_FOREGROUND_HWND.lock().unwrap();
            if let Some(last) = *last_guard {
                if last != fg_val {
                    *last_guard = Some(fg_val);
                    drop(last_guard);
                    hard_reset_engine();
                    return;
                }
            } else {
                *last_guard = Some(fg_val);
            }
        }
    }
}

/// Initialize and install low-level keyboard and mouse hooks
pub fn install_keyboard_hook(notify_hwnd: Option<HWND>) -> windows::core::Result<()> {
    // Spawn background Omnibox focus detector thread (SetWinEventHook + UI Automation)
    spawn_omnibox_detector();

    {
        let mut hwnd_guard = NOTIFY_HWND.lock().unwrap();
        *hwnd_guard = notify_hwnd.map(|h| h.0 as isize);

        let mut engine_guard = ENGINE.lock().unwrap();
        *engine_guard = Some(TelexEngine::new());
    }

    unsafe {
        let kbd_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            None,
            0,
        )?;
        let mut kbd_guard = KBD_HOOK.lock().unwrap();
        *kbd_guard = Some(kbd_hook.0 as isize);

        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL,
            Some(low_level_mouse_proc),
            None,
            0,
        )?;
        let mut mouse_guard = MOUSE_HOOK.lock().unwrap();
        *mouse_guard = Some(mouse_hook.0 as isize);
    }

    Ok(())
}

/// Uninstall low-level keyboard and mouse hooks
pub fn uninstall_keyboard_hook() {
    let mut kbd_guard = KBD_HOOK.lock().unwrap();
    if let Some(hook_val) = kbd_guard.take() {
        unsafe {
            let hook = HHOOK(hook_val as *mut core::ffi::c_void);
            let _ = UnhookWindowsHookEx(hook);
        }
    }

    let mut mouse_guard = MOUSE_HOOK.lock().unwrap();
    if let Some(hook_val) = mouse_guard.take() {
        unsafe {
            let hook = HHOOK(hook_val as *mut core::ffi::c_void);
            let _ = UnhookWindowsHookEx(hook);
        }
    }
}

/// Hard reset: clear both active syllable buffer AND entire history stack
pub fn hard_reset_engine() {
    if let Ok(mut guard) = ENGINE.lock() {
        if let Some(engine) = guard.as_mut() {
            engine.hard_reset();
        }
    }
}

/// Reset active syllable buffer (without clearing word history)
pub fn reset_engine() {
    if let Ok(mut guard) = ENGINE.lock() {
        if let Some(engine) = guard.as_mut() {
            engine.reset();
        }
    }
}

/// Commit current word buffer into history stack
pub fn commit_word_engine() {
    if let Ok(mut guard) = ENGINE.lock() {
        if let Some(engine) = guard.as_mut() {
            engine.commit_current_word();
        }
    }
}

/// Low-level mouse hook: reset word buffer on mouse clicks (word boundary)
unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let msg = w_param.0 as u32;
        if msg == WM_LBUTTONDOWN || msg == WM_RBUTTONDOWN || msg == WM_MBUTTONDOWN {
            check_foreground_window();
            hard_reset_engine();
        }
    }
    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Check if virtual key is a word separator (Space, punctuation, or digits)
fn is_word_separator_key(vk: VIRTUAL_KEY) -> bool {
    let v = vk.0;
    (0x30..=0x39).contains(&v) // 0-9
        || vk == VK_SPACE
        || vk == VK_OEM_PERIOD
        || vk == VK_OEM_COMMA
        || vk == VK_OEM_1
        || vk == VK_OEM_2
        || vk == VK_OEM_3
        || vk == VK_OEM_4
        || vk == VK_OEM_5
        || vk == VK_OEM_6
        || vk == VK_OEM_7
        || vk == VK_OEM_PLUS
        || vk == VK_OEM_MINUS
}

/// Low-level keyboard hook callback procedure
unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);

    // 1. Loopback prevention: Ignore all synthetic events injected by UniRush (tagged with UNIKEY_EXTRA_INFO).
    // CRITICAL: Checked before any buffer-reset or key-processing logic.
    if kbd_struct.dwExtraInfo == UNIKEY_EXTRA_INFO {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 2. Anti-Desync: Detect foreground window switch (e.g. Alt+Tab, click another window)
    check_foreground_window();

    let msg = w_param.0 as u32;
    let is_key_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
    let vk = VIRTUAL_KEY(kbd_struct.vkCode as u16);

    // 3. Track Ctrl + Shift for Hotkey Toggle (Ctrl + Shift switches Mode)
    if vk == VK_CONTROL || vk == VK_LCONTROL || vk == VK_RCONTROL {
        CTRL_PRESSED.store(is_key_down, Ordering::SeqCst);
    } else if vk == VK_SHIFT || vk == VK_LSHIFT || vk == VK_RSHIFT {
        let was_shift_down = SHIFT_PRESSED.load(Ordering::SeqCst);
        SHIFT_PRESSED.store(is_key_down, Ordering::SeqCst);

        // If Ctrl is held and Shift is pressed down: toggle mode!
        if is_key_down && !was_shift_down && CTRL_PRESSED.load(Ordering::SeqCst) {
            toggle_vietnamese_mode();
            hard_reset_engine();
            return CallNextHookEx(None, n_code, w_param, l_param);
        }
    }

    // Only process key-down events for typing transforms
    if !is_key_down {
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // If Vietnamese mode is disabled, simply track word boundary resets
    if !VIETNAMESE_MODE.load(Ordering::Relaxed) {
        if vk == VK_BACK || vk == VK_SPACE || vk == VK_RETURN {
            hard_reset_engine();
        }
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 4. Anti-Desync Guard: Any Ctrl or Alt key combination (e.g. Ctrl+A, Ctrl+Z, Ctrl+X, Ctrl+V, Ctrl+Backspace)
    // Bulk selections/deletions must immediately wipe buffer AND word history completely
    let ctrl_down = (GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
    let alt_down = (GetKeyState(0x12) as u16 & 0x8000) != 0; // VK_MENU
    if ctrl_down || alt_down {
        hard_reset_engine();
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 5. Anti-Desync Guard: Cursor navigation keys (moving cursor breaks word alignment)
    if vk == VK_LEFT
        || vk == VK_RIGHT
        || vk == VK_UP
        || vk == VK_DOWN
        || vk == VK_HOME
        || vk == VK_END
        || vk == VK_PRIOR
        || vk == VK_NEXT
        || vk == VK_DELETE
        || vk == VK_INSERT
    {
        hard_reset_engine();
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 6. Anti-Desync Guard: Line/focus breaks (Enter, Tab, Escape)
    if vk == VK_RETURN || vk == VK_TAB || vk == VK_ESCAPE {
        hard_reset_engine();
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 7. Smart Backspace Popping
    if vk == VK_BACK {
        if let Ok(mut guard) = ENGINE.lock() {
            if let Some(engine) = guard.as_mut() {
                engine.backspace();
            }
        }
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 8. Word separator keys (Space, standard punctuation, numbers): commit word to history
    if is_word_separator_key(vk) {
        commit_word_engine();
        return CallNextHookEx(None, n_code, w_param, l_param);
    }

    // 9. Convert VK code to unicode character based on Shift/Caps state
    let char_code = vk_to_char(vk);
    if let Some(ch) = char_code {
        if let Ok(mut guard) = ENGINE.lock() {
            if let Some(engine) = guard.as_mut() {
                let action = engine.process_char(ch);
                match action {
                    EngineAction::PassThrough => {
                        return CallNextHookEx(None, n_code, w_param, l_param);
                    }
                    EngineAction::Transform { delete_count, replacement } => {
                        send_replacement_batch(delete_count, &replacement);
                        return LRESULT(1);
                    }
                    EngineAction::Reset => {
                        return CallNextHookEx(None, n_code, w_param, l_param);
                    }
                }
            }
        }
    } else {
        // Any other non-character key -> hard reset to avoid desync
        hard_reset_engine();
    }

    CallNextHookEx(None, n_code, w_param, l_param)
}

/// Convert virtual key to char considering Shift and CapsLock states
unsafe fn vk_to_char(vk: VIRTUAL_KEY) -> Option<char> {
    let vk_val = vk.0;

    // Alphabetic A-Z (0x41 - 0x5A)
    if (0x41..=0x5A).contains(&vk_val) {
        let is_shift = (GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
        let is_caps = (GetKeyState(0x14) as u16 & 0x0001) != 0; // VK_CAPITAL
        let is_upper = is_shift ^ is_caps;

        let base_char = (vk_val as u8) as char;
        return Some(if is_upper {
            base_char
        } else {
            base_char.to_ascii_lowercase()
        });
    }

    None
}

/// Toggle Vietnamese mode and notify GUI / System Tray
pub fn toggle_vietnamese_mode() -> bool {
    let new_mode = !VIETNAMESE_MODE.load(Ordering::SeqCst);
    VIETNAMESE_MODE.store(new_mode, Ordering::SeqCst);

    if let Ok(guard) = NOTIFY_HWND.lock() {
        if let Some(hwnd_val) = *guard {
            unsafe {
                let hwnd = HWND(hwnd_val as *mut core::ffi::c_void);
                let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                    hwnd,
                    WM_APP_MODE_TOGGLED,
                    WPARAM(if new_mode { 1 } else { 0 }),
                    LPARAM(0),
                );
            }
        }
    }

    new_mode
}

/// Build the atomic batch of SendInput events for normal text inputs.
/// Contains delete_count of VK_BACK (down+up) followed by replacement UTF-16 KEYEVENTF_UNICODE (down+up).
/// Guaranteed ZERO VK_SPACE, ZERO thread::sleep, ZERO invisible characters.
pub fn build_normal_replacement_inputs(delete_count: usize, replacement: &str) -> Vec<INPUT> {
    let char_count = replacement.chars().count();
    let total_capacity = delete_count * 2 + char_count * 4;
    let mut batch_inputs: Vec<INPUT> = Vec::with_capacity(total_capacity);

    // 1. VK_BACK events to delete old characters
    for _ in 0..delete_count {
        batch_inputs.push(create_key_input(VK_BACK, false));
        batch_inputs.push(create_key_input(VK_BACK, true));
    }

    // 2. Replacement UTF-16 KEYEVENTF_UNICODE characters
    if !replacement.is_empty() {
        let mut utf16_buf = [0u16; 2];
        for ch in replacement.chars() {
            for &unit in ch.encode_utf16(&mut utf16_buf).iter() {
                batch_inputs.push(create_unicode_codeunit_input(unit, false));
                batch_inputs.push(create_unicode_codeunit_input(unit, true));
            }
        }
    }

    batch_inputs
}

/// Build the Omnibox autocomplete-cancellation inputs (VK_SPACE down/up + VK_BACK down/up)
pub fn build_omnibox_cancellation_inputs() -> [INPUT; 4] {
    [
        create_key_input(VK_SPACE, false),
        create_key_input(VK_SPACE, true),
        create_key_input(VK_BACK, false),
        create_key_input(VK_BACK, true),
    ]
}

/// Inject Backspaces and replacement Unicode characters.
/// - If IS_OMNIBOX_FOCUSED is true: applies the VK_SPACE + VK_BACK autocomplete-cancellation workaround
///   for browser address bars (Chrome/Edge/Brave/Firefox/etc.) where inline autocomplete highlights text.
/// - For ALL normal text inputs (web <input>/<textarea>, Monkeytype/typing tests, editors, IDEs, chat boxes):
///   dispatches VK_BACK events and UTF-16 KEYEVENTF_UNICODE events together in a single atomic SendInput batch
///   with zero VK_SPACE, zero thread::sleep, and zero invisible characters, guaranteeing clean pre-composed
///   NFC Unicode output with zero LLM token bloat and no race conditions.
fn send_replacement_batch(delete_count: usize, replacement: &str) {
    let is_omnibox = IS_OMNIBOX_FOCUSED.load(Ordering::Relaxed);

    if is_omnibox && delete_count > 0 {
        // 1. Omnibox autocomplete-cancellation trick:
        // Synthetic VK_SPACE (down + up) followed by synthetic VK_BACK (down + up)
        // collapses inline autocomplete selections in browser Omniboxes.
        let cancel_inputs = build_omnibox_cancellation_inputs();
        unsafe {
            SendInput(&cancel_inputs, std::mem::size_of::<INPUT>() as i32);
        }

        // Brief yield so target address bar cancels its inline autocomplete selection
        std::thread::sleep(std::time::Duration::from_millis(2));

        // 2. Dispatch VK_BACK events
        let mut back_inputs: Vec<INPUT> = Vec::with_capacity(delete_count * 2);
        for _ in 0..delete_count {
            back_inputs.push(create_key_input(VK_BACK, false));
            back_inputs.push(create_key_input(VK_BACK, true));
        }
        unsafe {
            SendInput(&back_inputs, std::mem::size_of::<INPUT>() as i32);
        }

        // Brief yield before replacement characters
        std::thread::sleep(std::time::Duration::from_millis(1));

        // 3. Inject replacement UTF-16 KEYEVENTF_UNICODE characters
        if !replacement.is_empty() {
            let mut unicode_inputs: Vec<INPUT> = Vec::with_capacity(replacement.chars().count() * 2);
            let mut utf16_buf = [0u16; 2];
            for ch in replacement.chars() {
                for &unit in ch.encode_utf16(&mut utf16_buf).iter() {
                    unicode_inputs.push(create_unicode_codeunit_input(unit, false));
                    unicode_inputs.push(create_unicode_codeunit_input(unit, true));
                }
            }
            unsafe {
                SendInput(&unicode_inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    } else {
        // Normal text fields (web inputs, typing tests, editors, IDEs, chat boxes):
        // Single atomic batch of VK_BACKs followed immediately by UTF-16 KEYEVENTF_UNICODE
        // Zero VK_SPACE, zero thread::sleep, zero race conditions!
        let batch_inputs = build_normal_replacement_inputs(delete_count, replacement);
        if !batch_inputs.is_empty() {
            unsafe {
                SendInput(&batch_inputs, std::mem::size_of::<INPUT>() as i32);
            }
        }
    }
}

fn create_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let mut flags = KEYBD_EVENT_FLAGS(0);
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: UNIKEY_EXTRA_INFO,
            },
        },
    }
}

fn create_unicode_codeunit_input(unit: u16, key_up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if key_up {
        flags |= KEYEVENTF_KEYUP;
    }

    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: UNIKEY_EXTRA_INFO,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_inputs_zero_space_and_atomic() {
        // Test replacing 2 characters with "việt"
        let inputs = build_normal_replacement_inputs(2, "việt");

        // 2 backspaces * 2 (down+up) + 4 chars * 2 (down+up) = 12 inputs
        assert_eq!(inputs.len(), 12);

        // Verify zero VK_SPACE
        for inp in &inputs {
            let ki = unsafe { inp.Anonymous.ki };
            assert_ne!(
                ki.wVk, VK_SPACE,
                "Normal text replacement batch must NEVER contain VK_SPACE!"
            );
            assert_eq!(
                ki.dwExtraInfo, UNIKEY_EXTRA_INFO,
                "All synthetic inputs must be tagged with UNIKEY_EXTRA_INFO"
            );
        }

        // Verify backspaces
        for i in 0..4 {
            let ki = unsafe { inputs[i].Anonymous.ki };
            assert_eq!(ki.wVk, VK_BACK);
            if i % 2 == 0 {
                assert_eq!(ki.dwFlags, KEYBD_EVENT_FLAGS(0));
            } else {
                assert_eq!(ki.dwFlags, KEYEVENTF_KEYUP);
            }
        }

        // Verify unicode code units are clean NFC and have no zero-width spaces or combining marks
        for i in 4..inputs.len() {
            let ki = unsafe { inputs[i].Anonymous.ki };
            assert_eq!(ki.wVk, VIRTUAL_KEY(0));
            let unit = ki.wScan;
            assert_ne!(unit, 0x200B, "Must not contain U+200B zero-width space");
            assert_ne!(unit, 0x2060, "Must not contain U+2060 word joiner");
            assert_ne!(unit, 0xFEFF, "Must not contain U+FEFF zero-width no-break space");
            assert!(
                !(0x0300..=0x036F).contains(&unit),
                "Must not contain decomposed combining mark U+{:04X}",
                unit
            );
        }
    }

    #[test]
    fn test_omnibox_cancellation_inputs() {
        let cancel = build_omnibox_cancellation_inputs();
        assert_eq!(cancel.len(), 4);

        // 1. VK_SPACE down
        let ki0 = unsafe { cancel[0].Anonymous.ki };
        assert_eq!(ki0.wVk, VK_SPACE);
        assert_eq!(ki0.dwFlags, KEYBD_EVENT_FLAGS(0));
        assert_eq!(ki0.dwExtraInfo, UNIKEY_EXTRA_INFO);

        // 2. VK_SPACE up
        let ki1 = unsafe { cancel[1].Anonymous.ki };
        assert_eq!(ki1.wVk, VK_SPACE);
        assert_eq!(ki1.dwFlags, KEYEVENTF_KEYUP);

        // 3. VK_BACK down
        let ki2 = unsafe { cancel[2].Anonymous.ki };
        assert_eq!(ki2.wVk, VK_BACK);
        assert_eq!(ki2.dwFlags, KEYBD_EVENT_FLAGS(0));

        // 4. VK_BACK up
        let ki3 = unsafe { cancel[3].Anonymous.ki };
        assert_eq!(ki3.wVk, VK_BACK);
        assert_eq!(ki3.dwFlags, KEYEVENTF_KEYUP);
    }

    #[test]
    fn test_corpus_replacement_batching_zero_space() {
        let test_words = [
            ("tieengs", "tiếng"),
            ("vieetj", "việt"),
            ("truongwf", "trường"),
            ("nguoiwf", "người"),
            ("suawr", "sửa"),
            ("giuawx", "giữa"),
            ("hoaf", "hoà"),
            ("thuyr", "thuỷ"),
        ];

        for (typed, expected) in test_words {
            let mut engine = TelexEngine::new();
            let mut rendered = String::new();

            for ch in typed.chars() {
                let action = engine.process_char(ch);
                match action {
                    EngineAction::PassThrough => rendered.push(ch),
                    EngineAction::Transform { delete_count, replacement } => {
                        // Inspect the actual inputs that would be dispatched to SendInput
                        let inputs = build_normal_replacement_inputs(delete_count, &replacement);

                        for inp in &inputs {
                            let ki = unsafe { inp.Anonymous.ki };
                            assert_ne!(
                                ki.wVk, VK_SPACE,
                                "Premature VK_SPACE found while typing '{}' -> '{}'!",
                                typed, expected
                            );
                            assert_eq!(ki.dwExtraInfo, UNIKEY_EXTRA_INFO);
                        }

                        for _ in 0..delete_count {
                            rendered.pop();
                        }
                        rendered.push_str(&replacement);
                    }
                    EngineAction::Reset => rendered.push(ch),
                }
            }
            assert_eq!(rendered, expected);
        }
    }
}

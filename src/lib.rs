//! UniRust — Pure Rust Vietnamese Input Method Library

pub mod engine;
pub mod hook;
pub mod platform {
    pub mod hook {
        pub use crate::hook::*;
    }
}
pub mod tray;

use windows::core::w;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, KillTimer, MessageBoxW,
    PostQuitMessage, RegisterClassW, SetTimer, TranslateMessage, MSG, WINDOW_EX_STYLE, WINDOW_STYLE,
    WNDCLASSW, MB_ICONINFORMATION, MB_OK, WM_DESTROY, WM_TIMER,
};

use hook::{install_keyboard_hook, uninstall_keyboard_hook};
use tray::{TrayIcon, WM_APP_TRAY_MSG};

const CLASS_NAME: &windows::core::PCWSTR = &w!("UniRustMainWnd");
const MUTEX_NAME: &windows::core::PCWSTR = &w!("Global\\UniRust_SingleInstance");

const TRIM_TIMER_ID: usize = 1001;
const TRIM_DELAY_MS: u32 = 2500;

// Store TrayIcon pointer in window user data
static mut GLOBAL_TRAY: Option<TrayIcon> = None;

pub fn run() -> windows::core::Result<()> {
    // 1. Single-Instance Guard: Named Mutex
    unsafe {
        let _mutex = CreateMutexW(None, true, *MUTEX_NAME)?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            MessageBoxW(
                None,
                w!("UniRust is already running.\nCheck the system tray icon at the bottom-right."),
                w!("UniRust"),
                MB_OK | MB_ICONINFORMATION,
            );
            return Ok(());
        }
    }

    // 2. Register hidden window class
    unsafe {
        let hinstance = HINSTANCE(GetModuleHandleW(None)?.0);

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: *CLASS_NAME,
            ..Default::default()
        };

        let atom = RegisterClassW(&wnd_class);
        if atom == 0 {
            return Err(windows::core::Error::from_win32());
        }

        // 3. Create hidden window
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            *CLASS_NAME,
            w!("UniRust Background Window"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            hinstance,
            None,
        )?;

        // 4. Initialize System Tray Icon
        let tray = TrayIcon::new(hwnd)?;
        GLOBAL_TRAY = Some(tray);

        // 5. One-Shot Working Set Trim Timer: releases initial startup pages strictly once after 2.5s
        SetTimer(hwnd, TRIM_TIMER_ID, TRIM_DELAY_MS, None);

        // 6. Install Low-Level Keyboard and Mouse Hooks
        install_keyboard_hook(Some(hwnd))?;

        // 7. Run Win32 Message Pump
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 7. Cleanup
        uninstall_keyboard_hook();
        #[allow(static_mut_refs)]
        {
            GLOBAL_TRAY = None;
        }
    }

    Ok(())
}

/// Window Procedure for handling tray messages and mode toggle events
unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    #[allow(static_mut_refs)]
    match msg {
        WM_APP_TRAY_MSG => {
            if let Some(tray) = GLOBAL_TRAY.as_mut() {
                return tray.handle_tray_message(w_param, l_param);
            }
            LRESULT(0)
        }
        hook::WM_APP_MODE_TOGGLED => {
            // Mode changed via hotkey (Ctrl + Shift) -> update tray icon
            let is_viet = w_param.0 != 0;
            if let Some(tray) = GLOBAL_TRAY.as_mut() {
                tray.update_mode(is_viet);
            }
            LRESULT(0)
        }
        WM_TIMER => {
            if w_param.0 == TRIM_TIMER_ID {
                let _ = KillTimer(hwnd, TRIM_TIMER_ID);
                // One-shot working set trim: frees startup pages from kernel32.dll (zero psapi dependency)
                unsafe {
                    let _ = windows::Win32::System::Memory::SetProcessWorkingSetSizeEx(
                        windows::Win32::System::Threading::GetCurrentProcess(),
                        usize::MAX,
                        usize::MAX,
                        windows::Win32::System::Memory::SETPROCESSWORKINGSETSIZEEX_FLAGS(0),
                    );
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

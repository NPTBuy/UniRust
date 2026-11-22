//! System Tray Icon and Context Menu using Windows-rs

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC,
    DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, SelectObject, SetBkMode, SetTextColor,
    DT_CENTER, DT_SINGLELINE, DT_VCENTER, FW_BOLD, TRANSPARENT,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, DestroyIcon, DestroyMenu, GetCursorPos,
    PostQuitMessage, SetForegroundWindow, TrackPopupMenu, HICON, ICONINFO, MF_CHECKED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON,
};

use crate::hook::{toggle_vietnamese_mode, VIETNAMESE_MODE};

pub const WM_APP_TRAY_MSG: u32 = windows::Win32::UI::WindowsAndMessaging::WM_APP + 1;

pub const IDM_TOGGLE_MODE: usize = 1001;
pub const IDM_EXIT: usize = 1002;

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
    hicon_v: HICON,
    hicon_e: HICON,
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> windows::core::Result<Self> {
        let hicon_v = create_letter_icon('V', 0x002020DC)?; // Red background (BGR: 0xDC2020)
        let hicon_e = create_letter_icon('E', 0x00DC7020)?; // Blue background (BGR: 0x2070DC)

        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_APP_TRAY_MSG;
        nid.hIcon = hicon_v;

        let tip = "UniRust - Vietnamese (Telex)\0".encode_utf16().collect::<Vec<u16>>();
        let len = tip.len().min(nid.szTip.len());
        nid.szTip[..len].copy_from_slice(&tip[..len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }

        Ok(Self {
            hwnd,
            nid,
            hicon_v,
            hicon_e,
        })
    }

    /// Update tray icon based on current mode
    pub fn update_mode(&mut self, is_viet: bool) {
        self.nid.hIcon = if is_viet { self.hicon_v } else { self.hicon_e };

        let tip_str = if is_viet {
            "UniRust - Vietnamese (Telex)\0"
        } else {
            "UniRust - English\0"
        };
        let tip = tip_str.encode_utf16().collect::<Vec<u16>>();
        let len = tip.len().min(self.nid.szTip.len());
        self.nid.szTip = [0; 128];
        self.nid.szTip[..len].copy_from_slice(&tip[..len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &self.nid);
        }
    }

    /// Handle tray callback messages (clicks, right clicks)
    pub fn handle_tray_message(&mut self, _w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let msg = l_param.0 as u32;

        match msg {
            windows::Win32::UI::WindowsAndMessaging::WM_RBUTTONUP => {
                // Show right-click context menu
                self.show_context_menu();
                LRESULT(0)
            }
            windows::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP => {
                // Toggle Vietnamese / English mode directly
                let is_viet = toggle_vietnamese_mode();
                self.update_mode(is_viet);
                LRESULT(0)
            }
            _ => LRESULT(0),
        }
    }

    /// Display context menu at mouse cursor position
    fn show_context_menu(&mut self) {
        unsafe {
            let hmenu = match CreatePopupMenu() {
                Ok(menu) => menu,
                Err(_) => return,
            };

            let is_viet = VIETNAMESE_MODE.load(std::sync::atomic::Ordering::Relaxed);
            let check_flag = if is_viet { MF_CHECKED } else { MF_UNCHECKED };

            let _ = AppendMenuW(
                hmenu,
                MF_STRING | check_flag,
                IDM_TOGGLE_MODE,
                w!("Vietnamese Mode (Ctrl + Shift)"),
            );
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_EXIT, w!("Exit UniRust"));

            let mut cursor_pos = POINT::default();
            let _ = GetCursorPos(&mut cursor_pos);

            // Set foreground window before TrackPopupMenu so clicking outside dismisses menu
            let _ = SetForegroundWindow(self.hwnd);

            let cmd = TrackPopupMenu(
                hmenu,
                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | windows::Win32::UI::WindowsAndMessaging::TPM_RETURNCMD,
                cursor_pos.x,
                cursor_pos.y,
                0,
                self.hwnd,
                None,
            );

            let _ = DestroyMenu(hmenu);

            match cmd.0 as usize {
                IDM_TOGGLE_MODE => {
                    let is_viet = toggle_vietnamese_mode();
                    self.update_mode(is_viet);
                }
                IDM_EXIT => {
                    PostQuitMessage(0);
                }
                _ => {}
            }
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
            let _ = DestroyIcon(self.hicon_v);
            let _ = DestroyIcon(self.hicon_e);
        }
    }
}

/// Dynamically generate a 16x16 icon in-memory with a colored background and letter
fn create_letter_icon(letter: char, bgr_color: u32) -> windows::core::Result<HICON> {
    unsafe {
        let screen_dc = GetDC(None);
        let mem_dc = CreateCompatibleDC(screen_dc);

        let width = 16;
        let height = 16;

        let hbm_color = CreateCompatibleBitmap(screen_dc, width, height);
        let hbm_mask = CreateCompatibleBitmap(screen_dc, width, height);

        let old_bmp = SelectObject(mem_dc, hbm_color);

        // 1. Draw colored square background
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(bgr_color));
        let rect = RECT { left: 0, top: 0, right: width, bottom: height };
        FillRect(mem_dc, &rect, brush);
        let _ = DeleteObject(brush);

        // 2. Draw white bold letter centered
        SetBkMode(mem_dc, TRANSPARENT);
        SetTextColor(mem_dc, windows::Win32::Foundation::COLORREF(0x00FFFFFF)); // White

        let font = CreateFontW(
            14, // height
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            windows::Win32::Graphics::Gdi::DEFAULT_CHARSET.0.into(),
            windows::Win32::Graphics::Gdi::OUT_DEFAULT_PRECIS.0.into(),
            windows::Win32::Graphics::Gdi::CLIP_DEFAULT_PRECIS.0.into(),
            windows::Win32::Graphics::Gdi::DEFAULT_QUALITY.0.into(),
            windows::Win32::Graphics::Gdi::DEFAULT_PITCH.0.into(),
            w!("Arial"),
        );
        let old_font = SelectObject(mem_dc, font);

        let mut text_buf = [letter as u16, 0];
        let mut text_rect = rect;
        DrawTextW(mem_dc, &mut text_buf[..1], &mut text_rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        SelectObject(mem_dc, old_font);
        let _ = DeleteObject(font);

        SelectObject(mem_dc, old_bmp);
        let _ = DeleteDC(mem_dc);
        ReleaseDC(None, screen_dc);

        // 3. Create HICON from color & mask bitmaps
        let icon_info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let hicon = CreateIconIndirect(&icon_info)?;

        let _ = DeleteObject(hbm_color);
        let _ = DeleteObject(hbm_mask);

        Ok(hicon)
    }
}

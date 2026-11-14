use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::{
    CreateFontW, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, DEFAULT_CHARSET, DEFAULT_PITCH,
    FF_DONTCARE, HFONT, OUT_DEFAULT_PRECIS,
};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconFromResourceEx, MessageBoxW, SetWindowTextW, HICON, MB_ICONERROR, MB_ICONWARNING,
    MB_OK, SW_SHOWNORMAL,
};

use crate::consts::CF_UNICODETEXT;

pub(crate) fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) unsafe fn make_font(size_pt: f32, dpi_scale: f32, weight: u32, face: &str) -> HFONT {
    let face_w = wide(face);
    CreateFontW(
        -(size_pt * 96.0 / 72.0 * dpi_scale).round() as i32,
        0,
        0,
        0,
        weight as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        CLEARTYPE_QUALITY as u32,
        (DEFAULT_PITCH | FF_DONTCARE) as u32,
        face_w.as_ptr(),
    )
}

pub(crate) unsafe fn set_text(hwnd: HWND, text: &str) {
    let text_w = wide(text);
    SetWindowTextW(hwnd, text_w.as_ptr());
}

pub(crate) unsafe fn warn(owner: HWND, text: &str) {
    let text_w = wide(text);
    let caption = wide("提示");
    MessageBoxW(
        owner,
        text_w.as_ptr(),
        caption.as_ptr(),
        MB_OK | MB_ICONWARNING,
    );
}

pub(crate) unsafe fn error(owner: HWND, text: &str) {
    let text_w = wide(text);
    let caption = wide("错误");
    MessageBoxW(owner, text_w.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONERROR);
}

pub(crate) unsafe fn open_url(url: &str) {
    let op = wide("open");
    let url_w = wide(url);
    let result = ShellExecuteW(
        null_mut(),
        op.as_ptr(),
        url_w.as_ptr(),
        null(),
        null(),
        SW_SHOWNORMAL,
    );
    if result as isize <= 32 {
        error(null_mut(), "无法打开链接。");
    }
}

pub(crate) fn icon_from_ico(bytes: &[u8], want: i32) -> HICON {
    unsafe {
        if bytes.len() < 6 {
            return null_mut();
        }
        let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let mut best: Option<(i32, usize, usize)> = None;
        for i in 0..count {
            let entry = 6 + i * 16;
            if entry + 16 > bytes.len() {
                break;
            }
            let e = &bytes[entry..entry + 16];
            let w = if e[0] == 0 { 256 } else { e[0] as i32 };
            let h = if e[1] == 0 { 256 } else { e[1] as i32 };
            let size = u32::from_le_bytes([e[8], e[9], e[10], e[11]]) as usize;
            let offset = u32::from_le_bytes([e[12], e[13], e[14], e[15]]) as usize;
            if offset + size > bytes.len() {
                continue;
            }
            let score = (w - want).abs() + (h - want).abs();
            if best.map_or(true, |(s, _, _)| score < s) {
                best = Some((score, offset, size));
            }
        }
        match best {
            Some((_, offset, size)) => CreateIconFromResourceEx(
                bytes.as_ptr().add(offset),
                size as u32,
                1,
                0x0003_0000,
                0,
                0,
                0,
            ),
            None => null_mut(),
        }
    }
}

pub(crate) fn clipboard_set(owner: HWND, text: &str) -> bool {
    unsafe {
        if OpenClipboard(owner) == 0 {
            return false;
        }
        EmptyClipboard();

        let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = units.len() * std::mem::size_of::<u16>();
        let handle = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if handle.is_null() {
            CloseClipboard();
            return false;
        }
        let ptr = GlobalLock(handle) as *mut u16;
        if ptr.is_null() {
            CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(units.as_ptr(), ptr, units.len());
        GlobalUnlock(handle);

        if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
            CloseClipboard();
            return false;
        }
        CloseClipboard();
        true
    }
}

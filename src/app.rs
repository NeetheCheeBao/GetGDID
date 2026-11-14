use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{GetLastError, COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateSolidBrush, FillRect, GetDC, GetSysColor, GetTextExtentPoint32W, InvalidateRect,
    ReleaseDC, SelectObject, SetBkColor, SetBkMode, SetTextColor, COLOR_BTNFACE, COLOR_WINDOW,
    COLOR_WINDOWTEXT, FW_BOLD, FW_NORMAL, HBRUSH, HDC, HFONT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
    GetWindowLongPtrW, LoadCursorW, MoveWindow, PostQuitMessage, SendMessageW, SetCursor,
    WS_CHILD, WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW,
    WS_TABSTOP, WS_VISIBLE, GWLP_USERDATA, HMENU, IDC_HAND, MINMAXINFO, WM_CLOSE, WM_COMMAND,
    WM_CTLCOLORSTATIC, WM_DESTROY, WM_ERASEBKGND, WM_GETMINMAXINFO, WM_SETCURSOR, WM_SETFONT,
    WM_SIZE,
};

use crate::consts::*;
use crate::gdid::read_lid;
use crate::win32::{clipboard_set, error, make_font, open_url, set_text, warn, wide};

pub(crate) struct App {
    hwnd: HWND,
    scale: f32,

    header: HWND,
    status: HWND,
    hex_label: HWND,
    hex_edit: HWND,
    hex_copy: HWND,
    ms_label: HWND,
    ms_edit: HWND,
    ms_copy: HWND,
    btn_refresh: HWND,
    btn_copy_both: HWND,
    btn_copy_src: HWND,
    btn_exit: HWND,

    font_ui: HFONT,
    font_bold: HFONT,
    font_mono: HFONT,

    bg_brush: HBRUSH,
    edit_brush: HBRUSH,
    status_color: COLORREF,

    raw_hex: String,
    ms_fmt: String,
}

impl App {
    pub(crate) unsafe fn new(hwnd: HWND) -> App {
        let scale = GetDpiForSystem().max(96) as f32 / 96.0;

        App {
            hwnd,
            scale,
            header: null_mut(),
            status: null_mut(),
            hex_label: null_mut(),
            hex_edit: null_mut(),
            hex_copy: null_mut(),
            ms_label: null_mut(),
            ms_edit: null_mut(),
            ms_copy: null_mut(),
            btn_refresh: null_mut(),
            btn_copy_both: null_mut(),
            btn_copy_src: null_mut(),
            btn_exit: null_mut(),
            font_ui: make_font(9.0, scale, FW_NORMAL, "Microsoft YaHei UI"),
            font_bold: make_font(13.0, scale, FW_BOLD, "Microsoft YaHei UI"),
            font_mono: make_font(11.0, scale, FW_NORMAL, "Consolas"),
            bg_brush: CreateSolidBrush(GetSysColor(COLOR_BTNFACE)),
            edit_brush: CreateSolidBrush(GetSysColor(COLOR_WINDOW)),
            status_color: CLR_STATUS,
            raw_hex: String::new(),
            ms_fmt: String::new(),
        }
    }

    pub(crate) fn scale(&self) -> f32 {
        self.scale
    }

    fn px(&self, value: i32) -> i32 {
        (value as f32 * self.scale).round() as i32
    }

    unsafe fn text_width(&self, text: &str, font: HFONT) -> i32 {
        let hdc = GetDC(self.hwnd);
        let prev = SelectObject(hdc, font);
        let mut size: SIZE = std::mem::zeroed();
        let text_w = wide(text);
        GetTextExtentPoint32W(hdc, text_w.as_ptr(), (text_w.len() - 1) as i32, &mut size);
        SelectObject(hdc, prev);
        ReleaseDC(self.hwnd, hdc);
        size.cx
    }

    unsafe fn child(&self, class: &str, text: &str, style: u32, ex_style: u32, id: u16) -> HWND {
        let class_w = wide(class);
        let text_w = wide(text);
        CreateWindowExW(
            ex_style,
            class_w.as_ptr(),
            text_w.as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            0,
            0,
            0,
            0,
            self.hwnd,
            id as usize as HMENU,
            GetModuleHandleW(null()),
            null(),
        )
    }

    pub(crate) unsafe fn build_children(&mut self) {
        self.header = self.child(
            "STATIC",
            HEADER_TEXT,
            SS_LEFT | SS_CENTERIMAGE | SS_NOTIFY,
            0,
            ID_HEADER,
        );

        self.status = self.child("STATIC", "正在读取…", SS_LEFT, 0, 0);

        self.hex_label = self.child("STATIC", LABEL_HEX, SS_LEFT | SS_CENTERIMAGE, 0, 0);
        self.hex_edit = self.child(
            "EDIT",
            "",
            ES_READONLY | ES_AUTOHSCROLL,
            WS_EX_CLIENTEDGE,
            0,
        );
        self.hex_copy = self.child("BUTTON", "复制", BS_PUSHBUTTON | WS_TABSTOP, 0, ID_COPY_HEX);

        self.ms_label = self.child("STATIC", LABEL_MS, SS_LEFT | SS_CENTERIMAGE, 0, 0);
        self.ms_edit = self.child(
            "EDIT",
            "",
            ES_READONLY | ES_AUTOHSCROLL,
            WS_EX_CLIENTEDGE,
            0,
        );
        self.ms_copy = self.child("BUTTON", "复制", BS_PUSHBUTTON | WS_TABSTOP, 0, ID_COPY_MS);

        self.btn_refresh = self.child("BUTTON", "刷新", BS_PUSHBUTTON | WS_TABSTOP, 0, ID_REFRESH);
        self.btn_copy_both =
            self.child("BUTTON", "全部复制", BS_PUSHBUTTON | WS_TABSTOP, 0, ID_COPY_BOTH);
        self.btn_copy_src = self.child(
            "BUTTON",
            "全部复制(源)",
            BS_PUSHBUTTON | WS_TABSTOP,
            0,
            ID_COPY_SRC,
        );
        self.btn_exit = self.child("BUTTON", "退出", BS_PUSHBUTTON | WS_TABSTOP, 0, ID_EXIT);

        let controls: [(HWND, HFONT); 12] = [
            (self.header, self.font_bold),
            (self.status, self.font_ui),
            (self.hex_label, self.font_ui),
            (self.ms_label, self.font_ui),
            (self.hex_edit, self.font_mono),
            (self.ms_edit, self.font_mono),
            (self.hex_copy, self.font_ui),
            (self.ms_copy, self.font_ui),
            (self.btn_refresh, self.font_ui),
            (self.btn_copy_both, self.font_ui),
            (self.btn_copy_src, self.font_ui),
            (self.btn_exit, self.font_ui),
        ];
        for (hwnd, font) in controls {
            if !hwnd.is_null() {
                SendMessageW(hwnd, WM_SETFONT, font as usize, 1);
            }
        }
    }

    pub(crate) unsafe fn layout(&self) {
        let mut rc: RECT = std::mem::zeroed();
        GetClientRect(self.hwnd, &mut rc);
        let w = rc.right - rc.left;
        let h = rc.bottom - rc.top;

        let mx = self.px(26);
        let gap = self.px(8);
        let header_y = self.px(14);
        let header_h = self.px(30);
        let status_y = self.px(52);
        let status_h = self.px(36);
        let row_h = self.px(26);
        let row1_y = self.px(116);
        let row2_y = row1_y + self.px(38);
        let btn_h = self.px(28);
        let btn_y = h - self.px(14) - btn_h;
        let label_w = self.px(140);
        let copy_w = self.px(72);

        let header_w = (self.text_width(HEADER_TEXT, self.font_bold) + self.px(4))
            .min(w - mx * 2)
            .max(0);
        MoveWindow(self.header, mx, header_y, header_w, header_h, 1);
        MoveWindow(self.status, mx, status_y, (w - mx * 2).max(0), status_h, 1);

        let edit_x = mx + label_w + gap;
        let edit_w = (w - mx - copy_w - gap - edit_x).max(0);

        MoveWindow(self.hex_label, mx, row1_y, label_w, row_h, 1);
        MoveWindow(self.hex_edit, edit_x, row1_y, edit_w, row_h, 1);
        MoveWindow(self.hex_copy, w - mx - copy_w, row1_y, copy_w, row_h, 1);

        MoveWindow(self.ms_label, mx, row2_y, label_w, row_h, 1);
        MoveWindow(self.ms_edit, edit_x, row2_y, edit_w, row_h, 1);
        MoveWindow(self.ms_copy, w - mx - copy_w, row2_y, copy_w, row_h, 1);

        let b_std = self.px(84);
        let b_wide = self.px(100);
        MoveWindow(self.btn_refresh, mx, btn_y, b_std, btn_h, 1);
        MoveWindow(self.btn_copy_both, mx + b_std + gap, btn_y, b_std, btn_h, 1);
        MoveWindow(self.btn_copy_src, mx + (b_std + gap) * 2, btn_y, b_wide, btn_h, 1);
        MoveWindow(self.btn_exit, w - mx - b_std, btn_y, b_std, btn_h, 1);
    }

    unsafe fn set_status(&mut self, text: &str, color: COLORREF) {
        set_text(self.status, text);
        self.status_color = color;
        InvalidateRect(self.status, null(), 1);
    }

    pub(crate) unsafe fn refresh(&mut self) {
        match read_lid() {
            Ok((hex, ms)) => {
                self.raw_hex = hex;
                self.ms_fmt = ms;
                set_text(self.hex_edit, &self.raw_hex);
                set_text(self.ms_edit, &self.ms_fmt);
                self.set_status("GDID 读取成功。", CLR_SUCCESS);
            }
            Err(message) => {
                self.raw_hex.clear();
                self.ms_fmt.clear();
                set_text(self.hex_edit, "");
                set_text(self.ms_edit, "");
                self.set_status(&message, CLR_ERROR);
            }
        }
    }

    unsafe fn copy(&mut self, text: &str, name: &str) {
        if text.is_empty() {
            warn(self.hwnd, &format!("{} 为空，无法复制。", name));
            return;
        }
        if clipboard_set(self.hwnd, text) {
            self.set_status(&format!("已复制「{}」到剪贴板。", name), CLR_INFO);
        } else {
            error(
                self.hwnd,
                &format!("无法写入剪贴板：\n错误码 {}", GetLastError()),
            );
        }
    }

    unsafe fn copy_hex(&mut self) {
        let text = self.raw_hex.clone();
        self.copy(&text, "原始十六进制 (LID)");
    }

    unsafe fn copy_ms(&mut self) {
        let text = self.ms_fmt.clone();
        self.copy(&text, "Microsoft 格式");
    }

    unsafe fn copy_both(&mut self) {
        if self.raw_hex.is_empty() && self.ms_fmt.is_empty() {
            warn(self.hwnd, "当前没有可用的 GDID 值。");
            return;
        }
        let text = format!(
            "原始十六进制 (LID)：{}\nMicrosoft 格式：{}",
            self.raw_hex, self.ms_fmt
        );
        self.copy(&text, "全部内容");
    }

    unsafe fn copy_both_source(&mut self) {
        if self.raw_hex.is_empty() && self.ms_fmt.is_empty() {
            warn(self.hwnd, "当前没有可用的 GDID 值。");
            return;
        }
        let text = format!(
            "=== Global Device Identifier (GDID) ===\nRaw Hex (LID): {}\nMicrosoft Format (g:<decimal>): {}",
            self.raw_hex, self.ms_fmt
        );
        self.copy(&text, "全部复制(源)");
    }

    unsafe fn command(&mut self, id: u16) {
        match id {
            ID_REFRESH => self.refresh(),
            ID_COPY_HEX => self.copy_hex(),
            ID_COPY_MS => self.copy_ms(),
            ID_COPY_BOTH => self.copy_both(),
            ID_COPY_SRC => self.copy_both_source(),
            ID_HEADER => open_url(GITHUB_URL),
            ID_EXIT => {
                DestroyWindow(self.hwnd);
            }
            _ => {}
        }
    }
}

unsafe fn app_ptr(hwnd: HWND) -> Option<&'static mut App> {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    if ptr.is_null() {
        None
    } else {
        Some(&mut *ptr)
    }
}

pub(crate) unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_ERASEBKGND => {
            if let Some(app) = app_ptr(hwnd) {
                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                FillRect(wparam as HDC, &rc, app.bg_brush);
                return 1;
            }
            0
        }
        WM_SIZE => {
            if let Some(app) = app_ptr(hwnd) {
                app.layout();
            }
            0
        }
        WM_SETCURSOR => {
            if let Some(app) = app_ptr(hwnd) {
                if wparam as HWND == app.header {
                    SetCursor(LoadCursorW(null_mut(), IDC_HAND));
                    return 1;
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as u16;
            let code = ((wparam >> 16) & 0xFFFF) as u16;
            if code == 0 {
                if let Some(app) = app_ptr(hwnd) {
                    app.command(id);
                }
            }
            0
        }
        WM_CTLCOLORSTATIC => {
            let hdc = wparam as HDC;
            if let Some(app) = app_ptr(hwnd) {
                let ctl = lparam as HWND;
                if ctl == app.status {
                    SetBkMode(hdc, BK_TRANSPARENT);
                    SetTextColor(hdc, app.status_color);
                    return app.bg_brush as LRESULT;
                }
                if ctl == app.hex_edit || ctl == app.ms_edit {
                    SetBkColor(hdc, GetSysColor(COLOR_WINDOW));
                    SetTextColor(hdc, GetSysColor(COLOR_WINDOWTEXT));
                    return app.edit_brush as LRESULT;
                }
                SetBkMode(hdc, BK_TRANSPARENT);
                return app.bg_brush as LRESULT;
            }
            0
        }
        WM_GETMINMAXINFO => {
            let info = &mut *(lparam as *mut MINMAXINFO);
            let scale = app_ptr(hwnd).map(|a| a.scale()).unwrap_or(1.0);
            let mut rc = RECT {
                left: 0,
                top: 0,
                right: (480.0 * scale) as i32,
                bottom: (240.0 * scale) as i32,
            };
            AdjustWindowRectEx(&mut rc, WS_OVERLAPPEDWINDOW, 0, 0);
            info.ptMinTrackSize.x = rc.right - rc.left;
            info.ptMinTrackSize.y = rc.bottom - rc.top;
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

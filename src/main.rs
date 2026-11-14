#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod consts;
mod gdid;
mod win32;

use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{HINSTANCE, RECT};
use windows_sys::Win32::Graphics::Gdi::{CreateSolidBrush, GetSysColor, UpdateWindow, COLOR_BTNFACE};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::GetDpiForSystem;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AdjustWindowRectEx, CreateAcceleratorTableW, CreateWindowExW, DispatchMessageW, GetMessageW,
    GetSystemMetrics, LoadCursorW, RegisterClassW, SendMessageW, SetProcessDPIAware,
    SetWindowLongPtrW, ShowWindow, TranslateAcceleratorW, TranslateMessage, ACCEL,
    ACCEL_VIRT_FLAGS, CS_HREDRAW, CS_VREDRAW, FNOINVERT, FVIRTKEY, GWLP_USERDATA, IDC_ARROW, MSG,
    SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_SETICON, WNDCLASSW, WS_CLIPCHILDREN, WS_OVERLAPPEDWINDOW,
};

use crate::app::{wndproc, App};
use crate::consts::{APP_TITLE, ICON_BIG_INDEX, ICON_ICO, ICON_SMALL_INDEX, ID_EXIT, VK_ESCAPE};
use crate::win32::{icon_from_ico, wide};

fn main() {
    unsafe {
        SetProcessDPIAware();

        let hinstance: HINSTANCE = GetModuleHandleW(null());
        let bg = CreateSolidBrush(GetSysColor(COLOR_BTNFACE));
        let class_name = wide("GetGDIDMainWindow");

        let icon_big = icon_from_ico(ICON_ICO, 32);
        let icon_small = icon_from_ico(ICON_ICO, 16);

        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.style = CS_HREDRAW | CS_VREDRAW;
        wc.lpfnWndProc = Some(wndproc);
        wc.hInstance = hinstance;
        wc.hIcon = icon_big;
        wc.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
        wc.hbrBackground = bg;
        wc.lpszClassName = class_name.as_ptr();
        RegisterClassW(&wc);

        let scale = GetDpiForSystem().max(96) as f32 / 96.0;
        let style = WS_OVERLAPPEDWINDOW;
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: (520.0 * scale) as i32,
            bottom: (260.0 * scale) as i32,
        };
        AdjustWindowRectEx(&mut rc, style, 0, 0);
        let win_w = rc.right - rc.left;
        let win_h = rc.bottom - rc.top;
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);
        let pos_x = ((screen_w - win_w) / 2).max(0);
        let pos_y = ((screen_h - win_h) / 2).max(0);

        let title = wide(APP_TITLE);
        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            style | WS_CLIPCHILDREN,
            pos_x,
            pos_y,
            win_w,
            win_h,
            null_mut(),
            null_mut(),
            hinstance,
            null(),
        );
        if hwnd.is_null() {
            return;
        }

        SendMessageW(hwnd, WM_SETICON, ICON_BIG_INDEX, icon_big as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL_INDEX, icon_small as isize);

        let app = Box::into_raw(Box::new(App::new(hwnd)));
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, app as isize);
        (*app).build_children();
        (*app).layout();
        (*app).refresh();

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let accel = ACCEL {
            fVirt: (FVIRTKEY | FNOINVERT) as ACCEL_VIRT_FLAGS,
            key: VK_ESCAPE,
            cmd: ID_EXIT,
        };
        let haccel = CreateAcceleratorTableW(&accel, 1);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if haccel.is_null() || TranslateAcceleratorW(hwnd, haccel, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

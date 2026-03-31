use windows::{Win32::{Foundation::{HWND, LPARAM, SIZE, WPARAM}, Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject}, UI::{Controls::WC_BUTTONW, WindowsAndMessaging::{BS_PUSHBUTTON, CreateWindowExW, SW_NORMAL, SendMessageW, ShowWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WM_SETFONT, WS_CHILD, WS_TABSTOP, WS_VISIBLE}}}, core::w};

use crate::win32::{app::FONT_CONTEXT, window::{self, Window}};

pub struct Button {
    hwnd: HWND,
}

impl Button {
    pub fn new(window: HWND) -> Self {
        let hwnd = unsafe {
            let label = w!("Button LONGER TEXT");

            let font_ctx = (&*FONT_CONTEXT.get()).assume_init_ref();

            let hdc = GetDC(None);
            let _ = SelectObject(hdc, font_ctx.default_font.into());
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, label.as_wide(), &mut size);

            ReleaseDC(None, hdc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WC_BUTTONW,
                label,
                WS_TABSTOP | WS_VISIBLE | WS_CHILD | WINDOW_STYLE(BS_PUSHBUTTON.try_into().unwrap()),
                0,
                0,
                size.cx + 32,
                size.cy + 16,
                Some(window),
                None,
                None,
                None,
            ).unwrap();

            
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(font_ctx.default_font_scaled.0 as usize)),
                Some(LPARAM(1))
            );

            //let _ = ShowWindow(hwnd, SW_NORMAL);

            hwnd
        };

        Self {
            hwnd
        }
    }
}
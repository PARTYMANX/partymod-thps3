use std::{cell::RefCell, ffi::c_void, rc::Rc};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, SIZE, WPARAM},
        Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject},
        UI::{
            Controls::WC_BUTTONW,
            WindowsAndMessaging::{
                BS_PUSHBUTTON, CreateWindowExW, HMENU, SendMessageW,
                WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_SETFONT, WS_CHILD, WS_TABSTOP,
                WS_VISIBLE,
            },
        },
    },
    core::w,
};

use crate::win32::app::FONT_CONTEXT;

pub struct Button<T> {
    _hwnd: HWND,
    on_pressed: fn(&mut T),
}

impl<T> Button<T> {
    pub fn new(window: HWND, on_pressed: fn(&mut T)) -> Self {
        let hwnd = unsafe {
            let label = w!("Button LONGER TEXT");

            let font_ctx = (&*FONT_CONTEXT.get()).assume_init_ref();

            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, font_ctx.default_font.into());
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, label.as_wide(), &mut size);
            ReleaseDC(None, hdc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WC_BUTTONW,
                label,
                WS_TABSTOP
                    | WS_VISIBLE
                    | WS_CHILD
                    | WINDOW_STYLE(BS_PUSHBUTTON.try_into().unwrap()),
                0,
                0,
                size.cx + 32,
                size.cy + 16,
                Some(window),
                Some(HMENU(1 as *mut c_void)),
                None,
                None,
            )
            .unwrap();

            // set the font to the correct one
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(font_ctx.default_font_scaled.0 as usize)),
                Some(LPARAM(1)),
            );

            //let _ = ShowWindow(hwnd, SW_NORMAL);

            hwnd
        };

        Self { _hwnd: hwnd, on_pressed }
    }

    pub fn get_hwnd(&self) -> HWND {
        self._hwnd
    }

    pub fn wndproc(
        &self,
        state: &mut T,
        _hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        match msg {
            WM_COMMAND => {
                let id = wparam.0 & 0xffff;

                println!("ID: {}", id);

                if id == 1 {
                    (self.on_pressed)(state);
                    Some(LRESULT(0))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

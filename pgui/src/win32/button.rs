use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, SIZE, WPARAM},
        Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject},
        UI::{
            Controls::WC_BUTTONW,
            WindowsAndMessaging::{
                BS_PUSHBUTTON, CreateWindowExW, HMENU, SendMessageW, WINDOW_EX_STYLE, WINDOW_STYLE,
                WM_COMMAND, WM_SETFONT, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::HSTRING,
};

use crate::win32::app::FONT_CONTEXT;

pub struct Button<T> {
    _hwnd: HWND,
    id: u16,
    _label: HSTRING,
    on_pressed: fn(&mut T),
}

impl<T> Button<T> {
    pub fn new(window: HWND, id: u16, label: String, on_pressed: fn(&mut T)) -> Self {
        let label = HSTRING::from(label);
        //let utf16_label = label.encode_utf16().collect();

        //PCWSTR::from(utf16_label);

        let hwnd = unsafe {
            let font_ctx = (&*FONT_CONTEXT.get()).assume_init_ref();

            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, font_ctx.default_font.into());
            let mut size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &label, &mut size);
            ReleaseDC(None, hdc);

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                WC_BUTTONW,
                &label,
                WS_TABSTOP
                    | WS_VISIBLE
                    | WS_CHILD
                    | WINDOW_STYLE(BS_PUSHBUTTON.try_into().unwrap()),
                0,
                0,
                size.cx + 32,
                size.cy + 16,
                Some(window),
                Some(HMENU(id as *mut c_void)),
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

        Self {
            _hwnd: hwnd,
            id,
            _label: label,
            on_pressed,
        }
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
                let id = (wparam.0 & 0xffff) as u16;

                println!("ID: {}", id);

                if id == self.id {
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

use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, SIZE, WPARAM},
        Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject},
        UI::{
            Controls::WC_BUTTONW,
            WindowsAndMessaging::{
                BS_PUSHBUTTON, CreateWindowExW, HMENU, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos, WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_SETFONT, WS_CHILD, WS_TABSTOP, WS_VISIBLE
            },
        },
    },
    core::HSTRING,
};

use crate::{genarena::GenArenaKey, layout::{Coords, Layout, Position, Size}, win32::app::FONT_CONTEXT};

pub struct Button<T> {
    _hwnd: HWND,
    id: u16,
    _label: HSTRING,
    on_pressed: fn(&mut T),
    layout_node: GenArenaKey,
    coords: Option<Coords>,
}

impl<T> Button<T> {
    pub fn new(window: HWND, id: u16, label: String, on_pressed: fn(&mut T), layout_parent: GenArenaKey, position: Position, layout: &mut Layout) -> Self {
        let label = HSTRING::from(label);
        //let utf16_label = label.encode_utf16().collect();

        //PCWSTR::from(utf16_label);

        let (hwnd, layout_node) = unsafe {
            let font_ctx = (&*FONT_CONTEXT.get()).assume_init_ref();

            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, font_ctx.default_font.into());
            let mut text_size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &label, &mut text_size);
            ReleaseDC(None, hdc);

            let width = match position.w {
                Size::Fill => text_size.cx as u32 + 32,
                Size::Min => text_size.cx as u32 + 32,
                Size::Exact(v) => v,
            };

            let height = match position.h {
                Size::Fill => text_size.cy as u32 + 16,
                Size::Min => text_size.cy as u32 + 16,
                Size::Exact(v) => v,
            };

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
                width as i32,
                height as i32,
                Some(window),
                Some(HMENU(id as *mut c_void)),
                None,
                None,
            )
            .unwrap();

            let layout_node = layout.add_node(
                layout_parent,
                crate::layout::LayoutNodeType::Leaf,
                Position { 
                    w: match position.w {
                        Size::Fill => Size::Fill,
                        Size::Exact(v) => Size::Exact(v),
                        Size::Min => Size::Exact(width),
                    },
                    h: match position.h {
                        Size::Fill => Size::Fill,
                        Size::Exact(v) => Size::Exact(v),
                        Size::Min => Size::Exact(height),
                    },
                    x: position.x,
                    y: position.y,
                    h_padding: position.h_padding,
                    v_padding: position.v_padding,
                }

            );

            // set the font to the correct one
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(font_ctx.default_font_scaled.0 as usize)),
                Some(LPARAM(1)),
            );

            //let _ = ShowWindow(hwnd, SW_NORMAL);

            (hwnd, layout_node)
        };

        Self {
            _hwnd: hwnd,
            id,
            _label: label,
            on_pressed,
            layout_node,
            coords: None,
        }
    }

    pub fn update(
        &mut self,
        layout: &mut Layout,
    ) {
        self.coords = layout.get_node_coords(self.layout_node);

        if let Some(coords) = self.coords {
            unsafe {
                let _ = SetWindowPos(
                    self._hwnd, 
                    None, 
                    coords.x, 
                    coords.y, 
                    coords.w as i32, 
                    coords.h as i32, 
                    SWP_NOZORDER | SWP_NOACTIVATE
                );
            }
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

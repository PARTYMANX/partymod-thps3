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

use crate::{genarena::GenArenaKey, layout::{Coords, Layout, Position, Size}, win32::{font::Fonts}};

pub struct Button<T> {
    _hwnd: HWND,
    id: u16,
    _label: HSTRING,
    on_pressed: Option<fn(&mut T)>,
    layout_node: GenArenaKey,
    coords: Option<Coords>,
    current_scale: f32,
}

impl<T> Button<T> {
    pub fn new(window: HWND, id: u16, label: String, on_pressed: Option<fn(&mut T)>, layout_parent: GenArenaKey, position: Position, layout: &mut Layout, fonts: &Fonts) -> Self {
        let label = HSTRING::from(label);
        //let utf16_label = label.encode_utf16().collect();

        //PCWSTR::from(utf16_label);

        let (hwnd, layout_node) = unsafe {
            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, fonts.default_font.into());
            let mut text_size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &label, &mut text_size);
            ReleaseDC(None, hdc);

            let width = match position.w {
                Size::Fill => text_size.cx as u32 + 32,
                Size::Min => text_size.cx as u32 + 32,
                Size::Exact(v) => v,
            };

            let height = match position.h {
                Size::Fill => fonts.default_font_height as u32 + 16,
                Size::Min => fonts.default_font_height as u32 + 16,
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
                Some(WPARAM(fonts.default_font_scaled.0 as usize)),
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
            current_scale: 1.0,
        }
    }

    pub fn update(&mut self, layout: &mut Layout, fonts: &Fonts, scale: f32) {
        let mut update_position = false;
        let mut update_font = false;

        let new_coords = layout.get_node_coords(self.layout_node);

        if new_coords != self.coords {
            self.coords = new_coords;

            update_position = true;
        }

        if scale != self.current_scale {
            self.current_scale = scale;

            update_position = true;
            update_font = true;
        }

        if update_position {
            if let Some(coords) = self.coords {
                unsafe {
                    let _ = SetWindowPos(
                        self._hwnd, 
                        None, 
                        (coords.x as f32 * scale) as i32, 
                        (coords.y as f32 * scale) as i32, 
                        (coords.w as f32 * scale) as i32, 
                        (coords.h as f32 * scale) as i32, 
                        SWP_NOZORDER | SWP_NOACTIVATE
                    );
                }
            }
        }

        if update_font {
            unsafe {
                SendMessageW(
                    self._hwnd,
                    WM_SETFONT,
                    Some(WPARAM(fonts.default_font_scaled.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    pub fn wndproc_on_pressed(
        &self,
        state: &mut T,
        _hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        if let Some(f) = self.on_pressed {
            (f)(state);
            Some(LRESULT(0))
        } else {
            None
        }
    }
}

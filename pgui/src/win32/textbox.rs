use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, SIZE, WPARAM},
        Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject},
        UI::{
            Controls::WC_EDITW,
            Input::KeyboardAndMouse::EnableWindow,
            WindowsAndMessaging::{
                CreateWindowExW, GetWindowTextLengthW, GetWindowTextW, HMENU, SHOW_WINDOW_CMD,
                SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos, SetWindowTextW,
                ShowWindow, WM_SETFONT, WS_CHILD, WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::HSTRING,
};

use crate::{
    genarena::GenArenaKey,
    layout::{Coords, Layout, Position, Size},
    textbox::TextboxState,
    win32::font::Fonts,
};

pub struct Textbox<T> {
    hwnd: HWND,
    _id: u16,
    current_state: TextboxState,
    on_focused: Option<Box<dyn Fn(&mut T, &String)>>,
    on_unfocused: Option<Box<dyn Fn(&mut T, &String)>>,
    on_changed: Option<Box<dyn Fn(&mut T, &String)>>,
    state_hook: Option<Box<dyn Fn(&T, &mut TextboxState)>>,
    layout_node: GenArenaKey,
    coords: Option<Coords>,
    current_scale: f32,
}

impl<T> Textbox<T> {
    pub fn new(
        window: HWND,
        id: u16,
        initial_state: TextboxState,
        on_focused: Option<Box<dyn Fn(&mut T, &String)>>,
        on_unfocused: Option<Box<dyn Fn(&mut T, &String)>>,
        on_changed: Option<Box<dyn Fn(&mut T, &String)>>,
        state_hook: Option<Box<dyn Fn(&T, &mut TextboxState)>>,
        layout_parent: GenArenaKey,
        layout: &mut Layout,
        fonts: &Fonts,
    ) -> Self {
        let initial_text = HSTRING::from(initial_state.text.clone());

        let (hwnd, layout_node) = unsafe {
            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let position = Self::calc_position(initial_state.position, &initial_text, fonts);

            let width = match position.w {
                Size::Fill => 0,
                Size::Min => 0,
                Size::Exact(v) => v,
            };

            let height = match position.h {
                Size::Fill => 0,
                Size::Min => 0,
                Size::Exact(v) => v,
            };

            let hwnd = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                WC_EDITW,
                &initial_text,
                WS_TABSTOP | WS_VISIBLE | WS_CHILD,
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

            let layout_node =
                layout.add_node(layout_parent, crate::layout::LayoutNodeType::Leaf, position);

            // set the font to the correct one
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(fonts.default_font_scaled.0 as usize)),
                Some(LPARAM(1)),
            );

            let _ = EnableWindow(hwnd, initial_state.enabled);

            (hwnd, layout_node)
        };

        Self {
            hwnd,
            _id: id,
            current_state: initial_state,
            on_focused,
            on_unfocused,
            on_changed,
            state_hook,
            layout_node,
            coords: None,
            current_scale: 1.0,
        }
    }

    fn calc_position(initial_position: Position, placeholder: &HSTRING, fonts: &Fonts) -> Position {
        // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
        let text_size = unsafe {
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, fonts.default_font.into());
            let mut text_size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &placeholder, &mut text_size);
            ReleaseDC(None, hdc);

            text_size
        };

        let width = match initial_position.w {
            Size::Fill => text_size.cx as u32 + 32,
            Size::Min => text_size.cx as u32 + 32,
            Size::Exact(v) => v,
        };

        let height = match initial_position.h {
            Size::Fill => fonts.default_font_height as u32 + 16,
            Size::Min => fonts.default_font_height as u32 + 16,
            Size::Exact(v) => v,
        };

        Position {
            w: match initial_position.w {
                Size::Fill => Size::Fill,
                Size::Exact(v) => Size::Exact(v),
                Size::Min => Size::Exact(width),
            },
            h: match initial_position.h {
                Size::Fill => Size::Fill,
                Size::Exact(v) => Size::Exact(v),
                Size::Min => Size::Exact(height),
            },
            x: initial_position.x,
            y: initial_position.y,
            h_padding: initial_position.h_padding,
            v_padding: initial_position.v_padding,
        }
    }

    pub fn hide(&self, hidden: bool) {
        let show_cmd = if hidden {
            SHOW_WINDOW_CMD(0)
        } else {
            SHOW_WINDOW_CMD(1)
        };

        unsafe {
            let _ = ShowWindow(self.hwnd, show_cmd);
        }
    }

    pub fn compare_hwnd(&self, hwnd: HWND) -> bool {
        self.hwnd == hwnd
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
                        self.hwnd,
                        None,
                        (coords.x as f32 * scale) as i32,
                        (coords.y as f32 * scale) as i32,
                        (coords.w as f32 * scale) as i32,
                        (coords.h as f32 * scale) as i32,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
            }
        }

        if update_font {
            unsafe {
                SendMessageW(
                    self.hwnd,
                    WM_SETFONT,
                    Some(WPARAM(fonts.default_font_scaled.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
        }
    }

    fn get_text_string(&self) -> String {
        // retrieves currently entered text from textbox
        unsafe {
            let len = GetWindowTextLengthW(self.hwnd);
            let mut buf = Vec::new();
            buf.resize(len as usize + 1, 0u16);
            let _ = GetWindowTextW(self.hwnd, &mut buf);
            let hstr = HSTRING::from_wide(&buf);
            // for some reason rust is retaining the null terminator
            // strip it manually
            hstr.to_string_lossy().trim_end_matches("\0").to_string()
        }
    }

    pub fn wndproc_on_focused(
        &mut self,
        state: &mut T,
        _hwnd: HWND,
        _msg: u32,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        if let Some(f) = &self.on_focused {
            self.current_state.text = self.get_text_string();
            (f)(state, &self.current_state.text);
            Some(LRESULT(0))
        } else {
            None
        }
    }

    pub fn wndproc_on_unfocused(
        &mut self,
        state: &mut T,
        _hwnd: HWND,
        _msg: u32,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        if let Some(f) = &self.on_unfocused {
            self.current_state.text = self.get_text_string();
            (f)(state, &self.current_state.text);
            Some(LRESULT(0))
        } else {
            None
        }
    }

    pub fn wndproc_on_changed(
        &mut self,
        state: &mut T,
        _hwnd: HWND,
        _msg: u32,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        if let Some(f) = &self.on_changed {
            self.current_state.text = self.get_text_string();
            (f)(state, &self.current_state.text);
            Some(LRESULT(0))
        } else {
            None
        }
    }

    pub fn do_state_hook(&mut self, state: &T, _layout: &mut Layout, _fonts: &Fonts) -> bool {
        let mut updated = false;

        if let Some(f) = &self.state_hook {
            let old_state = self.current_state.clone();
            (f)(state, &mut self.current_state);

            unsafe {
                if self.current_state.enabled != old_state.enabled {
                    let _ = EnableWindow(self.hwnd, self.current_state.enabled);

                    updated = true;
                }

                // we have to avoid modifying text if it's not different
                // the cursor will move to the start and also, more importantly,
                // it will cause a stack overflow with change events.
                if self.current_state.text != old_state.text {
                    let text = HSTRING::from(self.current_state.text.clone());
                    let _ = SetWindowTextW(self.hwnd, &text);

                    updated = true;
                }
            }

            if self.current_state.position != old_state.position {
                //let position = Self::calc_position(self.current_state.position, &self.placeholder, fonts);
                //layout.set_node_position(self.layout_node, position);
                self.coords = None;

                updated = true;
            }
        }

        updated
    }
}

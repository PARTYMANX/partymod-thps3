use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, SIZE, WPARAM},
        Graphics::Gdi::{GetDC, GetTextExtentPoint32W, ReleaseDC, SelectObject},
        UI::{
            Controls::{BST_CHECKED, BST_UNCHECKED, WC_BUTTONW},
            Input::KeyboardAndMouse::EnableWindow,
            WindowsAndMessaging::{
                BM_SETCHECK, BS_CHECKBOX, CreateWindowExW, HMENU, SHOW_WINDOW_CMD, SWP_NOACTIVATE,
                SWP_NOZORDER, SendMessageW, SetWindowPos, SetWindowTextW, ShowWindow,
                WINDOW_EX_STYLE, WINDOW_STYLE, WM_SETFONT, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::HSTRING,
};

use crate::{
    checkbox::CheckboxState,
    genarena::GenArenaKey,
    layout::{Coords, Layout, Position, Size},
    win32::font::Fonts,
};

pub struct Checkbox<T> {
    hwnd: HWND,
    _id: u16,
    label: HSTRING,
    current_state: CheckboxState,
    on_toggled: Option<Box<dyn Fn(&mut T, bool)>>,
    state_hook: Option<Box<dyn Fn(&T, &mut CheckboxState)>>,
    layout_node: GenArenaKey,
    coords: Option<Coords>,
    current_scale: f32,
}

impl<T> Checkbox<T> {
    pub fn new(
        window: HWND,
        id: u16,
        initial_state: CheckboxState,
        on_toggled: Option<Box<dyn Fn(&mut T, bool)>>,
        state_hook: Option<Box<dyn Fn(&T, &mut CheckboxState)>>,
        layout_parent: GenArenaKey,
        layout: &mut Layout,
        fonts: &Fonts,
    ) -> Self {
        let label = HSTRING::from(initial_state.label.clone());

        let (hwnd, layout_node) = unsafe {
            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let position = Self::calc_position(initial_state.position, &label, fonts);

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
                WINDOW_EX_STYLE::default(),
                WC_BUTTONW,
                &label,
                WS_TABSTOP | WS_VISIBLE | WS_CHILD | WINDOW_STYLE(BS_CHECKBOX as u32),
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
            label,
            current_state: initial_state,
            on_toggled,
            state_hook,
            layout_node,
            coords: None,
            current_scale: 1.0,
        }
    }

    fn calc_position(initial_position: Position, label: &HSTRING, fonts: &Fonts) -> Position {
        // get size of label
        let text_size = unsafe {
            let hdc = GetDC(None);
            let _ = SelectObject(hdc, fonts.default_font.into());
            let mut text_size = SIZE::default();
            let _ = GetTextExtentPoint32W(hdc, &label, &mut text_size);
            ReleaseDC(None, hdc);

            text_size
        };

        let width = match initial_position.w {
            Size::Fill => text_size.cx as u32 + 27,
            Size::Min => text_size.cx as u32 + 27,
            Size::Exact(v) => v,
        };

        let height = match initial_position.h {
            Size::Fill => 16.max(text_size.cy as u32),
            Size::Min => 16.max(text_size.cy as u32),
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

    pub fn wndproc_on_toggled(
        &mut self,
        state: &mut T,
        _hwnd: HWND,
        _msg: u32,
        _wparam: WPARAM,
        _lparam: LPARAM,
    ) -> Option<LRESULT> {
        unsafe {
            //let checked = SendMessageW(self.hwnd, BM_GETCHECK, None, None) == LRESULT(BST_CHECKED.0 as isize);

            self.current_state.checked = !self.current_state.checked;

            if self.current_state.checked {
                SendMessageW(
                    self.hwnd,
                    BM_SETCHECK,
                    Some(WPARAM(BST_CHECKED.0 as usize)),
                    None,
                );
            } else {
                SendMessageW(
                    self.hwnd,
                    BM_SETCHECK,
                    Some(WPARAM(BST_UNCHECKED.0 as usize)),
                    None,
                );
            }
        }

        if let Some(f) = &self.on_toggled {
            (f)(state, self.current_state.checked);
            Some(LRESULT(0))
        } else {
            None
        }
    }

    pub fn do_state_hook(&mut self, state: &T, layout: &mut Layout, fonts: &Fonts) -> bool {
        let mut updated = false;

        if let Some(f) = &self.state_hook {
            let old_state = self.current_state.clone();
            (f)(state, &mut self.current_state);

            unsafe {
                if self.current_state.enabled != old_state.enabled {
                    let _ = EnableWindow(self.hwnd, self.current_state.enabled);

                    updated = true;
                }

                if self.current_state.label != old_state.label {
                    self.label = HSTRING::from(self.current_state.label.clone());
                    let _ = SetWindowTextW(self.hwnd, &self.label);

                    updated = true;
                }

                if self.current_state.checked != old_state.checked {
                    if self.current_state.checked {
                        SendMessageW(
                            self.hwnd,
                            BM_SETCHECK,
                            Some(WPARAM(BST_CHECKED.0 as usize)),
                            None,
                        );
                    } else {
                        SendMessageW(
                            self.hwnd,
                            BM_SETCHECK,
                            Some(WPARAM(BST_UNCHECKED.0 as usize)),
                            None,
                        );
                    }

                    updated = true;
                }
            }

            if self.current_state.position != old_state.position {
                let position = Self::calc_position(self.current_state.position, &self.label, fonts);
                layout.set_node_position(self.layout_node, position);
                self.coords = None;

                updated = true;
            }
        }

        updated
    }
}

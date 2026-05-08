use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Controls::WC_BUTTONW,
            Input::KeyboardAndMouse::EnableWindow,
            WindowsAndMessaging::{
                BS_GROUPBOX, CreateWindowExW, HMENU, SHOW_WINDOW_CMD, SWP_NOACTIVATE, SWP_NOZORDER,
                SendMessageW, SetWindowPos, SetWindowTextW, ShowWindow, WINDOW_EX_STYLE,
                WINDOW_STYLE, WM_SETFONT, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
            },
        },
    },
    core::HSTRING,
};

use crate::{
    genarena::GenArenaKey,
    groupbox::GroupboxState,
    layout::{Coords, Layout, Position, Size},
    win32::font::Fonts,
};

pub struct Groupbox<T> {
    hwnd: HWND,
    _id: u16,
    label: HSTRING,
    current_state: GroupboxState,
    state_hook: Option<Box<dyn Fn(&T, &mut GroupboxState)>>,
    outer_layout_node: GenArenaKey,
    inner_layout_node: GenArenaKey,
    coords: Option<Coords>,
    current_scale: f32,
}

impl<T> Groupbox<T> {
    pub fn new(
        window: HWND,
        id: u16,
        initial_state: GroupboxState,
        state_hook: Option<Box<dyn Fn(&T, &mut GroupboxState)>>,
        layout_parent: GenArenaKey,
        layout: &mut Layout,
        fonts: &Fonts,
    ) -> Self {
        let label = HSTRING::from(initial_state.label.clone());

        let (hwnd, outer_layout_node, inner_layout_node) = unsafe {
            // get size of label (this could probably be moved elsewhere since i assume it'll get reused)
            let position = initial_state.position;

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
                WS_TABSTOP | WS_VISIBLE | WS_CHILD | WINDOW_STYLE(BS_GROUPBOX as u32),
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

            let outer_layout_node = layout.add_node(
                layout_parent,
                crate::layout::LayoutNodeType::Container,
                position,
            );

            let inner_layout_node = layout.add_node(
                outer_layout_node,
                crate::layout::LayoutNodeType::Container,
                Position {
                    w: crate::layout::Size::Fill,
                    h: crate::layout::Size::Fill,
                    x: crate::layout::HorizontalOffset::AlignLeft(0),
                    y: crate::layout::VerticalOffset::AlignTop(0),
                    h_padding: 16,
                    v_padding: 16,
                },
            );

            // set the font to the correct one
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(fonts.default_font_scaled.0 as usize)),
                Some(LPARAM(1)),
            );

            let _ = EnableWindow(hwnd, initial_state.enabled);

            (hwnd, outer_layout_node, inner_layout_node)
        };

        Self {
            hwnd,
            _id: id,
            label,
            current_state: initial_state,
            state_hook,
            inner_layout_node,
            outer_layout_node,
            coords: None,
            current_scale: 1.0,
        }
    }

    pub fn get_layout_node(&self) -> GenArenaKey {
        self.inner_layout_node
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

        // NOTE: known issue: because the inner layout node is Size::Fill,
        // size will overflow unless outer size is exact or fill.
        let new_coords = layout.get_node_coords(self.outer_layout_node);

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
                        ((coords.x + 8) as f32 * scale) as i32,
                        (coords.y as f32 * scale) as i32,
                        ((coords.w - 16) as f32 * scale) as i32,
                        ((coords.h - 8) as f32 * scale) as i32,
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

    pub fn do_state_hook(&mut self, state: &T, layout: &mut Layout, _fonts: &Fonts) -> bool {
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
            }

            if self.current_state.position != old_state.position {
                let position = self.current_state.position;
                layout.set_node_position(self.outer_layout_node, position);
                self.coords = None;

                updated = true;
            }
        }

        updated
    }
}

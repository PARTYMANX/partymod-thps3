use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Controls::{TAB_CONTROL_ITEM_STATE, TCIF_IMAGE, TCIF_TEXT, TCITEMW, TCM_INSERTITEMW, WC_TABCONTROLW},
            Input::KeyboardAndMouse::EnableWindow,
            WindowsAndMessaging::{
                CB_SETCURSEL, CreateWindowExW, HMENU, SHOW_WINDOW_CMD, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos, ShowWindow, WINDOW_EX_STYLE, WM_SETFONT, WS_CHILD, WS_VISIBLE
            },
        },
    },
    core::{HSTRING, PWSTR, w},
};

use crate::{
    genarena::GenArenaKey, layout::{Coords, Layout, Position, Size}, tabs::{Tab, TabsState}, win32::font::Fonts
};

pub struct Tabs<T> {
    hwnd: HWND,
    _id: u16,
    current_state: TabsState,
    state_hook: Option<fn(&T, &mut TabsState)>,
    outer_layout_node: GenArenaKey,
    inner_layout_node: GenArenaKey,
    coords: Option<Coords>,
    current_scale: f32,
}

impl<T> Tabs<T> {
    pub fn new(
        window: HWND,
        id: u16,
        tabs: &Vec<Tab<T>>,
        initial_state: TabsState,
        state_hook: Option<fn(&T, &mut TabsState)>,
        layout_parent: GenArenaKey,
        layout: &mut Layout,
        fonts: &Fonts,
    ) -> Self {
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
                WC_TABCONTROLW,
                w!(""),
                WS_VISIBLE
                    | WS_CHILD,
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

            let outer_layout_node =
                layout.add_node(layout_parent, crate::layout::LayoutNodeType::Container, position);

            //let tab_labels = Vec::new();
            for (i, tab) in tabs.iter().enumerate() {
                let label = HSTRING::from(tab.label.clone());

                let mut item = TCITEMW {
                    mask: TCIF_IMAGE | TCIF_TEXT,
                    dwState: TAB_CONTROL_ITEM_STATE::default(),
                    dwStateMask: TAB_CONTROL_ITEM_STATE::default(),
                    pszText: PWSTR(label.as_ptr() as *mut u16),
                    cchTextMax: 0,
                    iImage: -1,
                    lParam: LPARAM::default(),
                };

                SendMessageW(hwnd, TCM_INSERTITEMW, Some(WPARAM(i)), Some(LPARAM(&raw mut item as isize)));
            }

            let inner_layout_node = layout.add_node(
                outer_layout_node, 
                crate::layout::LayoutNodeType::MultiContainer, 
                Position { 
                    w: crate::layout::Size::Fill, 
                    h: crate::layout::Size::Fill, 
                    x: crate::layout::HorizontalOffset::AlignLeft(0), 
                    y: crate::layout::VerticalOffset::AlignTop(16), 
                    h_padding: 16, 
                    v_padding: 16,
                },
            );

            SendMessageW(hwnd, CB_SETCURSEL, Some(WPARAM(0)), None);

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

    pub fn do_state_hook(&mut self, state: &T, layout: &mut Layout, _fonts: &Fonts) -> bool {
        if let Some(f) = self.state_hook {
            let mut new_state = self.current_state.clone();
            (f)(state, &mut new_state);

            if new_state != self.current_state {
                self.current_state = new_state;

                unsafe {
                    let _ = EnableWindow(self.hwnd, self.current_state.enabled);
                }

                let position = self.current_state.position;
                layout.set_node_position(self.outer_layout_node, position);
                self.coords = None;

                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

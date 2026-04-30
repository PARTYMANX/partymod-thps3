use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, COLOR_WINDOW, EndPaint, FillRect, HBRUSH, InvalidateRect, PAINTSTRUCT},
        UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowRect, MoveWindow, PostQuitMessage, WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_DPICHANGED, WM_PAINT, WS_CAPTION, WS_MINIMIZEBOX, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_SYSMENU, WS_VISIBLE
        }},
    },
    core::{HSTRING, PCWSTR, w},
};

use crate::{genarena::GenArenaKey, layout::{Layout, Position}, win32::{button::Button, font::Fonts}};

pub enum Component<T> {
    Button(Button<T>),
}

pub struct Window<T> {
    _hwnd: HWND,

    scale: f32,
    width: u32,
    height: u32,
    title: HSTRING,

    // TODO: list of components (or more likely, a generational arena)
    components: Vec<Component<T>>,
    layout: Layout,
    // TODO: tree of objects to determine layout

    fonts: Fonts,
}

impl<T> Window<T> {
    pub fn new(window_class: PCWSTR, component: crate::component::Component<T>, width: u32, height: u32, title: String) -> Self {
        let title = HSTRING::from(title);

        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                &title,
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        };

        let scale = 1.0;

        let fonts = Fonts::new();

        // resize window to actually desired size
        unsafe {
            let mut client_rect = RECT::default();
            GetClientRect(window, &mut client_rect);

            let mut window_rect = RECT::default();
            GetWindowRect(window, &mut window_rect);

            let deco_width = (window_rect.right - window_rect.left) - client_rect.right;
            let deco_height = (window_rect.bottom - window_rect.top) - client_rect.bottom;

            MoveWindow(window, window_rect.left, window_rect.top, width as i32 + deco_width, height as i32 + deco_height, true);
        }

        let mut native_components = Vec::new();
        let mut layout = Layout::new(width, height);

        let root_node = layout.get_root_node();
        Self::create_components(
            window,
            component,
            &mut native_components,
            &mut layout,
            root_node,
            &fonts,
        );

        layout.calculate_coords();

        for native_component in &mut native_components {
            match native_component {
                Component::Button(button) => button.update(&mut layout, &fonts, 1.0),
            }
        }

        let mut result = Self {
            _hwnd: window,

            scale,
            width,
            height,
            title,

            components: native_components,
            layout,
            
            fonts,
        };

        result.rescale(None);

        result
    }

    fn create_components(window: HWND, component: crate::component::Component<T>, native_components: &mut Vec<Component<T>>, layout: &mut Layout, layout_parent: GenArenaKey, fonts: &Fonts) {
        match component {
            crate::component::Component::Button(button) => {
                let button = Component::Button(Button::new(
                    window,
                    (native_components.len() + 1) as u16,
                    button.label.clone(),
                    button.on_press,
                    layout_parent, 
                    button.position,
                    layout,
                    fonts,
                ));

                native_components.push(button);
            },
            crate::component::Component::Container(container) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::Container,
                    container.position,
                );

                Self::create_components(
                    window,
                    *container.child,
                    native_components,
                    layout,
                    node,
                    fonts
                );
            },
            crate::component::Component::Horizontal(horizontal) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::HorizontalGroup {
                        spacing: horizontal.spacing,
                    },
                    horizontal.position,
                );

                for child in horizontal.children {
                    Self::create_components(
                        window,
                        child,
                        native_components,
                        layout,
                        node,
                        fonts
                    );
                }
            },
            crate::component::Component::Vertical(vertical) => {
                let node = layout.add_node(
                    layout_parent,
                    crate::layout::LayoutNodeType::VerticalGroup {
                        spacing: vertical.spacing,
                    },
                    vertical.position,
                );

                for child in vertical.children {
                    Self::create_components(
                        window,
                        child,
                        native_components,
                        layout,
                        node,
                        fonts
                    );
                }
            },
        }
    }

    fn rescale(&mut self, suggested_pos: Option<(i32, i32)>) {
        self.scale = unsafe {
            let dpi = GetDpiForWindow(self._hwnd);
            dpi as f32 / 96.0
        };

        unsafe {
            let mut client_rect = RECT::default();
            GetClientRect(self._hwnd, &mut client_rect);

            let mut window_rect = RECT::default();
            GetWindowRect(self._hwnd, &mut window_rect);

            let deco_width = (window_rect.right - window_rect.left) - client_rect.right;
            let deco_height = (window_rect.bottom - window_rect.top) - client_rect.bottom;

            let width = (self.width as f32 * self.scale) as i32;
            let height = (self.height as f32 * self.scale) as i32;

            let (x, y) = match suggested_pos {
                Some(v) => v,
                None => (window_rect.left, window_rect.top),
            };

            MoveWindow(self._hwnd, x, y, width + deco_width, height + deco_height, true);
        }

        // update font size
        self.fonts.update(self.scale);

        // update controls
        for component in &mut self.components {
            match component {
                Component::Button(button) => button.update(&mut self.layout, &self.fonts, self.scale),
            }
        }
    }

    pub fn wndproc(
        &mut self,
        state: &mut T,
        window: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    let hdc = BeginPaint(window, &mut ps);

                    FillRect(hdc, &ps.rcPaint, HBRUSH(COLOR_WINDOW.0 as *mut c_void));

                    _ = EndPaint(window, &ps);

                    //_ = ValidateRect(Some(window), None);
                    LRESULT(0)
                }
                WM_DPICHANGED => {
                    let suggested_size = *(lparam.0 as *mut RECT);

                    self.rescale(Some((suggested_size.left, suggested_size.top)));

                    let _ = InvalidateRect(Some(window), None, true);

                    LRESULT(0)
                }
                WM_COMMAND => {
                    let id = (wparam.0 & 0xffff) as u16;

                    if let Some(c) = self.components.get(id as usize - 1) {
                        let result = match c {
                            Component::Button(button) => {
                                button.wndproc_on_pressed(state, window, msg, wparam, lparam)
                            }
                        };

                        match result {
                            Some(v) => v,
                            None => DefWindowProcW(window, msg, wparam, lparam),
                        }
                    } else {
                        DefWindowProcW(window, msg, wparam, lparam)
                    }
                }
                _ => {
                    DefWindowProcW(window, msg, wparam, lparam)
                }
            }
        }
    }
}

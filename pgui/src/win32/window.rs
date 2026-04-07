use std::ffi::c_void;

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{BeginPaint, COLOR_WINDOW, EndPaint, FillRect, HBRUSH, PAINTSTRUCT},
        UI::WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, PostQuitMessage, WINDOW_EX_STYLE,
            WM_DESTROY, WM_PAINT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
    core::{PCWSTR, w},
};

use crate::win32::button::Button;

pub enum Component<T> {
    Button(Button<T>),
}

pub struct Window<T> {
    _hwnd: HWND,

    // TODO: list of components (or more likely, a generational arena)
    components: Vec<Component<T>>,
    // TODO: tree of objects to determine layout
}

impl<T> Window<T> {
    pub fn new(window_class: PCWSTR, components: &[crate::component::Component<T>]) -> Self {
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                w!("Simple Window"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                None,
                None,
            )
            .unwrap()
        };

        let mut native_components = Vec::new();

        for component in components {
            let native_component = match component {
                crate::component::Component::Button(button) => Component::Button(Button::new(
                    window,
                    (native_components.len() + 1) as u16,
                    button.label.clone(),
                    button.on_press.unwrap(),
                )),
            };

            native_components.push(native_component);
        }

        Self {
            _hwnd: window,

            components: native_components,
        }
    }

    pub fn wndproc(
        &self,
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
                _ => {
                    let mut result = None;
                    for c in &self.components {
                        result = match c {
                            Component::Button(button) => {
                                button.wndproc(state, window, msg, wparam, lparam)
                            }
                        };

                        if result.is_some() {
                            break;
                        }
                    }

                    match result {
                        Some(v) => v,
                        None => DefWindowProcW(window, msg, wparam, lparam),
                    }
                }
            }
        }
    }
}

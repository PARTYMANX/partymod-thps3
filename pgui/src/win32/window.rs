use windows::{Win32::{Foundation::HWND, UI::WindowsAndMessaging::{CW_USEDEFAULT, CreateWindowExW, WINDOW_EX_STYLE, WS_OVERLAPPEDWINDOW, WS_VISIBLE}}, core::{PCWSTR, w}};

use crate::win32::button::Button;

pub struct Window {
    hwnd: HWND,

    // TODO: list of components (or more likely, a generational arena)
    // TODO: tree of objects to determine layout
}

impl Window {
    pub fn new(window_class: PCWSTR) -> Self {
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
            ).unwrap()
        };

        let _button = Button::new(window);

        Self {
            hwnd: window,
        }
    }

    pub fn get_hwnd(&self) -> HWND {
        self.hwnd
    }
}
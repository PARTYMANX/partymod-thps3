use std::{ffi::c_void, mem::MaybeUninit};

use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            CreateFontW, DEFAULT_GUI_FONT,
            GetObjectW, GetStockObject, HFONT, LOGFONTW,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{ICC_TAB_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx},
            WindowsAndMessaging::{
                CS_HREDRAW, CS_VREDRAW, DefWindowProcW, DispatchMessageW, GetMessageW, MSG,
                RegisterClassW, TranslateMessage,
                UnregisterClassW, WNDCLASSW,
            },
        },
    },
    core::{PCWSTR, w},
};

use crate::syncunsafecell::SyncUnsafeCell;

use super::window::Window;

pub static FONT_CONTEXT: SyncUnsafeCell<MaybeUninit<Fonts>> =
    SyncUnsafeCell::new(MaybeUninit::uninit());

pub struct Fonts {
    /// Unscaled copy of the default GUI font, for layout calculation purposes.
    pub default_font: HFONT,
    /// Scaled copy of the default GUI font, for actual use.
    pub default_font_scaled: HFONT,
}

unsafe impl Sync for Fonts {}

impl Fonts {
    fn new() -> Self {
        unsafe {
            let mut lf = LOGFONTW::default();
            let _ = GetObjectW(
                GetStockObject(DEFAULT_GUI_FONT),
                size_of::<LOGFONTW>() as i32,
                Some(&raw mut lf as *mut c_void),
            );
            let default_font = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet,
                lf.lfOutPrecision,
                lf.lfClipPrecision,
                lf.lfQuality,
                lf.lfPitchAndFamily as u32,
                PCWSTR(lf.lfFaceName.as_ptr()),
            );

            let default_font_scaled = CreateFontW(
                lf.lfHeight,
                lf.lfWidth,
                lf.lfEscapement,
                lf.lfOrientation,
                lf.lfWeight,
                lf.lfItalic as u32,
                lf.lfUnderline as u32,
                lf.lfStrikeOut as u32,
                lf.lfCharSet,
                lf.lfOutPrecision,
                lf.lfClipPrecision,
                lf.lfQuality,
                lf.lfPitchAndFamily as u32,
                PCWSTR(lf.lfFaceName.as_ptr()),
            );

            Self {
                default_font,
                default_font_scaled,
            }
        }
    }
}

pub static APP_CONTEXT: SyncUnsafeCell<Option<AppContext>> = SyncUnsafeCell::new(None);

pub struct AppContext {
    _instance: HINSTANCE,

    wndproc: Box<dyn FnMut(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
}

unsafe impl Sync for AppContext {}

pub fn run<T: 'static>(mut state: T, button_press: fn(&mut T)) {
    unsafe {
        // TODO: some sort of global setup?

        let icex = INITCOMMONCONTROLSEX {
            dwICC: ICC_TAB_CLASSES,
            ..Default::default()
        };
        let _ = InitCommonControlsEx(&icex);

        let fonts = Fonts::new();

        let ctx = &mut *FONT_CONTEXT.get();

        ctx.write(fonts);
    }

    // TODO: get default font

    //MessageBoxA(None, s!("Ansi"), s!("World"), MB_OK);
    //ShellMessageBoxW(None, None, w!("Wide"), w!("World"), MB_ICONERROR);

    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let window_class = w!("pgui_window");

        let wc = WNDCLASSW {
            //hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let _atom = RegisterClassW(&wc);

        let window = Window::new(window_class, button_press);

        let ctx = &mut *APP_CONTEXT.get();
        *ctx = Some(AppContext {
            _instance: instance.into(),
            wndproc: Box::new(move |hwnd, msg, wparam, lparam| {
                window.wndproc(&mut state, hwnd, msg, wparam, lparam)
            }),
        });

        // Run the event loop.

        let mut message = MSG::default();

        while GetMessageW(&mut message, None, 0, 0).into() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        // App terminating, unregister the class.

        let _ = UnregisterClassW(window_class, Some(instance.into()));
    }
}

extern "system" fn wndproc(window: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let ctx = &mut *APP_CONTEXT.get();

        match ctx {
            Some(v) => (*v.wndproc)(window, msg, wparam, lparam),
            None => DefWindowProcW(window, msg, wparam, lparam),
        }
    }
}

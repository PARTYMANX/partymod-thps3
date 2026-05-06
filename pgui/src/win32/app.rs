use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{ICC_TAB_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx},
            HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext},
            WindowsAndMessaging::{
                CS_HREDRAW, CS_VREDRAW, DefWindowProcW, DispatchMessageW, GetMessageW, IDC_ARROW,
                LoadCursorW, MSG, RegisterClassW, TranslateMessage, UnregisterClassW, WNDCLASSW,
            },
        },
    },
    core::w,
};

use crate::syncunsafecell::SyncUnsafeCell;

use super::window::Window;

pub static APP_CONTEXT: SyncUnsafeCell<Option<AppContext>> = SyncUnsafeCell::new(None);

pub struct AppContext {
    _instance: HINSTANCE,

    wndproc: Box<dyn FnMut(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
}

unsafe impl Sync for AppContext {}

pub fn run<T: 'static>(
    mut state: T,
    component: crate::component::Component<T>,
    window: crate::window::Window<T>,
) {
    unsafe {
        // TODO: some sort of global setup?

        let icex = INITCOMMONCONTROLSEX {
            dwICC: ICC_TAB_CLASSES,
            ..Default::default()
        };
        let _ = InitCommonControlsEx(&icex);
    }

    // TODO: get default font

    //MessageBoxA(None, s!("Ansi"), s!("World"), MB_OK);
    //ShellMessageBoxW(None, None, w!("Wide"), w!("World"), MB_ICONERROR);

    unsafe {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        let instance = GetModuleHandleW(None).unwrap();
        let window_class = w!("pgui_window");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            hInstance: instance.into(),
            lpszClassName: window_class,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let _atom = RegisterClassW(&wc);

        let mut window = Window::new(
            window_class,
            component,
            window.width,
            window.height,
            window.title,
            window.state_hook,
            window.post_update,
        );

        window.run_state_hooks(&state); // run state hooks to populate components
        window.run_post_update(&mut state);

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

use partymod_common::{logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell};
use raw_window_handle::HasWindowHandle;

use crate::{VERSION, logger, sdl::SDL_CONTEXT};

pub struct WindowContext {
    window: Option<sdl3::video::Window>,
    is_windowed: bool,
    is_borderless: bool,
    res_x: u32,
    res_y: u32,
}

unsafe impl Sync for WindowContext {}
unsafe impl Send for WindowContext {}

pub static WINDOW_CONTEXT: SyncUnsafeCell<Option<WindowContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    let is_windowed = crate::config::get_bool("Graphics", "Windowed", true);
    let is_borderless = crate::config::get_bool("Graphics", "Borderless", false);
    let mut res_x = crate::config::get_int("Graphics", "ResolutionX", 640) as u32;
    let mut res_y = crate::config::get_int("Graphics", "ResolutionY", 480) as u32;

    if res_x == 0 || res_y == 0 {
        unsafe {
            let ctx = &*SDL_CONTEXT.get();
            match ctx {
                Some(v) => {
                    let display = v.video_subsystem.get_primary_display().unwrap();
                    let mode = display.get_mode().unwrap();
                    res_x = mode.w as u32;
                    res_y = mode.h as u32;
                }
                None => panic!("Tried to use uninitialized SDL context!"),
            }
        }
    }

    res_x = res_x.max(640);
    res_y = res_y.max(480);

    unsafe {
        let ctx = &mut *WINDOW_CONTEXT.get();
        *ctx = Some(WindowContext {
            window: None,
            is_windowed,
            is_borderless,
            res_x,
            res_y,
        });
    }
}

pub fn init_settings() {
    let window_context = match unsafe { &*WINDOW_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized window context!"),
    };

    unsafe {
        let ptr_is_windowed = 0x008510a9 as *mut bool;
        let ptr_res_x = 0x00851084 as *mut u32;
        let ptr_res_y = 0x00851088 as *mut u32;

        *ptr_is_windowed = window_context.is_windowed;
        *ptr_res_x = window_context.res_x;
        *ptr_res_y = window_context.res_y;
    }

    logger::log(LogLevel::Debug, "INITIALIZING SETTINGS!");
}

pub fn handle_event(e: &sdl3::event::Event) {
    match e {
        sdl3::event::Event::Window {
            timestamp: _,
            window_id: _,
            win_event,
        } => match win_event {
            sdl3::event::WindowEvent::FocusGained => unsafe {
                let is_focused = 0x00850f74 as *mut bool;
                *is_focused = true;
            },
            sdl3::event::WindowEvent::FocusLost => unsafe {
                let is_focused = 0x00850f74 as *mut bool;
                *is_focused = false;
            },
            _ => {}
        },
        _ => {}
    }
}

pub fn get_window_size() -> (u32, u32) {
    let window_context = match unsafe { &*WINDOW_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized window context!"),
    };

    if let Some(window) = &window_context.window {
        window.size_in_pixels()
    } else {
        panic!("Window was None!")
    }
}

extern "C" fn get_or_create_window() -> isize {
    let window_context = match unsafe { &mut *WINDOW_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized window context!"),
    };

    let hwnd = 0x0085109c as *mut isize;

    if window_context.window.is_none() {
        logger::log(LogLevel::Debug, "CREATING WINDOW!");
        let is_windowed = window_context.is_windowed;
        let is_borderless = window_context.is_borderless;
        let res_x = window_context.res_x;
        let res_y = window_context.res_y;

        unsafe {
            let ptr_is_windowed = 0x008510a9 as *mut bool;
            let ptr_res_x = 0x00851084 as *mut u32;
            let ptr_res_y = 0x00851088 as *mut u32;

            *ptr_is_windowed = is_windowed;
            *ptr_res_x = res_x;
            *ptr_res_y = res_y;
        }

        let mut window_builder = unsafe {
            let ctx = &*SDL_CONTEXT.get();

            match ctx {
                Some(v) => {
                    v.video_subsystem
                        .window(&format!("THPS3 - PARTYMOD {}", VERSION), res_x, res_y)
                }
                None => panic!("Tried to use uninitialized SDL context!"),
            }
        };

        window_builder.position_centered().high_pixel_density();

        if !is_windowed {
            window_builder.fullscreen();
        }

        if is_borderless && is_windowed {
            window_builder.borderless();
        }

        let window = window_builder.build().unwrap();

        window_context.window = Some(window.clone());
        window_context.is_windowed = is_windowed;

        match window.window_handle().unwrap().as_raw() {
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                unsafe { *hwnd = handle.hwnd.get() };
            }
            _ => unreachable!("Got non-windows window handle!"),
        }
    }

    return unsafe { *hwnd };
}

extern "C" fn is_window() -> bool {
    let window_context = match unsafe { &*WINDOW_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized window context!"),
    };
    window_context.is_windowed
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x00409be0 as *mut (), get_or_create_window as *const ());
        patch::patch_jmp(0x00409f70 as *mut (), is_window as *const ());
    }
}

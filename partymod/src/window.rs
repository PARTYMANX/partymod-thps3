use partymod_common::{config, patch};
use raw_window_handle::HasWindowHandle;

use crate::sdl::SDL_CONTEXT;

pub struct WindowContext {
    window: Option<sdl3::video::Window>,
    is_windowed: bool,
    is_borderless: bool,
    res_x: u32,
    res_y: u32,
}

pub static mut WINDOW_CONTEXT: std::mem::MaybeUninit<WindowContext> = std::mem::MaybeUninit::uninit();

pub fn init() {
    let is_windowed = config::get_config_bool("Graphics", "Windowed", true);
    let is_borderless = config::get_config_bool("Graphics", "Borderless", false);
    let res_x = config::get_config_int("Graphics", "ResolutionX", 640);
    let res_y = config::get_config_int("Graphics", "ResolutionY", 480);

    unsafe {
        WINDOW_CONTEXT = std::mem::MaybeUninit::new(WindowContext { 
            window: None,
            is_windowed,
            is_borderless,
            res_x,
            res_y,
        });
    }
}

pub fn init_settings() {
    #[allow(static_mut_refs)]
    let window_context = unsafe { WINDOW_CONTEXT.assume_init_mut() };

    unsafe {
        let ptr_is_windowed = 0x008510a9 as *mut bool;
        let ptr_res_x = 0x00851084 as *mut u32;
        let ptr_res_y = 0x00851088 as *mut u32;

        *ptr_is_windowed = window_context.is_windowed;
        *ptr_res_x = window_context.res_x;
        *ptr_res_y = window_context.res_y;
    }

    println!("INITIALIZING SETTINGS!");
}

pub fn handle_event(e: &sdl3::event::Event) {
    match e {
        sdl3::event::Event::Window { timestamp: _, window_id: _, win_event } => {
            match win_event {
                sdl3::event::WindowEvent::FocusGained => {
                    unsafe {
                        let is_focused = 0x00850f74 as *mut bool;
                        *is_focused = true;
                    }
                },
                sdl3::event::WindowEvent::FocusLost => {
                    unsafe {
                        let is_focused = 0x00850f74 as *mut bool;
                        *is_focused = false;
                    }
                },
                _ => {},
            }
        },
        _ => {},
    }
}

extern "C" fn get_or_create_window() -> isize {
    #[allow(static_mut_refs)]
    let window_context = unsafe { WINDOW_CONTEXT.assume_init_mut() };

    let hwnd = 0x0085109c as *mut isize;

    if window_context.window.is_none() {
        println!("CREATING WINDOW!");
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

        #[allow(static_mut_refs)]
        let mut window_builder = unsafe {
            SDL_CONTEXT.assume_init_ref().video_subsystem.window("THPS3 - PARTYMOD", res_x, res_y)
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
            },
            _ => unreachable!("Got non-windows window handle!")
        }
    }

    return unsafe { *hwnd };
}

extern "C" fn is_window() -> bool {
    #[allow(static_mut_refs)]
    let window_context = unsafe { WINDOW_CONTEXT.assume_init_ref() };
    window_context.is_windowed
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x00409be0 as *mut (), get_or_create_window as *const ());
        patch::patch_jmp(0x00409f70 as *mut (), is_window as *const ());
    }
}
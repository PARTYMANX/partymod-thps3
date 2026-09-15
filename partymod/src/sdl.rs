use std::path::Path;

use partymod_common::{logger::LogLevel, syncunsafecell::SyncUnsafeCell};

use crate::logger;

pub struct SDLContext {
    pub sdl_context: sdl3::Sdl,
    pub event_pump: sdl3::EventPump,
    pub video_subsystem: sdl3::VideoSubsystem,
    //pub joystick_subsystem: sdl3::JoystickSubsystem,
    pub gamepad_subsystem: sdl3::GamepadSubsystem,
}

unsafe impl Sync for SDLContext {}
unsafe impl Send for SDLContext {}

pub static SDL_CONTEXT: SyncUnsafeCell<Option<SDLContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    let sdl_context = sdl3::init().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    //let joystick_subsystem = sdl_context.joystick().unwrap();
    let gamepad_subsystem = sdl_context.gamepad().unwrap();

    // load gamecontrollerdb
    let path = Path::new("gamecontrollerdb.txt");
    match gamepad_subsystem.load_mappings(path) {
        Ok(_) => {}
        Err(e) => {
            logger::log(
                LogLevel::Warn,
                &format!("Failed to load game controller database: {}", e),
            );
        }
    }

    unsafe {
        let ctx = &mut *SDL_CONTEXT.get();
        *ctx = Some(SDLContext {
            sdl_context,
            event_pump,
            video_subsystem,
            //joystick_subsystem,
            gamepad_subsystem,
        });
    }
}

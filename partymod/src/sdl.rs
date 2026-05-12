use partymod_common::syncunsafecell::SyncUnsafeCell;

pub struct SDLContext {
    pub sdl_context: sdl3::Sdl,
    pub video_subsystem: sdl3::VideoSubsystem,
    pub joystick_subsystem: sdl3::JoystickSubsystem,
    pub gamepad_subsystem: sdl3::GamepadSubsystem,
}

unsafe impl Sync for SDLContext {}
unsafe impl Send for SDLContext {}

pub static SDL_CONTEXT: SyncUnsafeCell<Option<SDLContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let gamepad_subsystem = sdl_context.gamepad().unwrap();

    unsafe {
        let ctx = &mut *SDL_CONTEXT.get();
        *ctx = Some(SDLContext {
            sdl_context,
            video_subsystem,
            joystick_subsystem,
            gamepad_subsystem,
        });
    }
}

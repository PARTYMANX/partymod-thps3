pub struct SdlContext {
    pub sdl_context: sdl3::Sdl,
    pub video_subsystem: sdl3::VideoSubsystem,
    pub joystick_subsystem: sdl3::JoystickSubsystem,
    pub gamepad_subsystem: sdl3::GamepadSubsystem,
}

unsafe impl Send for SdlContext {}

pub static mut SDL_CONTEXT: std::mem::MaybeUninit<SdlContext> = std::mem::MaybeUninit::uninit();

pub fn init() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let gamepad_subsystem = sdl_context.gamepad().unwrap();

    unsafe {
        SDL_CONTEXT = std::mem::MaybeUninit::new(SdlContext {
            sdl_context,
            video_subsystem,
            joystick_subsystem,
            gamepad_subsystem,
        });
    }
}

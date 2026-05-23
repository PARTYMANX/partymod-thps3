use partymod_common::{
    gamepad::GamepadManager, keyboard::KeybindManager, logger::LogLevel, patch,
    syncunsafecell::SyncUnsafeCell,
};
use partymod_config::{GAMEPAD_BINDS, KEYBINDS};
use partymod_config_common::{Button, Stick};
use sdl3::keyboard::Scancode;

use crate::{
    config, event,
    logger::{self, LOGGER_CONTEXT},
    sdl::{SDL_CONTEXT, SDLContext},
};

mod device;
mod keyboard;
mod parkeditor;

pub struct InputContext {
    keybind_manager: KeybindManager,
    gamepad_manager: GamepadManager,
    keyboard: keyboard::Keyboard,
    park_editor_state: parkeditor::ParkEditorControlsState,

    is_using_keyboard: bool,
    is_cursor_visible: bool,
}

unsafe impl Sync for InputContext {}
unsafe impl Send for InputContext {}

pub static INPUT_CONTEXT: SyncUnsafeCell<Option<InputContext>> = SyncUnsafeCell::new(None);

fn init_context() {
    unsafe {
        let log_ctx = &*LOGGER_CONTEXT.get();

        let logger = match log_ctx {
            Some(v) => v.clone(),
            None => panic!("Tried to use uninitialized logger context!"),
        };

        let ctx = &mut *INPUT_CONTEXT.get();
        *ctx = Some(InputContext {
            keybind_manager: KeybindManager::new(Some(logger.clone())),
            gamepad_manager: GamepadManager::new(1, Some(logger.clone())),
            keyboard: keyboard::Keyboard::new(),
            park_editor_state: parkeditor::ParkEditorControlsState::new(),

            is_using_keyboard: true,
            is_cursor_visible: true,
        })
    }
}

fn setup_controls() {
    let inp_ctx = unsafe {
        match &mut *INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    for bind in &KEYBINDS {
        let default = match bind.default {
            None => -1,
            Some(v) => v.to_i32(),
        };

        let key = config::get_int("Keybinds", bind.key, default);

        if let Some(v) = Scancode::from_i32(key) {
            inp_ctx.keybind_manager.add_key_binding(bind.key, v);
        }
    }

    inp_ctx.keybind_manager.add_menu_binding("Accept", Scancode::Return, "Ollie");
    inp_ctx.keybind_manager.add_menu_binding("Accept2", Scancode::KpEnter, "Ollie");
    inp_ctx.keybind_manager.add_menu_binding("Back", Scancode::Escape, "Grind");
    inp_ctx.keybind_manager.add_menu_binding("Up", Scancode::Up, "Forward");
    inp_ctx.keybind_manager.add_menu_binding("Down", Scancode::Down, "Backward");
    inp_ctx.keybind_manager.add_menu_binding("Left", Scancode::Left, "Left");
    inp_ctx.keybind_manager.add_menu_binding("Right", Scancode::Right, "Right");

    for bind in &GAMEPAD_BINDS {
        match &bind.default {
            partymod_config_common::BindType::Button { value } => {
                let default = match value {
                    None => -1,
                    Some(v) => *v as i32,
                };

                let button = config::get_int("Gamepad", bind.key, default);

                if let Some(v) = Button::from_i32(button) {
                    inp_ctx.gamepad_manager.add_button_binding(bind.key, v);
                }
            }
            partymod_config_common::BindType::Stick { value } => {
                let default = match value {
                    None => -1,
                    Some(v) => *v as i32,
                };

                let stick = config::get_int("Gamepad", bind.key, default);

                if let Some(v) = Stick::from_i32(stick) {
                    inp_ctx.gamepad_manager.add_stick_binding(bind.key, v);
                }
            }
        }
    }
}

fn event_handler(event: &sdl3::event::Event) {
    let sdl_ctx = unsafe {
        match &*SDL_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized SDL context!"),
        }
    };

    let inp_ctx = unsafe {
        match &mut *INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    match event {
        sdl3::event::Event::MouseMotion { x, y, .. } => unsafe {
            set_using_keyboard(inp_ctx, sdl_ctx, true);
            let mouse_move: unsafe extern "C" fn(i16, i16, i16) =
                std::mem::transmute(0x00405370 as *const ());

            mouse_move(0x0000, *x as i16, *y as i16);
        },
        sdl3::event::Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => unsafe {
            set_using_keyboard(inp_ctx, sdl_ctx, true);
            match mouse_btn {
                sdl3::mouse::MouseButton::Left => {
                    let mouse_left_down: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x00404950 as *const ());

                    mouse_left_down(0x0000, *x as i16, *y as i16);
                }
                sdl3::mouse::MouseButton::Middle => {
                    let mouse_middle_down: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x00404dc0 as *const ());

                    mouse_middle_down(0x0000, *x as i16, *y as i16);
                }
                sdl3::mouse::MouseButton::Right => {
                    let mouse_right_down: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x00405010 as *const ());

                    mouse_right_down(0x0000, *x as i16, *y as i16);
                }
                _ => {}
            }
        },
        sdl3::event::Event::MouseButtonUp {
            mouse_btn, x, y, ..
        } => unsafe {
            set_using_keyboard(inp_ctx, sdl_ctx, true);
            match mouse_btn {
                sdl3::mouse::MouseButton::Left => {
                    let mouse_left_up: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x00404d20 as *const ());

                    mouse_left_up(0x0000, *x as i16, *y as i16);
                }
                sdl3::mouse::MouseButton::Middle => {
                    let mouse_middle_up: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x00404f70 as *const ());

                    mouse_middle_up(0x0000, *x as i16, *y as i16);
                }
                sdl3::mouse::MouseButton::Right => {
                    let mouse_right_up: unsafe extern "C" fn(i16, i16, i16) =
                        std::mem::transmute(0x004052d0 as *const ());

                    mouse_right_up(0x0000, *x as i16, *y as i16);
                }
                _ => {}
            }
        },
        sdl3::event::Event::KeyDown { .. } => {
            set_using_keyboard(inp_ctx, sdl_ctx, true);
        }
        sdl3::event::Event::ControllerButtonDown { .. } => {
            set_using_keyboard(inp_ctx, sdl_ctx, false);
        }
        _ => {}
    }

    inp_ctx.keyboard.handle_keyboard_event(event);

    inp_ctx
        .gamepad_manager
        .event_handler(event, &sdl_ctx.gamepad_subsystem);
}

// in-engine manager for controllers. replaces SIO::Manager
#[repr(C)]
struct InputManager {
    unk_data: [u32; 7],
    hinstance: *const std::ffi::c_void,
    hwnd: *const std::ffi::c_void,
    dinput_interface: *const std::ffi::c_void,
}

impl InputManager {
    extern "thiscall" fn init(
        &mut self,
        hinstance: *const std::ffi::c_void,
        hwnd: *const std::ffi::c_void,
    ) -> u32 {
        logger::log(LogLevel::Info, "Initializing input manager!");

        self.hinstance = hinstance;
        self.hwnd = hwnd;

        // set up controls stuff here... maybe?
        init_context();
        setup_controls();

        event::register_handler(event_handler);

        let _ = self.new_device(0);

        // keyboard state must be initialized here otherwise the game will crash
        unsafe {
            let init_keyboard_state: unsafe extern "C" fn() =
                std::mem::transmute(0x00403bb0 as *const ());

            init_keyboard_state();
        }

        1
    }

    fn new_device(&mut self, idx: u32) -> bool {
        unsafe {
            let new_device: unsafe extern "thiscall" fn(
                *mut InputManager,
                u32,
                *const std::ffi::c_void,
            ) -> bool = std::mem::transmute(0x0040dac0 as *const ());

            new_device(self, idx, std::ptr::null())
        }
    }

    extern "thiscall" fn disable_actuator(&mut self, idx: u32) {
        unsafe {
            let self_ptr = self as *mut Self as *mut ();
            let device = self_ptr.byte_add((8 + (idx * 0x7c)) as usize) as *mut *mut device::Device;

            match (*device).as_mut() {
                Some(v) => {
                    v.disable_actuators();
                }
                None => {}
            }
        }
    }
}

extern "cdecl" fn is_window_active() -> bool {
    true
}

extern "C" fn set_cursor_active() {
    let inp_ctx = unsafe {
        match &mut *INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    let sdl_ctx = unsafe {
        match &*SDL_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized SDL context!"),
        }
    };

    inp_ctx.is_cursor_visible = true;

    show_hide_cursor(inp_ctx, sdl_ctx);
}

extern "C" fn set_cursor_inactive() {
    let inp_ctx = unsafe {
        match &mut *INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    let sdl_ctx = unsafe {
        match &*SDL_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized SDL context!"),
        }
    };

    inp_ctx.is_cursor_visible = true;

    show_hide_cursor(inp_ctx, sdl_ctx);
}

fn set_using_keyboard(inp_ctx: &mut InputContext, sdl_ctx: &SDLContext, using_keyboard: bool) {
    inp_ctx.is_using_keyboard = using_keyboard;

    inp_ctx.gamepad_manager.enable_rumble(0, !using_keyboard);

    show_hide_cursor(inp_ctx, sdl_ctx);
}

fn show_hide_cursor(inp_ctx: &mut InputContext, sdl_ctx: &SDLContext) {
    if inp_ctx.is_using_keyboard && inp_ctx.is_cursor_visible {
        sdl_ctx.sdl_context.mouse().show_cursor(true);
    } else {
        sdl_ctx.sdl_context.mouse().show_cursor(false);
    }
}

fn is_menu_open() -> bool {
    unsafe {
        let menu_is_open: unsafe extern "C" fn() -> bool =
            std::mem::transmute(0x0044a540 as *const ());

        menu_is_open()
    }
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040db20 as *mut (), InputManager::init as *const ());

        // replace DisableActuator aliases with correct calls
        patch::patch_call(
            0x004ab8a7 as *mut (),
            InputManager::disable_actuator as *const (),
        );
        patch::patch_call(
            0x0041fa98 as *mut (),
            InputManager::disable_actuator as *const (),
        );
        patch::patch_call(
            0x00463f3d as *mut (),
            InputManager::disable_actuator as *const (),
        );

        device::patch();
        keyboard::patch();
        parkeditor::patch();

        // always say the window is active
        patch::patch_jmp(0x004090b0 as *mut (), is_window_active as *const ());

        // cursor handling
        patch::patch_jmp(0x00405780 as *mut (), set_cursor_active as *const ());
        patch::patch_jmp(0x004057c0 as *mut (), set_cursor_inactive as *const ());
    }
}

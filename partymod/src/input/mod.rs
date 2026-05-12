use partymod_common::{
    controls::GamepadManager, logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell,
};
use partymod_config::GAMEPAD_BINDS;
use partymod_config_common::Button;

use crate::{config, event, logger, sdl::SDL_CONTEXT};

mod device;

pub struct InputContext {
    gamepad_manager: GamepadManager,
}

unsafe impl Sync for InputContext {}
unsafe impl Send for InputContext {}

pub static INPUT_CONTEXT: SyncUnsafeCell<Option<InputContext>> = SyncUnsafeCell::new(None);

fn init_context() {
    unsafe {
        let ctx = &mut *INPUT_CONTEXT.get();
        *ctx = Some(InputContext {
            gamepad_manager: GamepadManager::new(),
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

    for bind in &GAMEPAD_BINDS {
        match &bind.default {
            //
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
            partymod_config_common::BindType::Stick { value } => {}
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

    inp_ctx
        .gamepad_manager
        .event_handler(event, &sdl_ctx.gamepad_subsystem);
}

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

        let dev_result = self.new_device(0);
        println!("DEVICE RESULT: {}", dev_result);

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
}

extern "cdecl" fn is_window_active() -> bool {
    true
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040db20 as *mut (), InputManager::init as *const ());
        device::patch();

        // always say the window is active
        patch::patch_jmp(0x004090b0 as *mut (), is_window_active as *const ());
    }
}

use partymod_common::{keyboard::KeybindManager, patch::patch_jmp};
use sdl3::{
    keyboard::{Mod, Scancode},
    sys::{keyboard::SDL_GetKeyFromScancode, keycode::SDL_Keymod},
};

pub struct Keyboard {
    keys_down: Vec<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            keys_down: Vec::with_capacity(256),
        }
    }

    pub fn handle_keyboard_event(&mut self, event: &sdl3::event::Event) {
        match event {
            sdl3::event::Event::KeyDown {
                scancode, keymod, ..
            } => {
                if let Some(code) = scancode {
                    self.handle_key_down(code, keymod);
                }
            }
            _ => {}
        }
    }

    fn update_keys_down(&mut self, keybind_manager: &KeybindManager) {
        unsafe {
            let key_makes = 0x005d030c as *mut [u8; 256];
            let key_makes_count = 0x005d0300 as *mut u32;

            *key_makes_count = 0;

            for key in &self.keys_down {
                let locked = match *key {
                    25 => {
                        keybind_manager.is_menu_key_locked("Accept")
                            || keybind_manager.is_menu_key_locked("Accept2")
                    }
                    26 => keybind_manager.is_menu_key_locked("Back"),
                    _ => false,
                };

                if locked {
                    // key is locked, skip
                    continue;
                }

                (*key_makes)[(*key_makes_count) as usize] = *key;
                *key_makes_count += 1;

                if *key_makes_count >= 256 {
                    break;
                }
            }

            self.keys_down.clear();
        }
    }

    pub fn push_esc_down(&mut self) {
        self.handle_key_down(&Scancode::Escape, &Mod::empty());
    }

    fn handle_key_down(&mut self, scancode: &Scancode, keymod: &Mod) {
        match scancode {
            Scancode::Left => {
                self.push_key_down(20);
            }
            Scancode::Right => {
                self.push_key_down(21);
            }
            Scancode::Up => {
                self.push_key_down(22);
            }
            Scancode::Down => {
                self.push_key_down(23);
            }
            Scancode::Backspace => {
                self.push_key_down(24);
            }
            Scancode::Return | Scancode::KpEnter => {
                self.push_key_down(25);
            }
            Scancode::Escape => {
                self.push_key_down(26);
            }
            Scancode::Tab => {
                self.push_key_down(27);
            }
            Scancode::F1 => {
                self.push_key_down(28);
            }
            Scancode::F2 => {
                self.push_key_down(29);
            }
            Scancode::F3 | Scancode::Delete => {
                self.push_key_down(30);
            }
            Scancode::F4 => {
                self.push_key_down(31);
            }
            _ => {
                // use sys function here, the safe function is not sufficient
                let kmod = SDL_Keymod(keymod.bits());
                let key = unsafe { SDL_GetKeyFromScancode((*scancode).into(), kmod, false) };

                let c = char::from_u32(key.0);

                match c {
                    Some(v) => {
                        if v.is_ascii() {
                            self.push_key_down(v as u8);
                        }
                    }
                    None => {}
                }
            }
        }
    }

    fn push_key_down(&mut self, key_code: u8) {
        self.keys_down.push(key_code);
    }
}

pub extern "C" fn keyboard_frame(update: bool) {
    unsafe {
        let debounce_start = 0x005d0610 as *mut u64;
        let debounce_time = 0x005d0618 as *mut u32;

        if *debounce_start != 0 && *debounce_time != 0 {
            let tmr_get_time: unsafe extern "C" fn() -> u64 =
                std::mem::transmute(0x00409ae0 as *const ());
            let current_time = tmr_get_time();

            if (*debounce_start + (*debounce_time) as u64) < current_time {
                *debounce_start = 0;
                *debounce_time = 0;
            }
        }

        // NOTE: debounce doesn't affect reading keys
        if update {
            let inp_ctx = match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            };

            inp_ctx.keyboard.update_keys_down(&inp_ctx.keybind_manager);
        }
    }
}

pub fn is_keyboard_on_screen() -> bool {
    unsafe {
        let get_menu_factory: unsafe extern "C" fn(bool) -> *const () =
            std::mem::transmute(0x004d12a0 as *const ());
        let release_menu_factory: unsafe extern "C" fn() =
            std::mem::transmute(0x004d12f0 as *const ());

        let menu_factory = get_menu_factory(false);

        let result = *(menu_factory.byte_add(0x154) as *const bool);

        release_menu_factory();

        result
    }
}

pub fn is_network_menu_on_screen() -> bool {
    unsafe {
        let get_menu_something: unsafe extern "C" fn(bool) -> *const () =
            std::mem::transmute(0x004f05b0 as *const ());
        let release_menu_something: unsafe extern "C" fn() =
            std::mem::transmute(0x004f0600 as *const ());
        let is_menu_open: unsafe extern "thiscall" fn(*const ()) -> bool =
            std::mem::transmute(0x004f17d0 as *const ());

        let menu_something = get_menu_something(false);

        let result = is_menu_open(menu_something);

        release_menu_something();

        result
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_jmp(0x00403bf0 as *mut (), keyboard_frame as *const ());
    }
}

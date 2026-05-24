use std::collections::HashMap;

use sdl3::keyboard::{KeyboardState, Scancode};

// handles bindings for keyboards
pub struct KeybindManager {
    key_bindings: HashMap<String, Keybind>,
    keys_bound: HashMap<Scancode, String>,
    menu_bindings: HashMap<String, MenuBinding>,
    in_menu: bool,
}

struct Keybind {
    key: Scancode,
    locked_out: bool,
}

struct MenuBinding {
    key: Scancode,
    overlay: Option<String>,
    locked_out: bool,
}

impl KeybindManager {
    // TODO: logging
    pub fn new() -> Self {
        Self {
            key_bindings: HashMap::new(),
            keys_bound: HashMap::new(),
            menu_bindings: HashMap::new(),
            in_menu: false,
        }
    }

    pub fn add_key_binding(&mut self, name: &str, key: Scancode) {
        self.key_bindings.insert(
            name.to_owned(),
            Keybind {
                key,
                locked_out: false,
            },
        );
        self.keys_bound.insert(key, name.to_owned());
    }

    pub fn poll_key_binding(&mut self, name: &str, keyboard_state: &KeyboardState) -> bool {
        let binding = match self.key_bindings.get_mut(name) {
            Some(v) => v,
            None => return false,
        };

        let down = keyboard_state.is_scancode_pressed(binding.key);

        match binding.locked_out {
            false => down,
            true => {
                if !down {
                    binding.locked_out = false;
                }

                false
            }
        }
    }

    pub fn add_menu_binding(&mut self, name: &str, key: Scancode, action: &str) {
        // find the bind that this overlays, if any
        let (overlay, locked_out) = match self.keys_bound.get(&key) {
            Some(v) => {
                if v != action {
                    (Some(v.clone()), false)
                } else {
                    (None, true)
                }
            }
            None => (None, false),
        };

        self.menu_bindings.insert(
            name.to_owned(),
            MenuBinding {
                key,
                overlay,
                locked_out,
            },
        );
    }

    pub fn poll_menu_binding(&mut self, name: &str, keyboard_state: &KeyboardState) -> bool {
        let binding = match self.menu_bindings.get_mut(name) {
            Some(v) => v,
            None => return false,
        };

        let down = keyboard_state.is_scancode_pressed(binding.key);

        match binding.locked_out {
            false => down,
            true => {
                if binding.overlay.is_some() && !down {
                    binding.locked_out = false;
                }

                false
            }
        }
    }

    pub fn is_menu_key_locked(&self, name: &str) -> bool {
        let binding = match self.menu_bindings.get(name) {
            Some(v) => v,
            None => return false,
        };

        binding.locked_out
    }

    pub fn set_in_menu(&mut self, enabled: bool, keyboard_state: &KeyboardState) {
        if enabled == self.in_menu {
            return;
        }

        self.in_menu = enabled;

        for (_, menu_binding) in &mut self.menu_bindings {
            let down = keyboard_state.is_scancode_pressed(menu_binding.key);

            if down && let Some(overlay_name) = &menu_binding.overlay {
                menu_binding.locked_out = true;

                let overlay_binding = self.key_bindings.get_mut(overlay_name).unwrap();
                overlay_binding.locked_out = true;
            }
        }
    }
}

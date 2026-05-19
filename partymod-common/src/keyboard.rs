use std::{collections::HashMap, sync::Arc};

use sdl3::keyboard::{KeyboardState, Scancode};

use crate::logger::{LogLevel, Logger};

// handles bindings for keyboards
pub struct KeybindManager {
    key_bindings: HashMap<String, Scancode>,
    logger: Option<Arc<dyn Logger>>,
}

impl KeybindManager {
    // TODO: logging
    pub fn new(logger: Option<Arc<dyn Logger>>) -> Self {
        Self {
            key_bindings: HashMap::new(),

            logger,
        }
    }

    pub fn add_button_binding(&mut self, name: &str, key: Scancode) {
        self.key_bindings.insert(name.to_owned(), key);
    }

    pub fn poll_button_binding(&self, name: &str, keyboard_state: &KeyboardState) -> bool {
        let binding = match self.key_bindings.get(name) {
            Some(v) => v,
            None => return false,
        };

        keyboard_state.is_scancode_pressed(*binding)
    }

    /*
    pub fn event_handler(
        &mut self,
        event: &sdl3::event::Event,
        gamepad_subsystem: &sdl3::GamepadSubsystem,
    ) {
        match event {
            sdl3::event::Event::ControllerDeviceAdded { which, .. } => {
                self.add_controller(*which, gamepad_subsystem);
            }
            sdl3::event::Event::ControllerDeviceRemoved { which, .. } => {
                self.remove_controller(*which);
            }
            sdl3::event::Event::ControllerButtonDown { which, .. } => {
                // find the controller in our list
                let mut found = None;
                for (i, gamepad) in self.gamepads.iter().enumerate() {
                    if gamepad.id().unwrap().0 == *which {
                        found = Some(i);
                        break;
                    }
                }

                if let Some(idx) = found {
                    if self.max_players == 1 {
                        self.set_active(idx);
                    } else {
                        self.add_player(idx);
                    }
                }
            }
            _ => {}
        }
    }
    */
}

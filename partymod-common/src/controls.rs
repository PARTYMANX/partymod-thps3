use std::collections::HashMap;

use partymod_config_common::{Button, Stick};
use sdl3::{gamepad::Axis, sys::joystick::SDL_JoystickID};

// handles bindings, polling, and controllers
pub struct GamepadManager {
    button_bindings: HashMap<String, Button>,
    stick_bindings: HashMap<String, Stick>,
    // TODO: stick bindings
    gamepads: Vec<sdl3::gamepad::Gamepad>,
}

impl GamepadManager {
    // TODO: player management
    pub fn new() -> Self {
        Self {
            button_bindings: HashMap::new(),
            stick_bindings: HashMap::new(),
            gamepads: Vec::new(),
        }
    }

    pub fn add_controller(&mut self, idx: u32, gamepad_subsystem: &sdl3::GamepadSubsystem) {
        let gamepad = match gamepad_subsystem.open(SDL_JoystickID(idx)) {
            Ok(v) => v,
            Err(_e) => return, // fail silently for now; we'll want to add a log callback
        };

        self.gamepads.push(gamepad);
    }

    pub fn remove_controller(&mut self, id: u32) {
        let mut result = None;
        for (i, gamepad) in self.gamepads.iter().enumerate() {
            if gamepad.id().unwrap().0 == id {
                result = Some(i);
                break;
            }
        }

        match result {
            Some(v) => {
                self.gamepads.remove(v);
            }
            None => {
                // TODO: log failure
            }
        }
    }

    pub fn add_button_binding(&mut self, name: &str, button: Button) {
        self.button_bindings.insert(name.to_owned(), button);
    }

    pub fn add_stick_binding(&mut self, name: &str, stick: Stick) {
        self.stick_bindings.insert(name.to_owned(), stick);
    }

    pub fn poll_button_binding(&self, name: &str) -> (bool, i16) {
        let binding = match self.button_bindings.get(name) {
            Some(v) => v,
            None => {
                // TODO: log failure
                return (false, 0);
            }
        };

        let mut result = (false, 0);

        let button = SDLButton::from(binding);

        for gamepad in &self.gamepads {
            let (down, pressure) = match button {
                SDLButton::Button(v) => {
                    if gamepad.button(v) {
                        (true, i16::MAX)
                    } else {
                        (false, 0)
                    }
                }
                SDLButton::Axis(v) => {
                    let pressure = gamepad.axis(v);

                    (pressure > (i16::MAX / 2), pressure)
                }
            };

            result.0 |= down;
            result.1 = result.1.max(pressure);
        }

        result
    }

    pub fn poll_stick_binding(&self, name: &str) -> (i16, i16) {
        let binding = match self.stick_bindings.get(name) {
            Some(v) => v,
            None => {
                // TODO: log failure
                println!("Failed to poll!");
                return (0, 0);
            }
        };

        let mut result = (0, 0);
        let mut result_mag_sq = 0;

        for gamepad in &self.gamepads {
            let (x, y) = match binding {
                Stick::Left => {
                    (gamepad.axis(Axis::LeftX), gamepad.axis(Axis::LeftY))
                }
                Stick::Right => {
                    (gamepad.axis(Axis::RightX), gamepad.axis(Axis::RightY))
                }
            };

            // select the input with the greater magnitude
            let mag_sq = (x as i32 * x as i32) + (y as i32 * y as i32);

            if result_mag_sq < mag_sq {
                result = (x, y);
                result_mag_sq = mag_sq;
            }
        }

        result
    }

    pub fn set_rumble(&mut self, high: u16, low: u16) {
        for gamepad in &mut self.gamepads {
            let _ = gamepad.set_rumble(low, high, 0);
        }
    }

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
            _ => {}
        }
    }
}

enum SDLButton {
    Button(sdl3::gamepad::Button),
    Axis(sdl3::gamepad::Axis),
}

impl From<&Button> for SDLButton {
    fn from(value: &Button) -> Self {
        match value {
            Button::A => Self::Button(sdl3::gamepad::Button::South),
            Button::B => Self::Button(sdl3::gamepad::Button::East),
            Button::X => Self::Button(sdl3::gamepad::Button::West),
            Button::Y => Self::Button(sdl3::gamepad::Button::North),
            Button::Select => Self::Button(sdl3::gamepad::Button::Back),
            Button::Home => Self::Button(sdl3::gamepad::Button::Guide),
            Button::Start => Self::Button(sdl3::gamepad::Button::Start),
            Button::LeftStick => Self::Button(sdl3::gamepad::Button::LeftStick),
            Button::RightStick => Self::Button(sdl3::gamepad::Button::RightStick),
            Button::LeftShoulder => Self::Button(sdl3::gamepad::Button::LeftShoulder),
            Button::RightShoulder => Self::Button(sdl3::gamepad::Button::RightShoulder),
            Button::DPadUp => Self::Button(sdl3::gamepad::Button::DPadUp),
            Button::DPadDown => Self::Button(sdl3::gamepad::Button::DPadDown),
            Button::DPadLeft => Self::Button(sdl3::gamepad::Button::DPadLeft),
            Button::DPadRight => Self::Button(sdl3::gamepad::Button::DPadRight),
            Button::Misc1 => Self::Button(sdl3::gamepad::Button::Misc1),
            Button::RightPaddle1 => Self::Button(sdl3::gamepad::Button::RightPaddle1),
            Button::LeftPaddle1 => Self::Button(sdl3::gamepad::Button::LeftPaddle1),
            Button::RightPaddle2 => Self::Button(sdl3::gamepad::Button::RightPaddle2),
            Button::LeftPaddle2 => Self::Button(sdl3::gamepad::Button::LeftPaddle2),
            Button::Touchpad => Self::Button(sdl3::gamepad::Button::Touchpad),
            Button::RightTrigger => Self::Axis(sdl3::gamepad::Axis::TriggerRight),
            Button::LeftTrigger => Self::Axis(sdl3::gamepad::Axis::TriggerLeft),
            Button::Misc2 => Self::Button(sdl3::gamepad::Button::Misc2),
            Button::Misc3 => Self::Button(sdl3::gamepad::Button::Misc3),
            Button::Misc4 => Self::Button(sdl3::gamepad::Button::Misc4),
            Button::Misc5 => Self::Button(sdl3::gamepad::Button::Misc5),
        }
    }
}

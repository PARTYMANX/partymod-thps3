use std::{collections::HashMap, sync::Arc};

use partymod_config_common::{Button, Stick};
use sdl3::{gamepad::Axis, sys::joystick::SDL_JoystickID};

use crate::logger::{LogLevel, Logger};

// handles bindings, polling, and controllers
pub struct GamepadManager {
    max_players: usize,

    player_count: usize,
    player_slots: Vec<Option<usize>>,

    button_bindings: HashMap<String, Button>,
    stick_bindings: HashMap<String, Stick>,
    gamepads: Vec<sdl3::gamepad::Gamepad>,

    rumble_enabled: bool,

    logger: Option<Arc<dyn Logger>>,
}

impl GamepadManager {
    pub fn new(max_players: usize, logger: Option<Arc<dyn Logger>>) -> Self {
        let mut player_slots = Vec::with_capacity(max_players);
        player_slots.resize(max_players, None);

        Self {
            max_players,

            player_count: 0,
            player_slots,

            button_bindings: HashMap::new(),
            stick_bindings: HashMap::new(),
            gamepads: Vec::new(),

            rumble_enabled: true,

            logger,
        }
    }

    fn add_controller(&mut self, idx: u32, gamepad_subsystem: &sdl3::GamepadSubsystem) {
        let gamepad = match gamepad_subsystem.open(SDL_JoystickID(idx)) {
            Ok(v) => v,
            Err(_e) => {
                if let Some(logger) = &self.logger {
                    logger.log(
                        LogLevel::Error,
                        &format!("Failed to open controller {}!", idx),
                    );
                }
                return;
            }
        };

        if let Some(logger) = &self.logger {
            match gamepad.name() {
                Some(v) => {
                    logger.log(LogLevel::Info, &format!("Detected new controller: {}", v));
                }
                None => {
                    logger.log(LogLevel::Info, &format!("Detected new unnamed controller"));
                }
            }
        }

        self.gamepads.push(gamepad);
    }

    fn remove_controller(&mut self, id: u32) {
        let mut result = None;
        for (i, gamepad) in self.gamepads.iter().enumerate() {
            if gamepad.id().unwrap().0 == id {
                result = Some(i);
                break;
            }
        }

        match result {
            Some(v) => {
                for player in &mut self.player_slots {
                    if let Some(p) = player
                        && *p == v
                    {
                        *player = None;
                        self.player_count -= 1;
                    }
                }

                if let Some(logger) = &self.logger {
                    match self.gamepads[v].name() {
                        Some(v) => {
                            logger.log(LogLevel::Info, &format!("Lost controller: {}", v));
                        }
                        None => {
                            logger.log(LogLevel::Info, &format!("Lost unnamed controller"));
                        }
                    }
                }

                self.gamepads.remove(v);
            }
            None => {
                if let Some(logger) = &self.logger {
                    logger.log(
                        LogLevel::Error,
                        &format!("Failed to find controller {}!", id),
                    );
                }
            }
        }
    }

    pub fn add_button_binding(&mut self, name: &str, button: Button) {
        self.button_bindings.insert(name.to_owned(), button);
    }

    pub fn add_stick_binding(&mut self, name: &str, stick: Stick) {
        self.stick_bindings.insert(name.to_owned(), stick);
    }

    pub fn poll_button_binding(&self, player: usize, name: &str) -> (bool, i16) {
        let binding = match self.button_bindings.get(name) {
            Some(v) => v,
            None => return (false, 0),
        };

        if self.max_players == 1 {
            self.poll_button_binding_all(binding)
        } else {
            self.poll_button_binding_single(player, binding)
        }
    }

    fn poll_button_binding_all(&self, binding: &Button) -> (bool, i16) {
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

    fn poll_button_binding_single(&self, player: usize, binding: &Button) -> (bool, i16) {
        if let Some(idx) = self.player_slots[player] {
            let button = SDLButton::from(binding);
            let gamepad = &self.gamepads[idx];

            match button {
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
            }
        } else {
            (false, 0)
        }
    }

    pub fn poll_stick_binding(&self, player: usize, name: &str) -> (i16, i16) {
        let binding = match self.stick_bindings.get(name) {
            Some(v) => v,
            None => return (0, 0),
        };

        if self.max_players == 1 {
            self.poll_stick_binding_all(binding)
        } else {
            self.poll_stick_binding_single(player, binding)
        }
    }

    fn poll_stick_binding_all(&self, binding: &Stick) -> (i16, i16) {
        let mut result = (0, 0);
        let mut result_mag_sq = 0;

        for gamepad in &self.gamepads {
            let (x, y) = match binding {
                Stick::Left => (gamepad.axis(Axis::LeftX), gamepad.axis(Axis::LeftY)),
                Stick::Right => (gamepad.axis(Axis::RightX), gamepad.axis(Axis::RightY)),
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

    fn poll_stick_binding_single(&self, player: usize, binding: &Stick) -> (i16, i16) {
        if let Some(idx) = self.player_slots[player] {
            let gamepad = &self.gamepads[idx];

            match binding {
                Stick::Left => (gamepad.axis(Axis::LeftX), gamepad.axis(Axis::LeftY)),
                Stick::Right => (gamepad.axis(Axis::RightX), gamepad.axis(Axis::RightY)),
            }
        } else {
            (0, 0)
        }
    }

    pub fn set_rumble(&mut self, player: usize, high: u16, low: u16) {
        if !self.rumble_enabled {
            return;
        }

        let slot = self.player_slots[player];

        if let Some(idx) = slot {
            let _ = self.gamepads[idx].set_rumble(low, high, 0);
        }
    }

    pub fn enable_rumble(&mut self, player: usize, enabled: bool) {
        if !enabled {
            self.set_rumble(player, 0, 0);
        }

        self.rumble_enabled = enabled;
    }

    fn set_active(&mut self, idx: usize) {
        if self.player_slots[0] != Some(idx) {
            self.set_rumble(0, 0, 0); // cancel any remaining rumble

            self.player_slots[0] = Some(idx);

            if let Some(logger) = &self.logger {
                match self.gamepads[idx].name() {
                    Some(v) => {
                        logger.log(LogLevel::Info, &format!("Controller {} now active", v));
                    }
                    None => {
                        logger.log(LogLevel::Info, &format!("Controller {} now active", idx));
                    }
                }
            }
        }
    }

    fn add_player(&mut self, idx: usize) {
        if self.player_count == self.max_players || self.player_slots.contains(&Some(idx)) {
            return;
        }

        for (i, slot) in self.player_slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(idx);

                if let Some(logger) = &self.logger {
                    match self.gamepads[idx].name() {
                        Some(v) => {
                            logger.log(
                                LogLevel::Info,
                                &format!("Controller {} set to player {}", v, i + 1),
                            );
                        }
                        None => {
                            logger.log(
                                LogLevel::Info,
                                &format!("Controller {} set to player {}", idx, i + 1),
                            );
                        }
                    }
                }

                // TODO: set lockout for each player after first

                return;
            }
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

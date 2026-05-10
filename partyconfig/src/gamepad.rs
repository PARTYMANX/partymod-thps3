use partymod_config::GAMEPAD_BINDS;
use partymod_config_common::{BindType, Button, Stick};
use pgui::{component::Component, container::{horizontal, vertical}, dropdown::dropdown, groupbox::groupbox, layout::{Size, VerticalOffset}, text::text};

use crate::{AppState, ini::ConfigFile};

const BUTTON_STRINGS: [&str; 22] = [
    "Unbound",
    "A/Cross",
    "B/Circle",
    "X/Square",
    "Y/Triangle",
    "D-Pad Up",
    "D-Pad Down",
    "D-Pad Left",
    "D-Pad Right",
    "LB/L1",
    "RB/R1",
    "LR/L2",
    "RT/R2",
    "Left Stick/L3",
    "Right Stick/R3",
    "Start/Options",
    "Back/Select",
    "Left Paddle 1",
    "Right Paddle 1",
    "Left Paddle 2",
    "Right Paddle 2",
    "Touchpad",
];

const BUTTON_VALUES: [Option<Button>; 22] = [
    None,
    Some(Button::A),
    Some(Button::B),
    Some(Button::X),
    Some(Button::Y),
    Some(Button::DPadUp),
    Some(Button::DPadDown),
    Some(Button::DPadLeft),
    Some(Button::DPadRight),
    Some(Button::LeftShoulder),
    Some(Button::RightShoulder),
    Some(Button::LeftTrigger),
    Some(Button::RightTrigger),
    Some(Button::LeftStick),
    Some(Button::RightStick),
    Some(Button::Start),
    Some(Button::Select),
    Some(Button::LeftPaddle1),
    Some(Button::RightPaddle1),
    Some(Button::LeftPaddle2),
    Some(Button::RightPaddle2),
    Some(Button::Touchpad),
];

const BUTTON_VALUE_REVERSE: [u32; 27] = [
    1,
    2,
    3,
    4,
    16,
    0,
    15,
    13,
    14,
    9,
    10,
    5,
    6,
    7,
    8,
    0,
    18,
    17,
    20,
    19,
    21,
    12,
    11,
    0,
    0,
    0,
    0,
];

const STICK_STRINGS: [&str; 3] = [
    "Unbound",
    "Left Stick",
    "Right Stick",
];

const STICK_VALUES: [Option<Stick>; 3] = [
    None,
    Some(Stick::Left),
    Some(Stick::Right),
];

const STICK_VALUE_REVERSE: [u32; 2] = [
    1,
    2,
];

pub struct GamepadState {
    binds: Vec<BindType>,
}

impl GamepadState {
    pub fn new(config_file: &ConfigFile) -> Self {
        let mut binds = Vec::with_capacity(GAMEPAD_BINDS.len());

        for keybind in &GAMEPAD_BINDS {
            let bind = match &keybind.default {
                BindType::Button { value: default } => {
                    let d = match default {
                        None => -1,
                        Some(v) => *v as i32,
                    };

                    let value = config_file.get_config_int("Gamepad", keybind.key, d);

                    let button = if value == -1 {
                        None
                    } else {
                        Button::from_i32(value)
                    };

                    BindType::Button { value: button }
                },
                BindType::Stick { value: default } => {
                    let d = match default {
                        None => -1,
                        Some(v) => *v as i32,
                    };

                    let value = config_file.get_config_int("Gamepad", keybind.key, d);

                    let button = if value == -1 {
                        None
                    } else {
                        Stick::from_i32(value)
                    };

                    BindType::Stick { value: button }
                },
            };

            binds.push(bind);
        }

        Self {
            binds,
        }
    }

    pub fn save(&self, config_file: &ConfigFile) {
        for (idx, bind) in GAMEPAD_BINDS.iter().enumerate() {
            let value = match &self.binds[idx] {
                BindType::Button { value: default } => {
                    match default {
                        Some(v) => *v as i32,
                        None => -1,
                    }
                }
                BindType::Stick { value: default } => {
                    match default {
                        Some(v) => *v as i32,
                        None => -1,
                    }
                }
            };

            config_file.set_config_int("Gamepad", bind.key, value);
        }
    }
}

impl Default for GamepadState {
    fn default() -> Self {
        let mut binds = Vec::with_capacity(GAMEPAD_BINDS.len());

        for bind in &GAMEPAD_BINDS {
            binds.push(bind.default);
        }

        Self {
            binds,
        }
    }
}

pub fn gamepad_bind_row(idx: usize) -> Component<AppState> {
    let mut option_strings = Vec::new();

    match GAMEPAD_BINDS[idx].default {
        BindType::Button { .. } => {
            for string in &BUTTON_STRINGS {
                option_strings.push(string.to_string());
            }
        },
        BindType::Stick { .. } => {
            for string in &STICK_STRINGS {
                option_strings.push(string.to_string());
            }
        }
    }

    horizontal(vec![
        text(format!("{}:", GAMEPAD_BINDS[idx].display_name))
        .v_position(VerticalOffset::AlignTop(2))
        .height(Size::Exact(16))
        .width(Size::Exact(75))
        .into(),
        dropdown(option_strings)
        .on_select(move |app_state: &mut AppState, selected| {
            let value = match app_state.gamepad_state.binds[idx] {
                BindType::Button { .. } => {
                    BindType::Button {
                        value: BUTTON_VALUES[selected as usize]
                    }
                },
                BindType::Stick { .. } => {
                    BindType::Stick {
                        value: STICK_VALUES[selected as usize]
                    }
                },
            };

            app_state.gamepad_state.binds[idx] = value;
        })
        .state_hook(move |app_state: &AppState, dropdown_state| {
            match app_state.gamepad_state.binds[idx] {
                BindType::Button { value } => {
                    dropdown_state.selected = match value {
                        None => 0,
                        Some(v) => BUTTON_VALUE_REVERSE[v as usize],
                    };
                },
                BindType::Stick { value } => {
                    dropdown_state.selected = match value {
                        None => 0,
                        Some(v) => STICK_VALUE_REVERSE[v as usize],
                    };
                },
            }
        })
        .width(Size::Exact(85))
        .height(Size::Exact(20))
        .into(),
    ])
    .into()
}

pub fn gamepad_page(width: u32, height: u32) -> Component<AppState> {
    horizontal(vec![
        groupbox(
            "Actions".to_string(),
            vertical(vec![
                gamepad_bind_row(0),
                gamepad_bind_row(1),
                gamepad_bind_row(2),
                gamepad_bind_row(3),
                gamepad_bind_row(4),
                gamepad_bind_row(5),
                gamepad_bind_row(6),
                gamepad_bind_row(7),
                gamepad_bind_row(8),
            ])
            .v_position(VerticalOffset::AlignTop(8))
            .spacing(18)
            .into()
        )
        .width(Size::Exact(width / 2))
        .height(Size::Exact(height))
        .into(),
        vertical(vec![
            groupbox(
                "Skater Controls".to_string(),
                vertical(vec![
                    gamepad_bind_row(9),
                    gamepad_bind_row(10),
                    gamepad_bind_row(11),
                    gamepad_bind_row(12),
                    gamepad_bind_row(13),
                ])
                .v_position(VerticalOffset::AlignTop(8))
                .spacing(18)
                .into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact((height as f32 * (7.0 / 12.0)) as u32))
            .into(),
            groupbox(
                "Other".to_string(),
                vertical(vec![
                    gamepad_bind_row(14),
                    gamepad_bind_row(15),
                    gamepad_bind_row(16),
                ])
                .v_position(VerticalOffset::AlignTop(8))
                .spacing(22)
                .into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact((height as f32 * (5.0 / 12.0)) as u32))
            .into(),
        ])
        .into()
    ])
    .into()
}
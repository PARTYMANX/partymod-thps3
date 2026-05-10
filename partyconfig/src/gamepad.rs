use pgui::{component::Component, container::{horizontal, vertical}, dropdown::dropdown, groupbox::groupbox, layout::{Size, VerticalOffset}, text::text};

use crate::{AppState, ini::ConfigFile};

#[derive(Clone, Copy)]
#[repr(i32)]
enum Button {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Select = 4,
    Home = 5,
    Start = 6,
    LeftStick = 7,
    RightStick = 8,
    LeftShoulder = 9,
    RightShoulder = 10,
    DPadUp = 11,
    DPadDown = 12,
    DPadLeft = 13,
    DPadRight = 14,
    Misc1 = 15,
    RightPaddle1 = 16,
    LeftPaddle1 = 17,
    RightPaddle2 = 18,
    LeftPaddle2 = 19,
    Touchpad = 20,
    RightTrigger = 21,
    LeftTrigger = 22,
    Misc2 = 23,
    Misc3 = 24,
    Misc4 = 25,
    Misc5 = 26,
}

impl Button {
    fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::X),
            3 => Some(Self::Y),
            4 => Some(Self::Select),
            5 => Some(Self::Home),
            6 => Some(Self::Start),
            7 => Some(Self::LeftStick),
            8 => Some(Self::RightStick),
            9 => Some(Self::LeftShoulder),
            10 => Some(Self::RightShoulder),
            11 => Some(Self::DPadUp),
            12 => Some(Self::DPadDown),
            13 => Some(Self::DPadLeft),
            14 => Some(Self::DPadRight),
            15 => Some(Self::Misc1),
            16 => Some(Self::RightPaddle1),
            17 => Some(Self::LeftPaddle1),
            18 => Some(Self::RightPaddle2),
            19 => Some(Self::LeftPaddle2),
            20 => Some(Self::Touchpad),
            21 => Some(Self::RightTrigger),
            22 => Some(Self::LeftTrigger),
            23 => Some(Self::Misc2),
            24 => Some(Self::Misc3),
            25 => Some(Self::Misc4),
            26 => Some(Self::Misc5),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(i32)]
enum Stick {
    Left = 0,
    Right = 1,
}

impl Stick {
    fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
enum BindType {
    Button { value: Option<Button> },
    Stick { value: Option<Stick> },
}

struct GamepadBind {
    display_name: &'static str,
    key: &'static str,
    default: BindType,
}

const GAMEPAD_BINDS: [GamepadBind; 17] = [
    GamepadBind {
        display_name: "Ollie",
        key: "Ollie",
        default: BindType::Button {
            value: Some(Button::A),
        },
    },
    GamepadBind {
        display_name: "Grab",
        key: "Grab",
        default: BindType::Button {
            value: Some(Button::B),
        },
    },
    GamepadBind {
        display_name: "Flip",
        key: "Flip",
        default: BindType::Button {
            value: Some(Button::X),
        },
    },
    GamepadBind {
        display_name: "Grind",
        key: "Grind",
        default: BindType::Button {
            value: Some(Button::Y),
        },
    },
    GamepadBind {
        display_name: "Spin Left",
        key: "SpinLeft",
        default: BindType::Button {
            value: Some(Button::LeftShoulder),
        },
    },
    GamepadBind {
        display_name: "Spin Right",
        key: "SpinRight",
        default: BindType::Button {
            value: Some(Button::RightShoulder),
        },
    },
    GamepadBind {
        display_name: "Nollie",
        key: "Nollie",
        default: BindType::Button {
            value: Some(Button::LeftTrigger),
        },
    },
    GamepadBind {
        display_name: "Switch",
        key: "Switch",
        default: BindType::Button {
            value: Some(Button::RightTrigger),
        },
    },
    GamepadBind {
        display_name: "Pause",
        key: "Pause",
        default: BindType::Button {
            value: Some(Button::Start),
        },
    },
    GamepadBind {
        display_name: "Forward",
        key: "Forward",
        default: BindType::Button {
            value: Some(Button::DPadUp),
        },
    },
    GamepadBind {
        display_name: "Backward",
        key: "Backward",
        default: BindType::Button {
            value: Some(Button::DPadDown),
        },
    },
    GamepadBind {
        display_name: "Left",
        key: "Left",
        default: BindType::Button {
            value: Some(Button::DPadLeft),
        },
    },
    GamepadBind {
        display_name: "Right",
        key: "Right",
        default: BindType::Button {
            value: Some(Button::DPadRight),
        },
    },
    GamepadBind {
        display_name: "Move Stick",
        key: "MovementStick",
        default: BindType::Stick {
            value: Some(Stick::Left),
        },
    },
    GamepadBind {
        display_name: "Camera Stick",
        key: "CameraStick",
        default: BindType::Stick {
            value: Some(Stick::Right),
        },
    },
    GamepadBind {
        display_name: "View Toggle",
        key: "ViewToggle",
        default: BindType::Button {
            value: Some(Button::Select),
        },
    },
    GamepadBind {
        display_name: "Swivel Lock",
        key: "SwivelLock",
        default: BindType::Button {
            value: Some(Button::RightStick),
        },
    },
];

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
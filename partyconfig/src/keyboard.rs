use pgui::{component::Component, container::{horizontal, vertical}, groupbox::groupbox, layout::{HorizontalOffset, Size, VerticalOffset}, text::text, textbox::{TextboxState, textbox}};
use sdl3::sys::scancode::{self, SDL_Scancode};

use crate::{AppState, ini::ConfigFile};

struct Keybind {
    display_name: &'static str,
    key: &'static str,
    default: Option<SDL_Scancode>,
}

const KEYBINDS: [Keybind; 19] = [
    Keybind {
        display_name: "Ollie",
        key: "Ollie",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Grab",
        key: "Grab",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Flip",
        key: "Flip",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Grind",
        key: "Grind",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Spin Left",
        key: "SpinLeft",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Spin Right",
        key: "SpinRight",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Nollie",
        key: "Nollie",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Switch",
        key: "Switch",
        default: Some(scancode::SDL_SCANCODE_KP_2),
    },
    Keybind {
        display_name: "Pause",
        key: "Pause",
        default: None,
    },
    Keybind {
        display_name: "Forward",
        key: "Forward",
        default: None,
    },
    Keybind {
        display_name: "Backward",
        key: "Backward",
        default: None,
    },
    Keybind {
        display_name: "Left",
        key: "Left",
        default: None,
    },
    Keybind {
        display_name: "Right",
        key: "Right",
        default: None,
    },
    Keybind {
        display_name: "Camera Up",
        key: "CameraUp",
        default: None,
    },
    Keybind {
        display_name: "Camera Down",
        key: "CameraDown",
        default: None,
    },
    Keybind {
        display_name: "Camera Left",
        key: "CameraLeft",
        default: None,
    },
    Keybind {
        display_name: "Camera Right",
        key: "CameraRight",
        default: None,
    },
    Keybind {
        display_name: "View Toggle",
        key: "ViewToggle",
        default: None,
    },
    Keybind {
        display_name: "Swivel Lock",
        key: "SwivelLock",
        default: None,
    },
];

pub struct KeyboardState {
    is_setting_key: Option<usize>,
    key_binds: Vec<Option<SDL_Scancode>>,
}

impl KeyboardState {
    pub fn new(config_file: &ConfigFile) -> Self {
        let mut key_binds = Vec::with_capacity(KEYBINDS.len());

        for keybind in &KEYBINDS {
            let default = match keybind.default {
                None => -1,
                Some(v) => v.0,
            };

            let value = config_file.get_config_int("Keybinds", keybind.key, default);

            let scancode = if value == -1 {
                None
            } else {
                Some(SDL_Scancode(value))
            };

            key_binds.push(scancode);
        }

        Self {
            is_setting_key: None,
            key_binds,
        }
    }
}

fn keybind_row(idx: usize) -> Component<AppState> {
    horizontal(vec![
        text(format!("{}:", KEYBINDS[idx].display_name))
        .v_position(VerticalOffset::AlignTop(2))
        .height(Size::Exact(16))
        .width(Size::Exact(80))
        .into(),
        textbox("".to_string())
        .on_focus(move |app_state: &mut AppState, _text| {
            app_state.keyboard_state.is_setting_key = Some(idx);
        })
        .state_hook(move |app_state: &AppState, textbox_state: &mut TextboxState| {
            let setting_this = match app_state.keyboard_state.is_setting_key {
                None => false,
                Some(v) => v == idx,
            };

            if setting_this {
                textbox_state.text = "Press a key...".to_string();
            } else {
                match app_state.keyboard_state.key_binds[idx] {
                    Some(v) => {
                        textbox_state.text = format!("{}", v.0);
                    },
                    None => {
                        textbox_state.text = "Unbound".to_string();
                    },
                }
            }
        })
        .h_position(HorizontalOffset::AlignRight(0))
        .width(Size::Exact(75))
        .height(Size::Exact(20))
        .into(),
    ])
    .into()
}

pub fn keyboard_page(width: u32, height: u32) -> Component<AppState> {
    horizontal(vec![
        groupbox(
            "Actions".to_string(),
            vertical(vec![
                keybind_row(0),
                keybind_row(1),
                keybind_row(2),
                keybind_row(3),
                keybind_row(4),
                keybind_row(5),
                keybind_row(6),
                keybind_row(7),
                keybind_row(8),
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
                    keybind_row(9),
                    keybind_row(10),
                    keybind_row(11),
                    keybind_row(12),
                ])
                .v_position(VerticalOffset::AlignTop(8))
                .spacing(10)
                .into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact((height as f32 * (13.0 / 32.0)) as u32))
            .into(),
            groupbox(
                "Camera Controls".to_string(),
                vertical(vec![
                    keybind_row(13),
                    keybind_row(14),
                    keybind_row(15),
                    keybind_row(16),
                    keybind_row(17),
                    keybind_row(18),
                ])
                .v_position(VerticalOffset::AlignTop(8))
                .spacing(12)
                .into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact((height as f32 * (19.0 / 32.0)) as u32))
            .into(),
        ])
        .into()
    ])
    .into()
}
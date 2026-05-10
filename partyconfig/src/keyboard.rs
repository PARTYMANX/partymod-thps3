use partymod_config::KEYBINDS;
use pgui::{component::Component, container::{horizontal, vertical}, groupbox::groupbox, layout::{HorizontalOffset, Size, VerticalOffset}, text::text, textbox::{TextboxState, textbox}};
use sdl3::sys::keycode::SDL_KMOD_NONE;

use crate::{AppState, ini::ConfigFile};

pub struct KeyboardState {
    is_setting_key: Option<usize>,
    key_binds: Vec<Option<sdl3::keyboard::Scancode>>,
}

impl KeyboardState {
    pub fn new(config_file: &ConfigFile) -> Self {
        let mut key_binds = Vec::with_capacity(KEYBINDS.len());

        for keybind in &KEYBINDS {
            let default = match keybind.default {
                None => -1,
                Some(v) => v.to_i32(),
            };

            let value = config_file.get_config_int("Keybinds", keybind.key, default);

            let scancode = if value == -1 {
                None
            } else {
                sdl3::keyboard::Scancode::from_i32(value)
            };

            key_binds.push(scancode);
        }

        Self {
            is_setting_key: None,
            key_binds,
        }
    }

    pub fn save(&self, config_file: &ConfigFile) {
        for (idx, keybind) in KEYBINDS.iter().enumerate() {
            let value = match self.key_binds[idx] {
                Some(v) => v.to_i32(),
                None => -1,
            };

            config_file.set_config_int("Keybinds", keybind.key, value);
        }
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        let mut key_binds = Vec::with_capacity(KEYBINDS.len());

        for keybind in &KEYBINDS {
            key_binds.push(keybind.default);
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
                        textbox_state.text = format!("{}", app_state.sdl_key_context.get_key_name(v));
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

pub struct SDLKeyContext {
    context: sdl3::Sdl,
}

impl SDLKeyContext {
    pub fn new() -> Self {
        let context = sdl3::init().unwrap();
        //let video_subsystem = context.video().unwrap();

        // we create this window to populate key names
        /*let _temp_window = video_subsystem.window(
            "you're not supposed to see this...",
            0,
            0,
        ).hidden();*/

        Self {
            context,
        }
    }

    fn get_key_name(&self, scancode: sdl3::keyboard::Scancode) -> String {
        let key = sdl3::keyboard::Keycode::from_scancode(scancode, SDL_KMOD_NONE, false);

        match key {
            Some(v) => v.name(),
            None => "Unknown".to_string(),
        }
    }

    pub fn do_key_bind(&self, keyboard_state: &mut KeyboardState) {
        let idx = match keyboard_state.is_setting_key {
            Some(v) => v,
            None => return,
        };

        let video_subsystem = self.context.video().unwrap();

        let _window = video_subsystem.window(
            "Press Key...",
            1,
            1,
        ).input_grabbed()
        .borderless()
        .build().unwrap();

        let mut event_pump = self.context.event_pump().unwrap();

        'inputloop: loop {
            for event in event_pump.poll_iter() {
                match event {
                    sdl3::event::Event::KeyDown { scancode, .. } => {
                        keyboard_state.key_binds[idx] = scancode;
                        break 'inputloop;
                    },
                    sdl3::event::Event::Quit { .. } => {
                        break 'inputloop;
                    },
                    sdl3::event::Event::Window { win_event, .. } => {
                        match win_event {
                            sdl3::event::WindowEvent::Hidden |
                            sdl3::event::WindowEvent::FocusLost |
                            sdl3::event::WindowEvent::Minimized => {
                                break 'inputloop;
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }

        keyboard_state.is_setting_key = None;
    }
}
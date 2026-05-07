use pgui::{app, button::button, container::{container, horizontal, vertical}, layout::{HorizontalOffset, Size, VerticalOffset}, tabs::{Tab, tabs}, window::window};

use crate::general::GeneralState;

mod general;
mod keyboard;
mod gamepad;
mod ini;

struct AppState {
    pub config_file: ini::ConfigFile,
    pub resolution_info: general::ResolutionInfo,

    pub general_state: general::GeneralState,
    should_quit: bool,
}

impl AppState {
    fn new(config_path: &std::path::Path) -> Self {
        let config_file = ini::ConfigFile::new(config_path);
        let resolution_info = general::ResolutionInfo::init();

        let general_state = general::GeneralState::new(&config_file, &resolution_info);

        Self {
            config_file,
            resolution_info,
            general_state,
            should_quit: false,
        }
    }

    fn save_settings(&self) {
        self.general_state.save(&self.config_file, &self.resolution_info);
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}

fn main() {
    let exe_file = match std::env::current_exe() {
        Ok(exe_file) => exe_file,
        Err(e) => panic!("failed to get current exe path: {e}"),
    };

    let config_path = exe_file.parent().unwrap().join("partymod.ini");

    let state = AppState::new(config_path.as_path());

    let window_width = 400;
    let window_height = 450;

    let tab_height = window_height - 26 - (8 * 2);
    
    let page_width = window_width - (8 * 2);
    let page_height = tab_height - (8 * 2) - 16;

    let resolution_list = state.resolution_info.get_display_option_list();

    app::run(
        state,
        vertical(vec![
            container(
                tabs(vec![
                    Tab { 
                        label: "General".to_string(), 
                        child: general::general_page(page_width, page_height, resolution_list),
                    },
                    Tab { 
                        label: "Keyboard".to_string(), 
                        child: keyboard::keyboard_page(page_width, page_height),
                    },
                    Tab { 
                        label: "Gamepad".to_string(), 
                        child: gamepad::gamepad_page(page_width, page_height),
                    },
                ])
                .width(Size::Fill)
                .height(Size::Fill)
                .v_position(VerticalOffset::AlignTop(8))
                .h_padding(8)
                .into(),
            )
            .width(Size::Fill)
            .height(Size::Exact(tab_height))
            .into(),
            container(
                horizontal(vec![
                    button("Restore Defaults".to_string())
                    .on_press(|app_state: &mut AppState| {
                        app_state.general_state = GeneralState::default();
                    })
                    .height(Size::Exact(26))
                    .into(),
                    horizontal(vec![
                        button("Cancel".to_string())
                        .width(Size::Exact(80))
                        .height(Size::Exact(26))
                        .on_press(|app_state: &mut AppState| {
                            app_state.quit();
                        })
                        .into(),
                        button("OK".to_string())
                        .width(Size::Exact(80))
                        .height(Size::Exact(26))
                        .on_press(|app_state: &mut AppState| {
                            app_state.save_settings();
                            app_state.quit();
                        })
                        .into(),
                    ])
                    .h_position(HorizontalOffset::AlignRight(0))
                    .spacing(8)
                    .into()
                ])
                .into()
            )
            .v_position(pgui::layout::VerticalOffset::AlignBottom(0))
            .h_padding(8)
            .v_padding(8)
            .into()
        ])
        .into(),
        window("PARTYMOD Configuration".to_string())
        .dimensions(400, 450)
        .state_hook(|st, window_state| {
            window_state.should_quit = st.should_quit;
        }),
    );
}

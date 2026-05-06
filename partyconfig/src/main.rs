use pgui::{app, button::button, container::{container, horizontal, vertical}, layout::{HorizontalOffset, Size, VerticalOffset}, tabs::{Tab, tabs}, text::text, window::window};

mod general;

struct AppState {
    pub resolution_info: general::ResolutionInfo,

    pub general_state: general::GeneralState,
    should_quit: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            resolution_info: general::ResolutionInfo::init(),
            general_state: general::GeneralState::new(),
            should_quit: false,
        }
    }

    fn quit(&mut self) {
        self.should_quit = true;
    }
}

fn main() {
    let state = AppState::new();

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
                        child: text("well we need something here".to_string()).into(),
                    },
                    Tab { 
                        label: "Gamepad".to_string(), 
                        child: text("well we need something here".to_string()).into(),
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
                    .height(Size::Exact(26))
                    .into(),
                    horizontal(vec![
                        button("Cancel".to_string())
                        .width(Size::Exact(80))
                        .height(Size::Exact(26))
                        .on_press(|st: &mut AppState| {
                            st.quit();
                        })
                        .into(),
                        button("OK".to_string())
                        .width(Size::Exact(80))
                        .height(Size::Exact(26))
                        .on_press(|st: &mut AppState| {
                            st.quit();
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

mod general;

fn main() {
    let state = 0;

    let window_width = 400;
    let window_height = 450;

    let tab_height = window_height - 26 - (8 * 2);
    
    let page_width = window_width - (8 * 2);
    let page_height = tab_height - (8 * 2) - 16;

    pgui::app::run(
        state,
        pgui::container::vertical(
            vec![
                pgui::container::container(
                    pgui::tabs::tabs(
                        vec![
                            pgui::tabs::Tab { 
                                label: "General".to_string(), 
                                child: general::general_page(page_width, page_height),
                            },
                            pgui::tabs::Tab { 
                                label: "Keyboard".to_string(), 
                                child: pgui::text::text("well we need something here".to_string()).into(),
                            },
                            pgui::tabs::Tab { 
                                label: "Gamepad".to_string(), 
                                child: pgui::text::text("well we need something here".to_string()).into(),
                            },
                        ]
                    )
                    .width(pgui::layout::Size::Fill)
                    .height(pgui::layout::Size::Fill)
                    .v_position(pgui::layout::VerticalOffset::AlignTop(8))
                    .h_padding(8)
                    .into(),
                )
                .width(pgui::layout::Size::Fill)
                .height(pgui::layout::Size::Exact(tab_height))
                .into(),
                pgui::container::container(
                    pgui::container::horizontal(
                        vec![
                            pgui::button::button("Restore Defaults".to_string())
                            .height(pgui::layout::Size::Exact(26))
                            .into(),
                            pgui::container::horizontal(
                                vec![
                                    pgui::button::button("Cancel".to_string())
                                    .width(pgui::layout::Size::Exact(80))
                                    .height(pgui::layout::Size::Exact(26))
                                    .into(),
                                    pgui::button::button("OK".to_string())
                                    .width(pgui::layout::Size::Exact(80))
                                    .height(pgui::layout::Size::Exact(26))
                                    .into(),
                                ]
                            )
                            .h_position(pgui::layout::HorizontalOffset::AlignRight(0))
                            .spacing(8)
                            .into()
                        ]
                    )
                    .into()
                )
                .v_position(pgui::layout::VerticalOffset::AlignBottom(0))
                .h_padding(8)
                .v_padding(8)
                .into()
            ]
        )
        .into(),
        pgui::window::window("PARTYMOD Configuration".to_string()).dimensions(400, 450),
    );
}

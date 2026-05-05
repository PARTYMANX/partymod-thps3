pub fn general_page(width: u32, height: u32) -> pgui::component::Component<i32> {
    pgui::container::vertical(
        vec![
            pgui::groupbox::groupbox(
                "Resolution".to_string(),
                pgui::container::vertical(vec![
                    pgui::dropdown::dropdown(vec![
                        "Default Desktop Resolution".to_string(),
                    ])
                    .into(),
                    pgui::checkbox::checkbox("Use Custom Resolution".to_string())
                    .into(),
                    pgui::container::horizontal(vec![
                        pgui::text::text("Width:".to_string()).into(),
                        pgui::textbox::textbox("".to_string())
                        .width(pgui::layout::Size::Exact(64))
                        .into(),
                        pgui::text::text("Height:".to_string()).into(),
                        pgui::textbox::textbox("".to_string())
                        .width(pgui::layout::Size::Exact(64))
                        .into(),
                    ])
                    .into(),
                    pgui::checkbox::checkbox("Windowed".to_string())
                    .into(),
                    pgui::checkbox::checkbox("Borderless".to_string())
                    .into(),
                ])
                .into()
            )
            .width(pgui::layout::Size::Exact(width))
            .height(pgui::layout::Size::Exact(height / 2))
            .into(),
            pgui::container::horizontal(
                vec![
                    pgui::groupbox::groupbox(
                        "Graphics".to_string(),
                        pgui::text::text("well we need something here".to_string()).into()
                    )
                    .width(pgui::layout::Size::Exact(width / 2))
                    .height(pgui::layout::Size::Exact(height / 2))
                    .into(),
                    pgui::groupbox::groupbox(
                        "Miscellaneous".to_string(),
                        pgui::text::text("well we need something here".to_string()).into()
                    )
                    .width(pgui::layout::Size::Exact(width / 2))
                    .height(pgui::layout::Size::Exact(height / 2))
                    .into(),
                ]
            )
            .into()
        ]
    )
    .into()
}
use pgui::{component::Component, container::{horizontal, vertical}, groupbox::groupbox, layout::Size, text::text};

use crate::AppState;

pub fn gamepad_page(width: u32, height: u32) -> Component<AppState> {
    horizontal(vec![
        groupbox(
            "Actions".to_string(),
            text("well we need something here".to_string()).into()
        )
        .width(Size::Exact(width / 2))
        .height(Size::Exact(height))
        .into(),
        vertical(vec![
            groupbox(
                "Skater Controls".to_string(),
                text("well we need something here".to_string()).into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact(height / 2))
            .into(),
            groupbox(
                "Other".to_string(),
                text("well we need something here".to_string()).into()
            )
            .width(Size::Exact(width / 2))
            .height(Size::Exact(height / 2))
            .into(),
        ])
        .into()
    ])
    .into()
}
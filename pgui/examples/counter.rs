use pgui::{app, button, container, layout, window};

fn main() {
    let state = 0;
    app::run(
        state,
        container::horizontal(vec![
            button::button("👍".to_string())
                .on_press(|st| {
                    *st += 1;
                })
                .into(),
            button::button("".to_string())
                .state_hook(|st, b| {
                    b.label = format!("{}", st);
                })
                .into(),
            button::button("👎".to_string())
                .on_press(|st| {
                    *st -= 1;
                })
                .into(),
        ])
        .spacing(16)
        .h_position(layout::HorizontalOffset::Center)
        .v_position(layout::VerticalOffset::Center)
        .into(),
        window::window("Counter".to_string()).dimensions(500, 500),
    );
}

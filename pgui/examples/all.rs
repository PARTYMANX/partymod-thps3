use pgui::{app, button, checkbox, container, layout, text, window};

fn main() {
    let state = 0;
    app::run(
        state,
        container::vertical(vec![
            button::button("Button".to_string())
                .on_press(|st| {
                    println!("Pressed!");
                })
                .into(),
            checkbox::checkbox("Checkbox".to_string())
                .on_toggle(|st, c| {
                    if c {
                        println!("Checked!");
                    } else {
                        println!("Unchecked!");
                    }
                })
                .into(),
            text::text("Text".to_string())
                .into(),
        ])
        .spacing(16)
        .h_position(layout::HorizontalOffset::Center)
        .v_position(layout::VerticalOffset::Center)
        .into(),
        window::window("All Components".to_string()).dimensions(500, 500),
    );
}
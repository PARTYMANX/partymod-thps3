use pgui::{app, button, checkbox, container, groupbox, layout, tabs, text, window};

fn main() {
    let state = 0;
    app::run(
        state,
        tabs::tabs(vec![
            tabs::Tab {
                label: "Tab 1".to_string(),
                child: groupbox::groupbox(
                    "Groupbox".to_string(),
                    container::vertical(vec![
                        button::button("Button".to_string())
                            .on_press(|_st| {
                                println!("Pressed!");
                            })
                            .into(),
                        checkbox::checkbox("Checkbox".to_string())
                            .on_toggle(|_st, c| {
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
                    .h_position(layout::HorizontalOffset::AlignLeft(0))
                    .v_position(layout::VerticalOffset::AlignTop(0))
                    .into(),
                )
                .width(layout::Size::Fill)
                .height(layout::Size::Fill)
                .into(),
            },
            tabs::Tab {
                label: "Tab 2".to_string(),
                child: text::text("This is tab 2!!!! ".to_string()).into(),
            },
            tabs::Tab {
                label: "Tab 3".to_string(),
                child: text::text("This is tab 3!!!! ".to_string()).into(),
            },
        ])
        .width(layout::Size::Fill)
        .height(layout::Size::Fill)
        .into(),
        window::window("All Components".to_string()).dimensions(500, 500),
    );
}
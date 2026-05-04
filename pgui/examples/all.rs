use pgui::{
    app, button, checkbox, container, dropdown, groupbox, layout, tabs, text, textbox, window,
};

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
                        text::text("Text".to_string()).into(),
                        dropdown::dropdown(vec![
                            "Option 1".to_string(),
                            "Option 2".to_string(),
                            "Option 3".to_string(),
                        ])
                        .on_select(|_st, i| {
                            println!("Selected {}!", i);
                        })
                        .into(),
                        textbox::textbox("Textbox".to_string())
                            .on_focus(|_st, t| {
                                println!("Focused {}!", t);
                            })
                            .on_unfocus(|_st, t| {
                                println!("Unfocused {}!", t);
                            })
                            .on_change(|_st, t| {
                                println!("Changed {}!", t);
                            })
                            .width(layout::Size::Exact(100))
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

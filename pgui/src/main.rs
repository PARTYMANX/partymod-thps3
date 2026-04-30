use crate::win32::app;

mod layout;
mod button;
mod component;
mod container;
mod genarena;
mod syncunsafecell;
mod win32;
mod window;

fn main() {
    let state = 0;
    app::run(
        state,
        container::horizontal(vec![
            button::button("👍".to_string())
                .on_press(|st| {
                    *st += 1;
                    println!("Pressed {} times!", st);
                })
                .into(),
            button::button("👎".to_string())
                .on_press(|st| {
                    *st -= 1;
                    println!("Pressed {} times!", st);
                })
                .into(),
        ]).spacing(16).h_position(layout::HorizontalOffset::Center).v_position(layout::VerticalOffset::Center).into(),
        window::window("Test Window".to_string()).dimensions(500, 500)
    );
}

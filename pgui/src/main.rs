use crate::win32::app;

mod button;
mod component;
mod syncunsafecell;
mod win32;

fn main() {
    let state = 0;
    app::run(
        state,
        &vec![
            button::button("+".to_string())
                .on_press(|st| {
                    *st += 1;
                    println!("Pressed {} times!", st);
                })
                .into(),
            button::button("-".to_string())
                .on_press(|st| {
                    *st -= 1;
                    println!("Pressed {} times!", st);
                })
                .into(),
        ],
    );
}

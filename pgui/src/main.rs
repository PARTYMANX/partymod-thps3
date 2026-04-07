use crate::win32::app;

mod syncunsafecell;
mod win32;
mod component;
mod button;

fn main() {
    let state = 0;
    app::run(
        state,
        &vec![
            button::button("Button".to_string())
                .on_press(|st| {
                    *st += 1;
                    println!("Pressed {} times!", st);
                }
            ).into(),
            button::button("Button".to_string())
                .on_press(|st| {
                    *st += 1;
                    println!("Pressed {} times!", st);
                }
            ).into(),
        ]
    );
    
}

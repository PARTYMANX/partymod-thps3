use crate::win32::app;

mod syncunsafecell;
mod win32;

fn main() {
    let state = 0;
    app::run(state, |st| {
        *st += 1;
        println!("Pressed {} times!", st);
    });
}

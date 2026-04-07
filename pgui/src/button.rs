pub struct Button<T> {
    pub(crate) label: String,
    pub(crate) on_press: Option<fn(&mut T)>,
}

impl<T> Button<T> {
    pub fn on_press(mut self, func: fn(&mut T)) -> Self {
        self.on_press = Some(func);
        self
    }
}

pub fn button<T>(label: String) -> Button<T> {
    Button {
        label,
        on_press: None,
    }
}
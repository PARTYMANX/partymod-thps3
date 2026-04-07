pub enum Component<T> {
    Button(crate::button::Button<T>)
}

impl<T> From<crate::button::Button<T>> for Component<T> {
    fn from(value: crate::button::Button<T>) -> Self {
        Component::Button(value)
    }
}
pub enum Component<T> {
    Button(crate::button::Button<T>),
    Container(crate::container::Container<T>),
    Horizontal(crate::container::Horizontal<T>),
    Vertical(crate::container::Vertical<T>),
}

impl<T> From<crate::button::Button<T>> for Component<T> {
    fn from(value: crate::button::Button<T>) -> Self {
        Component::Button(value)
    }
}

impl<T> From<crate::container::Container<T>> for Component<T> {
    fn from(value: crate::container::Container<T>) -> Self {
        Component::Container(value)
    }
}

impl<T> From<crate::container::Horizontal<T>> for Component<T> {
    fn from(value: crate::container::Horizontal<T>) -> Self {
        Component::Horizontal(value)
    }
}

impl<T> From<crate::container::Vertical<T>> for Component<T> {
    fn from(value: crate::container::Vertical<T>) -> Self {
        Component::Vertical(value)
    }
}

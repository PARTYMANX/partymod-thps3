pub enum Component<T> {
    Button(crate::button::Button<T>),
    Checkbox(crate::checkbox::Checkbox<T>),
    Text(crate::text::Text<T>),
    Container(crate::container::Container<T>),
    Horizontal(crate::container::Horizontal<T>),
    Vertical(crate::container::Vertical<T>),
}

impl<T> From<crate::button::Button<T>> for Component<T> {
    fn from(value: crate::button::Button<T>) -> Self {
        Component::Button(value)
    }
}

impl<T> From<crate::checkbox::Checkbox<T>> for Component<T> {
    fn from(value: crate::checkbox::Checkbox<T>) -> Self {
        Component::Checkbox(value)
    }
}

impl<T> From<crate::text::Text<T>> for Component<T> {
    fn from(value: crate::text::Text<T>) -> Self {
        Component::Text(value)
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

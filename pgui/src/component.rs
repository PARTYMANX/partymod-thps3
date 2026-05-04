pub enum Component<T> {
    Button(crate::button::Button<T>),
    Checkbox(crate::checkbox::Checkbox<T>),
    Text(crate::text::Text<T>),
    Dropdown(crate::dropdown::Dropdown<T>),
    Textbox(crate::textbox::Textbox<T>),
    Container(crate::container::Container<T>),
    Groupbox(crate::groupbox::Groupbox<T>),
    Tabs(crate::tabs::Tabs<T>),
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

impl<T> From<crate::dropdown::Dropdown<T>> for Component<T> {
    fn from(value: crate::dropdown::Dropdown<T>) -> Self {
        Component::Dropdown(value)
    }
}

impl<T> From<crate::textbox::Textbox<T>> for Component<T> {
    fn from(value: crate::textbox::Textbox<T>) -> Self {
        Component::Textbox(value)
    }
}

impl<T> From<crate::container::Container<T>> for Component<T> {
    fn from(value: crate::container::Container<T>) -> Self {
        Component::Container(value)
    }
}

impl<T> From<crate::groupbox::Groupbox<T>> for Component<T> {
    fn from(value: crate::groupbox::Groupbox<T>) -> Self {
        Component::Groupbox(value)
    }
}

impl<T> From<crate::tabs::Tabs<T>> for Component<T> {
    fn from(value: crate::tabs::Tabs<T>) -> Self {
        Component::Tabs(value)
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

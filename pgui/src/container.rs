use crate::component::Component;

pub struct Container<T> {
    pub(crate) child: Box<Component<T>>,
}

pub fn container<T>(child: Component<T>) -> Container<T> {
    Container {
        child: Box::new(child),
    }
}

pub struct Horizontal<T> {
    pub(crate) children: Vec<Component<T>>,
}

pub fn horizontal<T>(children: Vec<Component<T>>) -> Horizontal<T> {
    Horizontal {
        children,
    }
}

pub struct Vertical<T> {
    pub(crate) children: Vec<Component<T>>,
}

pub fn vertical<T>(children: Vec<Component<T>>) -> Vertical<T> {
    Vertical {
        children,
    }
}

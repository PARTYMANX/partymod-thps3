pub struct Window<T> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) title: String,

    pub(crate) post_update: Option<fn(&mut T)>,
}

/*pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub title: String,
}*/

impl<T> Window<T> {
    pub fn new(title: String) -> Self {
        Self::default().title(title)
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn post_update(mut self, func: fn(&mut T)) -> Self {
        self.post_update = Some(func);
        self
    }
}

impl<T> Default for Window<T> {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            width: 640,
            height: 480,
            post_update: None,
        }
    }
}

pub fn window<T>(title: String) -> Window<T> {
    Window::new(title)
}

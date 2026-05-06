pub struct Window<T> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) title: String,

    pub(crate) state_hook: Option<fn(&T, &mut WindowState)>,
    pub(crate) post_update: Option<fn(&mut T)>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub should_quit: bool,
}

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

    pub fn state_hook(mut self, func: fn(&T, &mut WindowState)) -> Self {
        self.state_hook = Some(func);
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
            state_hook: None,
            post_update: None,
        }
    }
}

pub fn window<T>(title: String) -> Window<T> {
    Window::new(title)
}

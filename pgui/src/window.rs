pub struct Window<T> {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) title: String,

    pub(crate) state_hook: Option<Box<dyn Fn(&T, &mut WindowState)>>,
    pub(crate) post_update: Option<Box<dyn Fn(&mut T)>>,
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

    pub fn state_hook(mut self, func: impl Fn(&T, &mut WindowState) + 'static) -> Self {
        self.state_hook = Some(Box::new(func));
        self
    }

    pub fn post_update(mut self, func: impl Fn(&mut T) + 'static) -> Self {
        self.post_update = Some(Box::new(func));
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

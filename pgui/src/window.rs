pub struct WindowState {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) title: String,
}

impl WindowState {
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
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            width: 640,
            height: 480,
        }
    }
}

pub fn window(title: String) -> WindowState {
    WindowState::new(title)
}
use crate::layout::{HorizontalOffset, Position, Size, VerticalOffset};

pub struct Dropdown<T> {
    pub(crate) options: Vec<String>,
    pub(crate) enabled: bool,
    pub(crate) on_select: Option<Box<dyn Fn(&mut T, u32)>>,
    pub(crate) state_hook: Option<Box<dyn Fn(&T, &mut DropdownState)>>,
    pub(crate) position: Position,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DropdownState {
    pub options: Vec<String>,
    pub selected: u32,
    pub enabled: bool,
    pub position: Position,
}

impl<T> Dropdown<T> {
    pub fn on_select(mut self, func: impl Fn(&mut T, u32) + 'static) -> Self {
        self.on_select = Some(Box::new(func));

        if !self.enabled {
            self.enabled = true;
        }

        self
    }

    pub fn state_hook(mut self, func: impl Fn(&T, &mut DropdownState) + 'static) -> Self {
        self.state_hook = Some(Box::new(func));
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.position.w = width;
        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.position.h = height;
        self
    }

    pub fn position(mut self, x: HorizontalOffset, y: VerticalOffset) -> Self {
        self.position.x = x;
        self.position.y = y;
        self
    }

    pub fn h_position(mut self, h_position: HorizontalOffset) -> Self {
        self.position.x = h_position;
        self
    }

    pub fn v_position(mut self, v_position: VerticalOffset) -> Self {
        self.position.y = v_position;
        self
    }

    pub fn padding(mut self, padding: u32) -> Self {
        self.position.h_padding = padding;
        self.position.v_padding = padding;
        self
    }

    pub fn v_padding(mut self, v_padding: u32) -> Self {
        self.position.v_padding = v_padding;
        self
    }

    pub fn h_padding(mut self, h_padding: u32) -> Self {
        self.position.h_padding = h_padding;
        self
    }
}

pub fn dropdown<T>(options: Vec<String>) -> Dropdown<T> {
    Dropdown {
        options,
        enabled: false,
        on_select: None,
        state_hook: None,
        position: Position::default(),
    }
}

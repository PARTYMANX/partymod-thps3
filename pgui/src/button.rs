use crate::layout::{HorizontalOffset, Position, Size, VerticalOffset};

pub struct Button<T> {
    pub(crate) label: String,
    pub(crate) on_press: Option<fn(&mut T)>,
    pub(crate) position: Position,
}

impl<T> Button<T> {
    pub fn on_press(mut self, func: fn(&mut T)) -> Self {
        self.on_press = Some(func);
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

pub fn button<T>(label: String) -> Button<T> {
    Button {
        label,
        on_press: None,
        position: Position::default(),
    }
}

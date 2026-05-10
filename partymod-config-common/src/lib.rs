#[derive(Clone, Copy)]
#[repr(i32)]
pub enum Button {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Select = 4,
    Home = 5,
    Start = 6,
    LeftStick = 7,
    RightStick = 8,
    LeftShoulder = 9,
    RightShoulder = 10,
    DPadUp = 11,
    DPadDown = 12,
    DPadLeft = 13,
    DPadRight = 14,
    Misc1 = 15,
    RightPaddle1 = 16,
    LeftPaddle1 = 17,
    RightPaddle2 = 18,
    LeftPaddle2 = 19,
    Touchpad = 20,
    RightTrigger = 21,
    LeftTrigger = 22,
    Misc2 = 23,
    Misc3 = 24,
    Misc4 = 25,
    Misc5 = 26,
}

impl Button {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::A),
            1 => Some(Self::B),
            2 => Some(Self::X),
            3 => Some(Self::Y),
            4 => Some(Self::Select),
            5 => Some(Self::Home),
            6 => Some(Self::Start),
            7 => Some(Self::LeftStick),
            8 => Some(Self::RightStick),
            9 => Some(Self::LeftShoulder),
            10 => Some(Self::RightShoulder),
            11 => Some(Self::DPadUp),
            12 => Some(Self::DPadDown),
            13 => Some(Self::DPadLeft),
            14 => Some(Self::DPadRight),
            15 => Some(Self::Misc1),
            16 => Some(Self::RightPaddle1),
            17 => Some(Self::LeftPaddle1),
            18 => Some(Self::RightPaddle2),
            19 => Some(Self::LeftPaddle2),
            20 => Some(Self::Touchpad),
            21 => Some(Self::RightTrigger),
            22 => Some(Self::LeftTrigger),
            23 => Some(Self::Misc2),
            24 => Some(Self::Misc3),
            25 => Some(Self::Misc4),
            26 => Some(Self::Misc5),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
#[repr(i32)]
pub enum Stick {
    Left = 0,
    Right = 1,
}

impl Stick {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum BindType {
    Button { value: Option<Button> },
    Stick { value: Option<Stick> },
}

pub struct GamepadBind {
    pub display_name: &'static str,
    pub key: &'static str,
    pub default: BindType,
}

pub struct Keybind {
    pub display_name: &'static str,
    pub key: &'static str,
    pub default: Option<sdl3::keyboard::Scancode>,
}

use partymod_config_common::{BindType, Button, GamepadBind, Keybind, Stick};

pub const GAMEPAD_BINDS: [GamepadBind; 17] = [
    GamepadBind {
        display_name: "Ollie",
        key: "Ollie",
        default: BindType::Button {
            value: Some(Button::A),
        },
    },
    GamepadBind {
        display_name: "Grab",
        key: "Grab",
        default: BindType::Button {
            value: Some(Button::B),
        },
    },
    GamepadBind {
        display_name: "Flip",
        key: "Flip",
        default: BindType::Button {
            value: Some(Button::X),
        },
    },
    GamepadBind {
        display_name: "Grind",
        key: "Grind",
        default: BindType::Button {
            value: Some(Button::Y),
        },
    },
    GamepadBind {
        display_name: "Spin Left",
        key: "SpinLeft",
        default: BindType::Button {
            value: Some(Button::LeftShoulder),
        },
    },
    GamepadBind {
        display_name: "Spin Right",
        key: "SpinRight",
        default: BindType::Button {
            value: Some(Button::RightShoulder),
        },
    },
    GamepadBind {
        display_name: "Nollie",
        key: "Nollie",
        default: BindType::Button {
            value: Some(Button::LeftTrigger),
        },
    },
    GamepadBind {
        display_name: "Switch",
        key: "Switch",
        default: BindType::Button {
            value: Some(Button::RightTrigger),
        },
    },
    GamepadBind {
        display_name: "Pause",
        key: "Pause",
        default: BindType::Button {
            value: Some(Button::Start),
        },
    },
    GamepadBind {
        display_name: "Forward",
        key: "Forward",
        default: BindType::Button {
            value: Some(Button::DPadUp),
        },
    },
    GamepadBind {
        display_name: "Backward",
        key: "Backward",
        default: BindType::Button {
            value: Some(Button::DPadDown),
        },
    },
    GamepadBind {
        display_name: "Left",
        key: "Left",
        default: BindType::Button {
            value: Some(Button::DPadLeft),
        },
    },
    GamepadBind {
        display_name: "Right",
        key: "Right",
        default: BindType::Button {
            value: Some(Button::DPadRight),
        },
    },
    GamepadBind {
        display_name: "Move Stick",
        key: "MovementStick",
        default: BindType::Stick {
            value: Some(Stick::Left),
        },
    },
    GamepadBind {
        display_name: "Camera Stick",
        key: "CameraStick",
        default: BindType::Stick {
            value: Some(Stick::Right),
        },
    },
    GamepadBind {
        display_name: "View Toggle",
        key: "ViewToggle",
        default: BindType::Button {
            value: Some(Button::Select),
        },
    },
    GamepadBind {
        display_name: "Swivel Lock",
        key: "SwivelLock",
        default: BindType::Button {
            value: Some(Button::RightStick),
        },
    },
];

pub const KEYBINDS: [Keybind; 19] = [
    Keybind {
        display_name: "Ollie",
        key: "Ollie",
        default: Some(sdl3::keyboard::Scancode::Kp2),
    },
    Keybind {
        display_name: "Grab",
        key: "Grab",
        default: Some(sdl3::keyboard::Scancode::Kp6),
    },
    Keybind {
        display_name: "Flip",
        key: "Flip",
        default: Some(sdl3::keyboard::Scancode::Kp4),
    },
    Keybind {
        display_name: "Grind",
        key: "Grind",
        default: Some(sdl3::keyboard::Scancode::Kp8),
    },
    Keybind {
        display_name: "Spin Left",
        key: "SpinLeft",
        default: Some(sdl3::keyboard::Scancode::Kp1),
    },
    Keybind {
        display_name: "Spin Right",
        key: "SpinRight",
        default: Some(sdl3::keyboard::Scancode::Kp3),
    },
    Keybind {
        display_name: "Nollie",
        key: "Nollie",
        default: Some(sdl3::keyboard::Scancode::Kp7),
    },
    Keybind {
        display_name: "Switch",
        key: "Switch",
        default: Some(sdl3::keyboard::Scancode::Kp9),
    },
    Keybind {
        display_name: "Pause",
        key: "Pause",
        default: None,
    },
    Keybind {
        display_name: "Forward",
        key: "Forward",
        default: Some(sdl3::keyboard::Scancode::W),
    },
    Keybind {
        display_name: "Backward",
        key: "Backward",
        default: Some(sdl3::keyboard::Scancode::S),
    },
    Keybind {
        display_name: "Left",
        key: "Left",
        default: Some(sdl3::keyboard::Scancode::A),
    },
    Keybind {
        display_name: "Right",
        key: "Right",
        default: Some(sdl3::keyboard::Scancode::D),
    },
    Keybind {
        display_name: "Camera Up",
        key: "CameraUp",
        default: Some(sdl3::keyboard::Scancode::I),
    },
    Keybind {
        display_name: "Camera Down",
        key: "CameraDown",
        default: Some(sdl3::keyboard::Scancode::K),
    },
    Keybind {
        display_name: "Camera Left",
        key: "CameraLeft",
        default: Some(sdl3::keyboard::Scancode::J),
    },
    Keybind {
        display_name: "Camera Right",
        key: "CameraRight",
        default: Some(sdl3::keyboard::Scancode::L),
    },
    Keybind {
        display_name: "View Toggle",
        key: "ViewToggle",
        default: Some(sdl3::keyboard::Scancode::Grave),
    },
    Keybind {
        display_name: "Swivel Lock",
        key: "SwivelLock",
        default: None,
    },
];

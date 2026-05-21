// 004618f3 - park editor method that runs right after controller input

use partymod_common::{gamepad::GamepadManager, patch::patch_call};
use sdl3::keyboard::Scancode;

use crate::sdl::SDL_CONTEXT;

pub struct ParkEditorControlsState {
    button_mask: u16,
    buttons_just: u16,
    move_stick: (f32, f32),
    camera_stick: (f32, f32),
    debouncing: bool,
}

impl ParkEditorControlsState {
    const JUST_MASK: u16 = 0x9FFF;

    pub fn new() -> Self {
        Self {
            button_mask: 0x0000,
            buttons_just: 0x0000,
            debouncing: false,
            move_stick: (0.0, 0.0),
            camera_stick: (0.0, 0.0),
        }
    }

    // for some reason, park editor buttons exist somewhere else
    // poll the mapped controller binds and the hardcoded keyboard binds
    // (so that the loading tutorial image matches inputs)
    fn update(&mut self, gamepad_manager: &GamepadManager) {
        let sdl_ctx = unsafe {
            match &*SDL_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized SDL context!"),
            }
        };

        let keyboard_state = sdl_ctx.event_pump.keyboard_state();

        let mut new_button_mask = 0u16;

        if gamepad_manager.poll_button_binding(0, "Ollie").0
            || keyboard_state.is_scancode_pressed(Scancode::Return)
            || keyboard_state.is_scancode_pressed(Scancode::KpEnter)
        {
            new_button_mask |= 0x0001 << 5;
        }

        if gamepad_manager.poll_button_binding(0, "Left").0
            || keyboard_state.is_scancode_pressed(Scancode::Left)
        {
            new_button_mask |= 0x0001 << 3;
        }
        if gamepad_manager.poll_button_binding(0, "Right").0
            || keyboard_state.is_scancode_pressed(Scancode::Right)
        {
            new_button_mask |= 0x0001 << 4;
        }
        if gamepad_manager.poll_button_binding(0, "Backward").0
            || keyboard_state.is_scancode_pressed(Scancode::Down)
        {
            new_button_mask |= 0x0001 << 9;
        }
        if gamepad_manager.poll_button_binding(0, "Forward").0
            || keyboard_state.is_scancode_pressed(Scancode::Up)
        {
            new_button_mask |= 0x0001 << 10;
        }

        if gamepad_manager.poll_button_binding(0, "Switch").0
            || keyboard_state.is_scancode_pressed(Scancode::PageDown)
        {
            new_button_mask |= 0x0001 << 7;
        }
        if gamepad_manager.poll_button_binding(0, "SpinRight").0
            || keyboard_state.is_scancode_pressed(Scancode::PageUp)
        {
            new_button_mask |= 0x0001 << 8;
        }

        if gamepad_manager.poll_button_binding(0, "Grab").0
            || (keyboard_state.is_scancode_pressed(Scancode::Space)
                && (keyboard_state.is_scancode_pressed(Scancode::LShift)
                    || keyboard_state.is_scancode_pressed(Scancode::RShift)))
        {
            new_button_mask |= 0x0001 << 1;
        }
        if gamepad_manager.poll_button_binding(0, "Flip").0
            || (keyboard_state.is_scancode_pressed(Scancode::Space)
                && !(keyboard_state.is_scancode_pressed(Scancode::LShift)
                    || keyboard_state.is_scancode_pressed(Scancode::RShift)))
        {
            new_button_mask |= 0x0001 << 2;
        }
        if gamepad_manager.poll_button_binding(0, "Grind").0
            || keyboard_state.is_scancode_pressed(Scancode::Backspace)
        {
            new_button_mask |= 0x0001 << 6;
        }

        if gamepad_manager.poll_button_binding(0, "Nollie").0
            || keyboard_state.is_scancode_pressed(Scancode::Minus)
        {
            new_button_mask |= 0x0001 << 13;
        }
        if gamepad_manager.poll_button_binding(0, "SpinLeft").0
            || keyboard_state.is_scancode_pressed(Scancode::Equals)
        {
            new_button_mask |= 0x0001 << 14;
        }

        let camera_stick = gamepad_manager.poll_stick_binding(0, "CameraStick");

        let camera_x = {
            let v = camera_stick.0;
            let controller_value = if v > 0 {
                v as f32 / i16::MAX as f32
            } else {
                -(v as f32 / i16::MIN as f32)
            } * 128.0;

            let keyboard_pos = keyboard_state.is_scancode_pressed(Scancode::Kp3)
                || keyboard_state.is_scancode_pressed(Scancode::Kp6)
                || keyboard_state.is_scancode_pressed(Scancode::Kp9);
            let keyboard_neg = keyboard_state.is_scancode_pressed(Scancode::Kp1)
                || keyboard_state.is_scancode_pressed(Scancode::Kp4)
                || keyboard_state.is_scancode_pressed(Scancode::Kp7);
            let keyboard_value: f32 = if keyboard_pos && keyboard_neg {
                0.0
            } else if keyboard_pos {
                128.0
            } else if keyboard_neg {
                -128.0
            } else {
                0.0
            };

            if keyboard_value.abs() > controller_value.abs() {
                keyboard_value
            } else {
                controller_value
            }
        };

        let camera_y = {
            let v = camera_stick.1;
            let controller_value = if v > 0 {
                v as f32 / i16::MAX as f32
            } else {
                -(v as f32 / i16::MIN as f32)
            } * 128.0;

            let keyboard_pos = keyboard_state.is_scancode_pressed(Scancode::Kp1)
                || keyboard_state.is_scancode_pressed(Scancode::Kp2)
                || keyboard_state.is_scancode_pressed(Scancode::Kp3);
            let keyboard_neg = keyboard_state.is_scancode_pressed(Scancode::Kp7)
                || keyboard_state.is_scancode_pressed(Scancode::Kp8)
                || keyboard_state.is_scancode_pressed(Scancode::Kp9);
            let keyboard_value: f32 = if keyboard_pos && keyboard_neg {
                0.0
            } else if keyboard_pos {
                128.0
            } else if keyboard_neg {
                -128.0
            } else {
                0.0
            };

            if keyboard_value.abs() > controller_value.abs() {
                keyboard_value
            } else {
                controller_value
            }
        };

        self.camera_stick = (camera_x, camera_y);

        let movement_stick = gamepad_manager.poll_stick_binding(0, "MovementStick");

        let move_x = {
            let v = movement_stick.0;
            let controller_value = if v > 0 {
                v as f32 / i16::MAX as f32
            } else {
                -(v as f32 / i16::MIN as f32)
            } * 128.0;

            let keyboard_pos = keyboard_state.is_scancode_pressed(Scancode::D);
            let keyboard_neg = keyboard_state.is_scancode_pressed(Scancode::A);
            let keyboard_value: f32 = if keyboard_pos && keyboard_neg {
                0.0
            } else if keyboard_pos {
                128.0
            } else if keyboard_neg {
                -128.0
            } else {
                0.0
            };

            if keyboard_value.abs() > controller_value.abs() {
                keyboard_value
            } else {
                controller_value
            }
        };

        let move_y = {
            let v = movement_stick.1;
            let controller_value = if v > 0 {
                v as f32 / i16::MAX as f32
            } else {
                -(v as f32 / i16::MIN as f32)
            } * 128.0;

            let keyboard_pos = keyboard_state.is_scancode_pressed(Scancode::S);
            let keyboard_neg = keyboard_state.is_scancode_pressed(Scancode::W);
            let keyboard_value: f32 = if keyboard_pos && keyboard_neg {
                0.0
            } else if keyboard_pos {
                128.0
            } else if keyboard_neg {
                -128.0
            } else {
                0.0
            };

            if keyboard_value.abs() > controller_value.abs() {
                keyboard_value
            } else {
                controller_value
            }
        };

        self.move_stick = (move_x, move_y);

        if !self.debouncing {
            self.buttons_just = !self.button_mask & new_button_mask;
        }
        self.button_mask = new_button_mask;

        if self.debouncing && self.button_mask == 0 {
            self.debouncing = false;
        }
    }

    // prevents an extra button when transitioning into the park editor
    fn debounce(&mut self) {
        self.debouncing = true;
        self.buttons_just = 0x0000;
    }

    fn get_button_state(&self) -> u16 {
        (self.buttons_just & Self::JUST_MASK) | (self.button_mask & !Self::JUST_MASK)
    }

    fn get_move_stick(&self) -> (f32, f32) {
        self.move_stick
    }

    fn get_camera_stick(&self) -> (f32, f32) {
        self.camera_stick
    }
}

pub unsafe extern "C" fn park_editor_debounce_wrapper() {
    let inp_ctx = unsafe {
        match &mut *super::INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    // initialize park editor control state
    inp_ctx.park_editor_state.update(&inp_ctx.gamepad_manager);
    inp_ctx.park_editor_state.debounce();
    // TODO: change this to control state. then diff the states in the update to get the makes
    // make a new func to generate our button mask. stick positions can still go into update

    unsafe {
        let orig_func: unsafe extern "C" fn() = std::mem::transmute(0x004c6870 as *const ());

        orig_func();
    }
}

pub unsafe extern "thiscall" fn park_editor_controls_wrapper(
    editor: *mut (),
    unk1: u32,
    unk2: u32,
) {
    let inp_ctx = unsafe {
        match &mut *super::INPUT_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized input context!"),
        }
    };

    inp_ctx.park_editor_state.update(&inp_ctx.gamepad_manager);

    let button_mask = inp_ctx.park_editor_state.get_button_state();

    let move_stick = inp_ctx.park_editor_state.get_move_stick();
    let camera_stick = inp_ctx.park_editor_state.get_camera_stick();

    unsafe {
        let editor_buttons = editor.byte_add(0x44) as *mut u32;

        *editor_buttons |= button_mask as u32;

        let move_y = editor.byte_add(0x40) as *mut f32;
        let move_x = editor.byte_add(0x3c) as *mut f32;
        let camera_y = editor.byte_add(0x38) as *mut f32;
        let camera_x = editor.byte_add(0x34) as *mut f32;

        *move_x = move_stick.0;
        *move_y = move_stick.1;
        *camera_x = camera_stick.0;
        *camera_y = camera_stick.1;
    }

    unsafe {
        let orig_func: unsafe extern "thiscall" fn(*mut (), u32, u32) =
            std::mem::transmute(0x004629f0 as *const ());

        orig_func(editor, unk1, unk2);
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_call(
            0x00463f48 as *mut (),
            park_editor_debounce_wrapper as *const (),
        );
        patch_call(
            0x004618f3 as *mut (),
            park_editor_controls_wrapper as *const (),
        );
    }
}

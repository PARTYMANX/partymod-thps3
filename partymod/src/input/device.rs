use partymod_common::{logger::LogLevel, patch};
use sdl3::keyboard::KeyboardState;

use crate::{
    input::{
        InputContext, is_menu_open,
        keyboard::{is_keyboard_on_screen, is_network_menu_on_screen},
    },
    logger,
    sdl::SDL_CONTEXT,
};

#[repr(C)]
pub struct Device {
    vtable: u32,
    node: u32,
    ty: u32,
    port: u32,
    slot: u32,
    is_valid: u32,
    unk1: u32,
    control_data: [u8; 32],
    vibration_data_align: [u8; 32],
    vibration_data_direct: [u8; 32],
    vibration_data_max: [u8; 32],
    vibration_data_old_direct: [u8; 32],
    paused: u32,
    actuators_disabled: u32,
    capabilities: u32,
    unk2: u32,
    num_actuators: u32,
    unk3: u32,
    state: u32,
    next_state: u32,
    index: u32,
    is_plugged_in: u32,
    device_interface: u32,
    unk4: u32,
    unk5: u32,
}

impl Device {
    extern "thiscall" fn process(&mut self) {
        self.capabilities = 0x0003;
        self.num_actuators = 2;
        self.vibration_data_max[0] = 255;
        self.vibration_data_max[1] = 255;
        self.state = 2;

        self.is_valid = 1;
        self.is_plugged_in = 1;

        self.control_data[0] = 0x00;
        self.control_data[1] = 0x50;

        // button bitmap
        self.control_data[2] = 0x00;
        self.control_data[3] = 0x00;

        // pressure info
        // buttons
        self.control_data[12] = 0;
        self.control_data[13] = 0;
        self.control_data[14] = 0;
        self.control_data[15] = 0;

        // shoulders
        self.control_data[16] = 0;
        self.control_data[17] = 0;
        self.control_data[18] = 0;
        self.control_data[19] = 0;

        // d-pad
        self.control_data[8] = 0;
        self.control_data[9] = 0;
        self.control_data[10] = 0;
        self.control_data[11] = 0;

        // sticks
        self.control_data[4] = 127;
        self.control_data[5] = 127;
        self.control_data[6] = 127;
        self.control_data[7] = 127;

        if !is_network_menu_on_screen() {
            self.poll_controller();

            if self.slot == 0 {
                self.poll_keyboard();
            }
        } else {
            self.poll_controller_network_menu();
        }

        // TODO: convert movement stick to d-pad inputs when in menu
        if is_menu_open() {
            if self.control_data[6] < 64 {
                self.control_data[2] |= 0x01 << 7;
                self.control_data[9] = 0xFF;
            } else if self.control_data[6] > (255 - 64) {
                self.control_data[2] |= 0x01 << 5;
                self.control_data[8] = 0xFF;
            }

            if self.control_data[7] < 64 {
                self.control_data[2] |= 0x01 << 4;
                self.control_data[10] = 0xFF;
            } else if self.control_data[7] > (255 - 64) {
                self.control_data[2] |= 0x01 << 6;
                self.control_data[11] = 0xFF;
            }
        }

        self.control_data[2] = !self.control_data[2];
        self.control_data[3] = !self.control_data[3];
    }

    extern "thiscall" fn acquire(&mut self) {
        // do nothing
    }

    extern "thiscall" fn unacquire(&mut self) {
        // do nothing
    }

    extern "thiscall" fn init(&mut self, _unk: u32) {
        logger::log(
            LogLevel::Debug,
            &format!(
                "Initializing controller, index: {} slot: {} port: {}",
                self.index, self.slot, self.port
            ),
        );
    }

    extern "thiscall" fn release(&mut self) {
        // do nothing
    }

    extern "thiscall" fn activate_actuator(&mut self, idx: u32, percent: u32) {
        if self.actuators_disabled != 0 {
            return;
        }

        let inp_ctx = unsafe {
            match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        let player = self.slot as usize;

        if self.state == 2 && (self.capabilities & 0x02 != 0) {
            if idx < self.num_actuators {
                let strength =
                    (percent as f32 * 0.01) * self.vibration_data_max[idx as usize] as f32;
                self.vibration_data_direct[idx as usize] = strength as u8;

                let high = (self.vibration_data_direct[0] as u16) << 8;
                let low = (self.vibration_data_direct[1] as u16) << 8;

                inp_ctx.gamepad_manager.set_rumble(player, high, low);
            }
        }
    }

    extern "thiscall" fn enable_actuators(&mut self) {
        self.actuators_disabled = 0;
    }

    pub extern "thiscall" fn disable_actuators(&mut self) {
        if self.actuators_disabled != 0 {
            return;
        }

        self.actuators_disabled = 1;

        let inp_ctx = unsafe {
            match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        let player = self.slot as usize;

        if self.state == 2 && (self.capabilities & 0x02 != 0) {
            for i in 0..self.num_actuators {
                self.vibration_data_direct[i as usize] = 0;
            }

            let high = (self.vibration_data_direct[0] as u16) << 8;
            let low = (self.vibration_data_direct[1] as u16) << 8;

            inp_ctx.gamepad_manager.set_rumble(player, high, low);
        }
    }

    extern "thiscall" fn reset_actuators(&mut self) {
        if self.actuators_disabled != 0 {
            self.enable_actuators();
            self.disable_actuators();
        } else {
            self.disable_actuators();
            self.enable_actuators();
        }
    }

    extern "thiscall" fn pause(&mut self) {
        if self.paused == 0 {
            let inp_ctx = unsafe {
                match &mut *super::INPUT_CONTEXT.get() {
                    Some(v) => v,
                    None => panic!("Tried to get uninitialized input context!"),
                }
            };

            let player = self.slot as usize;

            if self.state == 2 && (self.capabilities & 0x02 != 0) {
                for i in 0..self.num_actuators {
                    self.vibration_data_old_direct[i as usize] =
                        self.vibration_data_direct[i as usize];

                    self.vibration_data_direct[i as usize] = 0;
                }

                let high = (self.vibration_data_direct[0] as u16) << 8;
                let low = (self.vibration_data_direct[1] as u16) << 8;

                inp_ctx.gamepad_manager.set_rumble(player, high, low);
            }
        }

        self.paused += 1;
    }

    extern "thiscall" fn unpause(&mut self) {
        if self.paused > 0 {
            self.paused -= 1;

            if self.paused == 0 {
                let inp_ctx = unsafe {
                    match &mut *super::INPUT_CONTEXT.get() {
                        Some(v) => v,
                        None => panic!("Tried to get uninitialized input context!"),
                    }
                };

                let player = self.slot as usize;

                if self.state == 2 && (self.capabilities & 0x02 != 0) {
                    for i in 0..self.num_actuators {
                        self.vibration_data_direct[i as usize] =
                            self.vibration_data_old_direct[i as usize];
                    }

                    let high = (self.vibration_data_direct[0] as u16) << 8;
                    let low = (self.vibration_data_direct[1] as u16) << 8;

                    inp_ctx.gamepad_manager.set_rumble(player, high, low);
                }
            }
        }
    }

    fn poll_keyboard(&mut self) {
        let inp_ctx = unsafe {
            match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        let sdl_ctx = unsafe {
            match &*SDL_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized SDL context!"),
            }
        };

        let keyboard_state = sdl_ctx.event_pump.keyboard_state();

        let in_menu = is_menu_open();
        inp_ctx
            .keybind_manager
            .set_in_menu(in_menu, &keyboard_state);

        if is_keyboard_on_screen() {
            return;
        }

        if in_menu {
            // polls with ignored results to update lock state
            inp_ctx
                .keybind_manager
                .poll_menu_binding("Accept", &keyboard_state);
            inp_ctx
                .keybind_manager
                .poll_menu_binding("Accept2", &keyboard_state);
            inp_ctx
                .keybind_manager
                .poll_menu_binding("Back", &keyboard_state);

            // okay, actual menu controls now
            self.poll_menu_key(inp_ctx, &keyboard_state, "Up", InternalButton::DPadUp);
            self.poll_menu_key(inp_ctx, &keyboard_state, "Down", InternalButton::DPadDown);
            self.poll_menu_key(inp_ctx, &keyboard_state, "Left", InternalButton::DPadLeft);
            self.poll_menu_key(inp_ctx, &keyboard_state, "Right", InternalButton::DPadRight);
        }

        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Pause", InternalButton::Start);
        self.poll_keyboard_key(
            inp_ctx,
            &keyboard_state,
            "ViewToggle",
            InternalButton::Select,
        );
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "SwivelLock", InternalButton::R3);

        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Ollie", InternalButton::Cross);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Flip", InternalButton::Square);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Grab", InternalButton::Circle);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Grind", InternalButton::Triangle);

        self.poll_keyboard_key(inp_ctx, &keyboard_state, "SpinLeft", InternalButton::L1);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "SpinRight", InternalButton::R1);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Nollie", InternalButton::L2);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Switch", InternalButton::R2);

        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Forward", InternalButton::DPadUp);
        self.poll_keyboard_key(
            inp_ctx,
            &keyboard_state,
            "Backward",
            InternalButton::DPadDown,
        );
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Left", InternalButton::DPadLeft);
        self.poll_keyboard_key(inp_ctx, &keyboard_state, "Right", InternalButton::DPadRight);

        let camera_x =
            self.keyboard_keys_to_axis(inp_ctx, &keyboard_state, "CameraRight", "CameraLeft");
        let camera_y =
            self.keyboard_keys_to_axis(inp_ctx, &keyboard_state, "CameraDown", "CameraUp");
        (self.control_data[4], self.control_data[5]) = Self::stick_max(
            self.control_data[4],
            self.control_data[5],
            camera_x,
            camera_y,
        );
    }

    fn poll_keyboard_key(
        &mut self,
        inp_ctx: &mut InputContext,
        keyboard_state: &KeyboardState,
        name: &str,
        internal_button: InternalButton,
    ) {
        let pressed = inp_ctx
            .keybind_manager
            .poll_key_binding(name, keyboard_state);

        let pressure = if pressed { 0xFF } else { 0x00 };

        if pressed {
            self.update_internal_button(internal_button, pressed, pressure);
        }
    }

    fn poll_menu_key(
        &mut self,
        inp_ctx: &mut InputContext,
        keyboard_state: &KeyboardState,
        name: &str,
        internal_button: InternalButton,
    ) {
        let pressed = inp_ctx
            .keybind_manager
            .poll_menu_binding(name, keyboard_state);

        let pressure = if pressed { 0xFF } else { 0x00 };

        if pressed {
            self.update_internal_button(internal_button, pressed, pressure);
        }
    }

    fn keyboard_keys_to_axis(
        &mut self,
        inp_ctx: &mut InputContext,
        keyboard_state: &KeyboardState,
        name_positive: &str,
        name_negative: &str,
    ) -> u8 {
        let pos = inp_ctx
            .keybind_manager
            .poll_key_binding(name_positive, keyboard_state);
        let neg = inp_ctx
            .keybind_manager
            .poll_key_binding(name_negative, keyboard_state);

        if pos && neg {
            // NOTE: SOCD handling behavior is neutral
            127
        } else if pos {
            255
        } else if neg {
            0
        } else {
            127
        }
    }

    fn poll_controller(&mut self) {
        let inp_ctx = unsafe {
            match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        let player = self.slot as usize;

        if inp_ctx.network_menu_exit_debounce {
            if !inp_ctx
                .gamepad_manager
                .poll_button_binding(player, "Grind")
                .0
                && !inp_ctx
                    .gamepad_manager
                    .poll_button_binding(player, "Pause")
                    .0
            {
                inp_ctx.network_menu_exit_debounce = false;
            }
        }

        self.poll_controller_button(inp_ctx, player, "Pause", InternalButton::Start);
        self.poll_controller_button(inp_ctx, player, "ViewToggle", InternalButton::Select);
        self.poll_controller_button(inp_ctx, player, "SwivelLock", InternalButton::R3);

        self.poll_controller_button(inp_ctx, player, "Ollie", InternalButton::Cross);
        self.poll_controller_button(inp_ctx, player, "Flip", InternalButton::Square);
        self.poll_controller_button(inp_ctx, player, "Grab", InternalButton::Circle);
        self.poll_controller_button(inp_ctx, player, "Grind", InternalButton::Triangle);

        self.poll_controller_button(inp_ctx, player, "SpinLeft", InternalButton::L1);
        self.poll_controller_button(inp_ctx, player, "SpinRight", InternalButton::R1);
        self.poll_controller_button(inp_ctx, player, "Nollie", InternalButton::L2);
        self.poll_controller_button(inp_ctx, player, "Switch", InternalButton::R2);

        self.poll_controller_button(inp_ctx, player, "Forward", InternalButton::DPadUp);
        self.poll_controller_button(inp_ctx, player, "Backward", InternalButton::DPadDown);
        self.poll_controller_button(inp_ctx, player, "Left", InternalButton::DPadLeft);
        self.poll_controller_button(inp_ctx, player, "Right", InternalButton::DPadRight);

        let camera_stick = inp_ctx
            .gamepad_manager
            .poll_stick_binding(player, "CameraStick");
        let camera_x = ((camera_stick.0 >> 8) + 128) as u8;
        let camera_y = ((camera_stick.1 >> 8) + 128) as u8;

        (self.control_data[4], self.control_data[5]) = Self::stick_max(
            self.control_data[4],
            self.control_data[5],
            camera_x,
            camera_y,
        );

        let movement_stick = inp_ctx
            .gamepad_manager
            .poll_stick_binding(player, "MovementStick");
        let move_x = ((movement_stick.0 >> 8) + 128) as u8;
        let move_y = ((movement_stick.1 >> 8) + 128) as u8;

        (self.control_data[6], self.control_data[7]) =
            Self::stick_max(self.control_data[6], self.control_data[7], move_x, move_y);

        if inp_ctx.network_menu_exit_debounce {
            self.control_data[3] &= !(0x01 << 4);
            self.control_data[2] &= !(0x01 << 3);
        }
    }

    fn poll_controller_network_menu(&mut self) {
        let inp_ctx = unsafe {
            match &mut *super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        if inp_ctx.network_menu_exit_debounce {
            if !inp_ctx.gamepad_manager.poll_button_binding(0, "Grind").0
                && !inp_ctx.gamepad_manager.poll_button_binding(0, "Pause").0
            {
                inp_ctx.network_menu_exit_debounce = false;
            }

            return;
        }

        if inp_ctx.gamepad_manager.poll_button_binding(0, "Grind").0
            || inp_ctx.gamepad_manager.poll_button_binding(0, "Pause").0
        {
            inp_ctx.network_menu_exit_debounce = true;
            inp_ctx.keyboard.push_esc_down();
        }
    }

    fn poll_controller_button(
        &mut self,
        inp_ctx: &InputContext,
        player: usize,
        name: &str,
        internal_button: InternalButton,
    ) {
        let (pressed, pressure) = inp_ctx.gamepad_manager.poll_button_binding(player, name);
        if pressed {
            self.update_internal_button(internal_button, pressed, (pressure << 7) as u8);
        }
    }

    fn stick_max(a_x: u8, a_y: u8, b_x: u8, b_y: u8) -> (u8, u8) {
        // because range is between 0 and 255, take the absolute values
        let abs_a_x = Self::axis_abs(a_x);
        let abs_a_y = Self::axis_abs(a_y);
        let abs_b_x = Self::axis_abs(b_x);
        let abs_b_y = Self::axis_abs(b_y);

        let a_sq = (abs_a_x as i32 * abs_a_x as i32) + (abs_a_y as i32 * abs_a_y as i32);
        let b_sq = (abs_b_x as i32 * abs_b_x as i32) + (abs_b_y as i32 * abs_b_y as i32);

        if a_sq >= b_sq { (a_x, a_y) } else { (b_x, b_y) }
    }

    fn axis_abs(v: u8) -> u8 {
        if v > 0x7F { v & 0x7F } else { !v & 0x7F }
    }

    fn update_internal_button(&mut self, button: InternalButton, pressed: bool, pressure: u8) {
        match button {
            InternalButton::Cross => {
                self.control_data[3] |= (pressed as u8) << 6;
                let control_pressure = &mut self.control_data[14];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::Circle => {
                self.control_data[3] |= (pressed as u8) << 5;
                let control_pressure = &mut self.control_data[13];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::Square => {
                self.control_data[3] |= (pressed as u8) << 7;
                let control_pressure = &mut self.control_data[15];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::Triangle => {
                self.control_data[3] |= (pressed as u8) << 4;
                let control_pressure = &mut self.control_data[12];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::Start => {
                self.control_data[2] |= (pressed as u8) << 3;
            }
            InternalButton::Select => {
                self.control_data[2] |= (pressed as u8) << 0;
            }
            InternalButton::DPadUp => {
                self.control_data[2] |= (pressed as u8) << 4;
                let control_pressure = &mut self.control_data[10];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::DPadDown => {
                self.control_data[2] |= (pressed as u8) << 6;
                let control_pressure = &mut self.control_data[11];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::DPadLeft => {
                self.control_data[2] |= (pressed as u8) << 7;
                let control_pressure = &mut self.control_data[9];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::DPadRight => {
                self.control_data[2] |= (pressed as u8) << 5;
                let control_pressure = &mut self.control_data[8];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::L1 => {
                self.control_data[3] |= (pressed as u8) << 2;
                let control_pressure = &mut self.control_data[16];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::R1 => {
                self.control_data[3] |= (pressed as u8) << 3;
                let control_pressure = &mut self.control_data[17];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::L2 => {
                self.control_data[3] |= (pressed as u8) << 0;
                let control_pressure = &mut self.control_data[18];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::R2 => {
                self.control_data[3] |= (pressed as u8) << 1;
                let control_pressure = &mut self.control_data[19];
                *control_pressure = (*control_pressure).max(pressure);
            }
            InternalButton::L3 => {
                self.control_data[2] |= (pressed as u8) << 1;
            }
            InternalButton::R3 => {
                self.control_data[2] |= (pressed as u8) << 2;
            }
        }
    }
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040c570 as *mut (), Device::process as *const ());
        patch::patch_jmp(0x0040d010 as *mut (), Device::acquire as *const ());
        patch::patch_jmp(0x0040d060 as *mut (), Device::unacquire as *const ());
        patch::patch_jmp(0x0040d0a0 as *mut (), Device::init as *const ());
        patch::patch_jmp(0x0040d390 as *mut (), Device::release as *const ());

        patch::patch_jmp(0x0040d3c0 as *mut (), Device::reset_actuators as *const ());

        // ActivateActuators was optimized into a different call
        // restore each individual call
        patch::patch_call(
            0x004a90fb as *mut (),
            Device::activate_actuator as *const (),
        );
        patch::patch_call(
            0x004ac37f as *mut (),
            Device::activate_actuator as *const (),
        );
        patch::patch_call(
            0x004ac390 as *mut (),
            Device::activate_actuator as *const (),
        );
        patch::patch_call(
            0x004b18f2 as *mut (),
            Device::activate_actuator as *const (),
        );
        patch::patch_call(
            0x004b191d as *mut (),
            Device::activate_actuator as *const (),
        );
        patch::patch_call(
            0x004a9141 as *mut (),
            Device::activate_actuator as *const (),
        );

        // same deal with all of these but they were only ever called once
        patch::patch_call(0x004c0f03 as *mut (), Device::enable_actuators as *const ());

        patch::patch_call(0x0040de99 as *mut (), Device::unpause as *const ());

        patch::patch_call(0x0040de19 as *mut (), Device::pause as *const ());
    }
}

#[allow(unused)]
enum InternalButton {
    Cross,
    Circle,
    Square,
    Triangle,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
}

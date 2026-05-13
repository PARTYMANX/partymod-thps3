use partymod_common::{logger::LogLevel, patch::{patch_call, patch_jmp}};

use crate::logger;

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
    unk3: u32,
    num_actuators: u32,
    unk4: u32,
    state: u32,
    next_state: u32,
    index: u32,
    is_plugged_in: u32,
    device_interface: u32,
    unk5: u32,
    unk6: u32,
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

        self.poll_controller();

        self.control_data[2] = !self.control_data[2];
        self.control_data[3] = !self.control_data[3];
    }

    extern "thiscall" fn acquire(&mut self) {
        logger::log(LogLevel::Debug, "Device::acquire!");
    }

    extern "thiscall" fn unacquire(&mut self) {
        logger::log(LogLevel::Debug, "Device::unacquire!");
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
        logger::log(LogLevel::Debug, "Device::release!");
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

        if self.state == 2 && (self.capabilities & 0x02 != 0) {
            if idx < self.num_actuators {
                let strength = (percent as f32 * 0.01) * self.vibration_data_max[idx as usize] as f32;
                self.vibration_data_direct[idx as usize] = strength as u8;

                let high = (self.vibration_data_direct[0] as u16) << 8;
                let low = (self.vibration_data_direct[1] as u16) << 8;

                inp_ctx.gamepad_manager.set_rumble(high, low);
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

        if self.state == 2 && (self.capabilities & 0x02 != 0) {
            for i in 0..self.num_actuators {
                self.vibration_data_direct[i as usize] = 0;
            }

            let high = (self.vibration_data_direct[0] as u16) << 8;
            let low = (self.vibration_data_direct[1] as u16) << 8;

            inp_ctx.gamepad_manager.set_rumble(high, low);
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

            if self.state == 2 && (self.capabilities & 0x02 != 0) {
                for i in 0..self.num_actuators {
                    self.vibration_data_old_direct[i as usize] = self.vibration_data_direct[i as usize];

                    self.vibration_data_direct[i as usize] = 0;
                }

                let high = (self.vibration_data_direct[0] as u16) << 8;
                let low = (self.vibration_data_direct[1] as u16) << 8;

                inp_ctx.gamepad_manager.set_rumble(high, low);
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

                if self.state == 2 && (self.capabilities & 0x02 != 0) {
                    for i in 0..self.num_actuators {
                        self.vibration_data_direct[i as usize] = self.vibration_data_old_direct[i as usize];
                    }

                    let high = (self.vibration_data_direct[0] as u16) << 8;
                    let low = (self.vibration_data_direct[1] as u16) << 8;

                    inp_ctx.gamepad_manager.set_rumble(high, low);
                }
            }
        }
    }

    fn poll_controller(&mut self) {
        let inp_ctx = unsafe {
            match &*super::INPUT_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to get uninitialized input context!"),
            }
        };

        if inp_ctx.gamepad_manager.poll_button_binding("Pause").0 {
            self.control_data[2] |= 0x01 << 3;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("ViewToggle").0 {
            self.control_data[2] |= 0x01 << 0;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("SwivelLock").0 {
            self.control_data[2] |= 0x01 << 2;
        }

        if inp_ctx.gamepad_manager.poll_button_binding("Grind").0 {
            self.control_data[3] |= 0x01 << 4;
            self.control_data[12] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Grab").0 {
            self.control_data[3] |= 0x01 << 5;
            self.control_data[13] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Ollie").0 {
            self.control_data[3] |= 0x01 << 6;
            self.control_data[14] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Flip").0 {
            self.control_data[3] |= 0x01 << 7;
            self.control_data[15] = 0xFF;
        }

        if inp_ctx.gamepad_manager.poll_button_binding("SpinLeft").0 {
            self.control_data[3] |= 0x01 << 2;
            self.control_data[16] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("SpinRight").0 {
            self.control_data[3] |= 0x01 << 3;
            self.control_data[17] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Nollie").0 {
            self.control_data[3] |= 0x01 << 0;
            self.control_data[18] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Switch").0 {
            self.control_data[3] |= 0x01 << 1;
            self.control_data[19] = 0xFF;
        }

        if inp_ctx.gamepad_manager.poll_button_binding("Forward").0 {
            self.control_data[2] |= 0x01 << 4;
            self.control_data[10] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Right").0 {
            self.control_data[2] |= 0x01 << 5;
            self.control_data[8] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Backward").0 {
            self.control_data[2] |= 0x01 << 6;
            self.control_data[11] = 0xFF;
        }
        if inp_ctx.gamepad_manager.poll_button_binding("Left").0 {
            self.control_data[2] |= 0x01 << 7;
            self.control_data[9] = 0xFF;
        }

        let camera_stick = inp_ctx.gamepad_manager.poll_stick_binding("CameraStick");
        self.control_data[4] = ((camera_stick.0 >> 8) + 128) as u8;
        self.control_data[5] = ((camera_stick.1 >> 8) + 128) as u8;

        let movement_stick = inp_ctx.gamepad_manager.poll_stick_binding("MovementStick");
        self.control_data[6] = ((movement_stick.0 >> 8) + 128) as u8;
        self.control_data[7] = ((movement_stick.1 >> 8) + 128) as u8;
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_jmp(0x0040c570 as *mut (), Device::process as *const ());
        patch_jmp(0x0040d010 as *mut (), Device::acquire as *const ());
        patch_jmp(0x0040d060 as *mut (), Device::unacquire as *const ());
        patch_jmp(0x0040d0a0 as *mut (), Device::init as *const ());
        patch_jmp(0x0040d390 as *mut (), Device::release as *const ());

        patch_jmp(0x0040d3c0 as *mut (), Device::reset_actuators as *const ());

        // ActivateActuators was optimized into a different call
        // restore each individual call
        patch_call(0x004a90fb as *mut (), Device::activate_actuator as *const ());
        patch_call(0x004ac37f as *mut (), Device::activate_actuator as *const ());
        patch_call(0x004ac390 as *mut (), Device::activate_actuator as *const ());
        patch_call(0x004b18f2 as *mut (), Device::activate_actuator as *const ());
        patch_call(0x004b191d as *mut (), Device::activate_actuator as *const ());
        patch_call(0x004a9141 as *mut (), Device::activate_actuator as *const ());

        // same deal with all of these but they were only ever called once
        patch_call(0x004c0f03 as *mut (), Device::enable_actuators as *const ());

        patch_call(0x0040de99 as *mut (), Device::unpause as *const ());

        patch_call(0x0040de19 as *mut (), Device::pause as *const ());
    }
}

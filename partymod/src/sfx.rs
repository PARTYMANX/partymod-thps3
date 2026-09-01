use partymod_common::patch;

pub fn get_master_volume() -> f64 {
    unsafe {
        let get_miles_manager: unsafe extern "C" fn(bool) -> *const std::ffi::c_void =
            std::mem::transmute(0x00402f80);
        let release_miles_manager: unsafe extern "thiscall" fn() = std::mem::transmute(0x00402fd0);

        let miles_get_master_volume =
            0x0058d3ac as *const extern "stdcall" fn(*const std::ffi::c_void) -> i8;

        let miles_manager = get_miles_manager(false);

        if !miles_manager.is_null() {
            let mut result = *(miles_manager.byte_add(4) as *const i8);

            // if the volume is 0, that may mean it's just uninitialized
            // get the volume directly from miles sound system
            if result == 0 {
                result =
                    (*miles_get_master_volume)(*(miles_manager as *const *const std::ffi::c_void));
            }

            release_miles_manager();

            result as f64 / 127.0
        } else {
            0.0
        }
    }
}

pub fn get_sound_volume() -> f64 {
    unsafe {
        let get_miles_manager: unsafe extern "C" fn(bool) -> *const std::ffi::c_void =
            std::mem::transmute(0x00402f80);
        let release_miles_manager: unsafe extern "C" fn() = std::mem::transmute(0x00402fd0);

        let miles_manager = get_miles_manager(false);

        let result = *(miles_manager.byte_add(8) as *const i8);

        release_miles_manager();

        result as f64 / 127.0
    }
}

unsafe extern "thiscall" fn do_sound_cleanup(skate: *mut ()) {
    unsafe {
        let orig_cleanup: unsafe extern "thiscall" fn(*mut ()) = std::mem::transmute(0x004397a0);
        let sound_cleanup: unsafe extern "C" fn() = std::mem::transmute(0x00408c30);

        orig_cleanup(skate);
        sound_cleanup();
    }
}

unsafe fn patch_sound_cleanup() {
    unsafe {
        patch::patch_call(0x00439b1c as *mut (), do_sound_cleanup as *const ());
    }
}

unsafe extern "C" fn get_bgm_status() -> u32 {
    // if still playing, return 2. if done playing, return 1
    unsafe {
        let miles_stream_status =
            0x0058d390 as *const extern "stdcall" fn(*const ()) -> u32;
        let close_bgm_stream: unsafe extern "C" fn(u32) = std::mem::transmute(0x00407c30);

        let bgm_stream = *(0x005d0b78 as *const *const ());

        if !bgm_stream.is_null() {
            let status = (*miles_stream_status)(bgm_stream);

            match status {
                // playing
                4 => 2,
                // paused
                8 => 2,
                // anything else. likely stopped (2)
                _ => {
                    close_bgm_stream(0);

                    1
                },
            }
        } else {
            1
        }
    }
}

unsafe fn patch_bgm_pause() {
    unsafe {
        patch::patch_jmp(0x00407d40 as *mut (), get_bgm_status as *const ());
    }
}

unsafe extern "C" fn set_stream_pan(left_percentage: f32, right_percentage: f32, stream_idx: u32) {
    unsafe {
        let miles_set_stream_pan = 0x0058d398 as *const extern "stdcall" fn(*const (), u32);
        let miles_set_stream_volume = 0x0058d3c4 as *const extern "stdcall" fn(*const (), u32);

        let stream = *((0x005d0b88 as *const *const ()).byte_add(stream_idx as usize * 12));

        let scale_f32 = f32::max(left_percentage, right_percentage);
        let left_f32 = left_percentage / scale_f32;
        let right_f32 = right_percentage / scale_f32;

        let scale = (scale_f32 / 100.0) * 127.0;
        let vol = (get_sound_volume() as f32 * scale) as u32;

        let pan_rightness = ((right_f32 - left_f32) + 1.0) * 0.5;
        let pan = (pan_rightness * 127.0) as u32;

        (*miles_set_stream_volume)(stream, vol);
        (*miles_set_stream_pan)(stream, pan);
    }
}

unsafe fn patch_stream_volume() {
    unsafe {
        patch::patch_jmp(0x00407ea0 as *mut (), set_stream_pan as *const ());
    }
}

unsafe extern "C" fn set_sample_parameters(sample_idx:u32, left_percentage: f32, right_percentage: f32, rate: f32) {
    unsafe {
        let miles_set_sample_pan = 0x0058d3b4 as *const extern "stdcall" fn(*const (), u32);
        let miles_set_sample_volume = 0x0058d354 as *const extern "stdcall" fn(*const (), u32);
        let miles_set_sample_playback_rate = 0x0058d3b0 as *const extern "stdcall" fn(*const (), u32);

        let sample = *((0x00850cd0 as *const *const ()).byte_add(sample_idx as usize * 8));

        let scale_f32 = f32::max(left_percentage, right_percentage);
        let left_f32 = left_percentage / scale_f32;
        let right_f32 = right_percentage / scale_f32;

        let scale = (scale_f32 / 100.0) * 127.0;
        let vol = (get_sound_volume() as f32 * scale) as u32;

        let pan_rightness = ((right_f32 - left_f32) + 1.0) * 0.5;
        let pan = (pan_rightness * 127.0) as u32;

        let orig_sample_rate = *((0x00850cd0 as *const u32).byte_add(4 + (sample_idx as usize * 8)));
        let rate_f32 = ((orig_sample_rate as f32 / 44100.0) * (rate / 100.0)) as f32;
        let rate = (44100.0 * rate_f32) as u32;

        (*miles_set_sample_volume)(sample, vol);
        (*miles_set_sample_pan)(sample, pan);
        (*miles_set_sample_playback_rate)(sample, rate);
    }
}

unsafe fn patch_sample_parameters() {
    unsafe {
        patch::patch_jmp(0x00408f10 as *mut (), set_sample_parameters as *const ());
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_sound_cleanup();
        patch_bgm_pause();
        patch_stream_volume();
        patch_sample_parameters();
    }
}

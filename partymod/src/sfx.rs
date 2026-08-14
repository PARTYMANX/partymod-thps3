use partymod_common::patch;

pub fn get_master_volume() -> f64 {
    unsafe {
        let get_sfx_manager: unsafe extern "C" fn(bool) -> *const std::ffi::c_void
            = std::mem::transmute(0x00402f80);
        let release_sfx_manager: unsafe extern "thiscall" fn(*const std::ffi::c_void)
            = std::mem::transmute(0x00402fd0);

        let sfx_manager = get_sfx_manager(false);

        let result = *(sfx_manager.byte_add(4) as *const i8);

        release_sfx_manager(sfx_manager);

        result as f64 / 127.0
    }
}

pub fn get_sound_volume() -> f64 {
    unsafe {
        let get_sfx_manager: unsafe extern "C" fn(bool) -> *const std::ffi::c_void
            = std::mem::transmute(0x00402f80);
        let release_sfx_manager: unsafe extern "thiscall" fn(*const std::ffi::c_void)
            = std::mem::transmute(0x00402fd0);

        let sfx_manager = get_sfx_manager(false);

        let result = *(sfx_manager.byte_add(8) as *const i8);

        release_sfx_manager(sfx_manager);

        result as f64 / 127.0
    }
}

unsafe extern "thiscall" fn do_sound_cleanup(skate: *mut ()) {
    unsafe {
        let orig_cleanup: unsafe extern "thiscall" fn(*mut ())
            = std::mem::transmute(0x004397a0);
        let sound_cleanup: unsafe extern "C" fn()
            = std::mem::transmute(0x00408c30);

        orig_cleanup(skate);
        sound_cleanup();
    }
}

unsafe fn patch_sound_cleanup() {
    unsafe {
        patch::patch_call(0x00439b1c as *mut (), do_sound_cleanup as *const ());
    }
}

pub unsafe fn patch() {
    unsafe {
        patch_sound_cleanup();
    }
}

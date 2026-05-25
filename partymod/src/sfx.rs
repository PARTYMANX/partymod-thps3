use partymod_common::patch;

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

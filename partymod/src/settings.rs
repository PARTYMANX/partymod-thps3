use partymod_common::patch;

use crate::{config, gfx, window};

extern "C" fn init_settings() {
    // initializes settings values to ensure they're correct for startup
    window::init_settings();
    gfx::init_settings();

    unsafe {
        let ptr_high_bandwidth = 0x005b4e75 as *mut bool;
        let ptr_play_intro = 0x005b4e79 as *mut bool;
        
        *ptr_high_bandwidth = true;
        *ptr_play_intro = config::get_bool("Miscellaneous", "PlayIntro", true);
    }
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040b150 as *mut (), init_settings as *const ());
    }
}

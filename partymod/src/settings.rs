use partymod_common::patch;

use crate::window;

// TODO: move this stuff into lib.rs

extern "C" fn init_settings() {
    // initializes settings values to ensure they're correct for startup
    window::init_settings();

    unsafe {
        let ptr_high_bandwidth = 0x005b4e75 as *mut bool;
        let ptr_shadows = 0x005b4e76 as *mut bool;
        let ptr_particles = 0x005b4e77 as *mut bool;
        let ptr_animating_textures = 0x005b4e78 as *mut bool;
        let ptr_play_intro = 0x005b4e79 as *mut bool;
        let ptr_custom_settings = 0x008510b1 as *mut bool;
        let ptr_distance_fog = 0x008510b2 as *mut bool;
        let ptr_low_detail_models = 0x008510b3 as *mut bool;
        let ptr_frame_cap = 0x008510b4 as *mut bool;

        let ptr_bit_depth = 0x0085108c as *mut u32;

        *ptr_high_bandwidth = true;
        *ptr_play_intro = false;

        // TODO: move rest of these to gfx::d3d8 module
        *ptr_animating_textures = true;
        *ptr_particles = true;
        *ptr_shadows = true;
        *ptr_distance_fog = false;
        *ptr_low_detail_models = false;

        *ptr_custom_settings = true;
        *ptr_frame_cap = true;

        *ptr_bit_depth = 32;
    }
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_jmp(0x0040b150 as *mut (), init_settings as *const ());
    }
}

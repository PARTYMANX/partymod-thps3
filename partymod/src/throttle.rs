use partymod_common::{patch, syncunsafecell::SyncUnsafeCell, throttle::FramerateThrottle};

pub static THROTTLE_CONTEXT: SyncUnsafeCell<Option<FramerateThrottle>> = SyncUnsafeCell::new(None);

pub fn init() {
    let minimum_frame_length = std::time::Duration::from_secs_f64(1.0 / 60.0);

    unsafe {
        let ctx = &mut *THROTTLE_CONTEXT.get();
        *ctx = Some(FramerateThrottle::new(minimum_frame_length));
    }
}

pub fn throttle_frame() {
    let throttle_context = match unsafe { &mut *THROTTLE_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized throttle context!"),
    };

    throttle_context.throttle();
}

pub unsafe fn patch() {
    unsafe {
        patch::patch_byte(0x004c0507 as *mut (), 0xEB); // skip original framerate cap logic
        patch::patch_nop(0x004c04ef as *mut (), 24);
        patch::patch_call(0x004c04ef as *mut (), throttle_frame as *const ());
    }
}

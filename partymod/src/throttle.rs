use partymod_common::{syncunsafecell::SyncUnsafeCell, throttle::FramerateThrottle};

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

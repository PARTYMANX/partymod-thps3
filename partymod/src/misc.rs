use partymod_common::{patch, prng, syncunsafecell::SyncUnsafeCell};

struct MusicPrngContext(prng::Pcg32);

static MUSIC_PRNG_CONTEXT: SyncUnsafeCell<Option<MusicPrngContext>> = SyncUnsafeCell::new(None);

fn init_music_prng() {
    let ctx = unsafe { &mut *MUSIC_PRNG_CONTEXT.get() };

    let now = std::time::SystemTime::now();

    let seed = match now.duration_since(std::time::UNIX_EPOCH) {
        Ok(v) => v.as_millis() as u64,
        Err(e) => panic!("Error setting prng seed: {}", e),
    };

    *ctx = Some(MusicPrngContext(prng::Pcg32::new(seed)));
}

unsafe fn patch_music_prng() {
    unsafe {
        patch::patch_call(0x004c6b37 as *mut (), our_random as *const ());
    }
}

unsafe extern "C" fn our_random(out_of: u32) -> u32 {
    let prng_ctx = unsafe {
        match &mut *MUSIC_PRNG_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to get uninitialized music PRNG context!"),
        }
    };

    unsafe {
        let their_random: unsafe extern "C" fn(u32) -> u32
            = std::mem::transmute(0x0040e4c0);

        their_random(out_of);
    }

    let value = prng_ctx.0.get();

    value % out_of
}

// TODO: version display

pub fn init() {
    init_music_prng();
}

pub unsafe fn patch() {
    unsafe {
        patch_music_prng();
    }
}
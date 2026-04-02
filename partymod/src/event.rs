use partymod_common::patch;

use crate::sdl;

pub static mut EVENT_MANAGER_CONTEXT: std::mem::MaybeUninit<partymod_common::event::EventManager> =
    std::mem::MaybeUninit::uninit();

pub fn init() {
    unsafe {
        EVENT_MANAGER_CONTEXT =
            std::mem::MaybeUninit::new(partymod_common::event::EventManager::new());
    }
}

#[allow(static_mut_refs)]
fn process_events() {
    let manager = unsafe { EVENT_MANAGER_CONTEXT.assume_init_ref() };
    let sdl_context = unsafe { sdl::SDL_CONTEXT.assume_init_mut() };

    let mut event_pump = match sdl_context.sdl_context.event_pump() {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to process events: {}", e);
            return;
        }
    };

    manager.process_events(&mut event_pump);
}

#[allow(static_mut_refs)]
pub fn register_handler(handler: fn(&sdl3::event::Event)) {
    let manager = unsafe { EVENT_MANAGER_CONTEXT.assume_init_mut() };
    manager.register_handler(handler);
}

pub unsafe fn patch() {
    unsafe {
        // peekmessages at 00409280
        patch::patch_jmp(0x00409280 as *mut (), process_events as *const ());
        // there's another event handler for movies specifically, but I think I'll just disable that for the time being because we're just going to write a new player
    }
}

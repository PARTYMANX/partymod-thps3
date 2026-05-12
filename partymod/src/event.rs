use partymod_common::{
    event::EventManager, logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell,
};

use crate::{logger, sdl::SDL_CONTEXT};

pub static EVENT_MANAGER_CONTEXT: SyncUnsafeCell<Option<EventManager>> = SyncUnsafeCell::new(None);

pub fn init() {
    unsafe {
        let ctx = &mut *EVENT_MANAGER_CONTEXT.get();
        *ctx = Some(EventManager::new());
    }
}

fn process_events() {
    let manager = unsafe {
        let ctx = &*EVENT_MANAGER_CONTEXT.get();
        match ctx {
            Some(v) => v,
            None => panic!("Tried to get uninitialized event manager context!"),
        }
    };

    let sdl_context = unsafe {
        let ctx = &mut *SDL_CONTEXT.get();
        match ctx {
            Some(v) => v,
            None => panic!("Tried to get uninitialized event manager context!"),
        }
    };

    let mut event_pump = match sdl_context.sdl_context.event_pump() {
        Ok(v) => v,
        Err(e) => {
            logger::log(LogLevel::Error, &format!("Failed to process events: {}", e));
            return;
        }
    };

    manager.process_events(&mut event_pump);
}

pub fn register_handler(handler: fn(&sdl3::event::Event)) {
    let manager = unsafe {
        let ctx = &mut *EVENT_MANAGER_CONTEXT.get();
        match ctx {
            Some(v) => v,
            None => panic!("Tried to get uninitialized event manager context!"),
        }
    };

    manager.register_handler(handler);
}

pub unsafe fn patch() {
    unsafe {
        // peekmessages at 00409280
        patch::patch_jmp(0x00409280 as *mut (), process_events as *const ());
        // there's another event handler for movies specifically, but I think I'll just disable that for the time being because we're just going to write a new player
    }
}

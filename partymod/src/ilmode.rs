use partymod_common::{logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell};

use crate::{args, config, logger};

struct ILModeContext {
    enabled: bool,
    old_level: usize,
    old_progress: u32,
    old_pickups: u32,
}

static ILMODE_CONTEXT: SyncUnsafeCell<Option<ILModeContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    let enabled = args::has_arg("ilmode") || config::get_bool("Miscellaneous", "ILMode", false);

    if enabled {
        logger::log(LogLevel::Info, "IL Mode enabled!");
        unsafe { patch() };
    }

    let new_ctx = ILModeContext {
        enabled,
        old_level: 0,
        old_progress: 0,
        old_pickups: 0,
    };

    unsafe {
        let ctx = &mut *ILMODE_CONTEXT.get();
        *ctx = Some(new_ctx);
    }
}

pub fn is_enabled() -> bool {
    match unsafe { &mut *ILMODE_CONTEXT.get() } {
        Some(v) => v.enabled,
        None => panic!("Tried to use uninitialized IL mode context!"),
    }
}

static COMP_LEVELS: [bool; 10] = [
    false, // skateshop; not real
    false, // foundry
    false, // canada
    true,  // rio
    false, // suburbia
    false, // airport
    true,  // skater island
    false, // los angeles
    true,  // tokyo
    false, // cruise ship
];
const STATS_AND_BOARDS_MASK: u32 = 0x01f80000;

extern "C" fn retry_hook() {
    unsafe {
        let is_career_mode: unsafe extern "C" fn() -> bool = std::mem::transmute(0x00421540);
        if is_career_mode() {
            // reset progress
            let ctx = match &mut *ILMODE_CONTEXT.get() {
                Some(v) => v,
                None => panic!("Tried to use uninitialized IL mode context!"),
            };

            let career = *((*((*(0x008e1e90 as *const *const ())).byte_add(0x134)
                as *const *const ()))
                .byte_add(0x14) as *const *const ());

            let level = *((career as *const u32).byte_add(0x690)) as usize;

            if level < COMP_LEVELS.len() && !COMP_LEVELS[level] {
                let goals = career.byte_add(0x564 + (level - 1) * 8) as *mut u32;
                let someflags = career.byte_add(0x5e4 + (level - 1) * 8) as *mut u32;
                let pickups = career.byte_add(0x5e8 + (level - 1) * 8) as *mut u32;

                *someflags = 0; // keeps goal messages from disappearing

                if level != ctx.old_level {
                    ctx.old_level = level;
                    ctx.old_progress = *goals;
                    ctx.old_pickups = *pickups;
                }

                if *goals & ctx.old_progress != *goals {
                    ctx.old_progress = *goals | ctx.old_progress;
                }

                let pickup_flags = *pickups & STATS_AND_BOARDS_MASK;
                if pickup_flags & ctx.old_pickups != pickup_flags {
                    ctx.old_pickups = pickup_flags | ctx.old_pickups;
                }

                *goals = 0;
                *pickups ^= pickup_flags;
            }
        }

        let orig_func: unsafe extern "C" fn() = std::mem::transmute(0x00422400);
        orig_func();
    }
}

extern "C" fn load_requested_level_hook() {
    unsafe {
        let ctx = match &mut *ILMODE_CONTEXT.get() {
            Some(v) => v,
            None => panic!("Tried to use uninitialized IL mode context!"),
        };

        if ctx.old_level != 0 && ctx.old_level < COMP_LEVELS.len() && !COMP_LEVELS[ctx.old_level] {
            // restore progress

            let career = *((*((*(0x008e1e90 as *const *const ())).byte_add(0x134)
                as *const *const ()))
                .byte_add(0x14) as *const *const ());

            let goals = career.byte_add(0x564 + (ctx.old_level - 1) * 8) as *mut u32;
            let pickups = career.byte_add(0x5e8 + (ctx.old_level - 1) * 8) as *mut u32;

            *goals = *goals | ctx.old_progress;
            *pickups = *pickups | ctx.old_pickups;
        }

        let orig_func: unsafe extern "C" fn() = std::mem::transmute(0x004220c0);
        orig_func();
    }
}

unsafe fn patch() {
    unsafe {
        patch::patch_u32(0x005b801c as *mut (), retry_hook as *const () as u32);
        patch::patch_u32(
            0x005b7ffc as *mut (),
            load_requested_level_hook as *const () as u32,
        );
    }
}

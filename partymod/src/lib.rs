use partymod_common::{logger::LogLevel, patch};

mod args;
mod config;
mod event;
mod file;
mod gameplay;
mod gfx;
mod glyph;
mod ilmode;
mod input;
mod logger;
mod misc;
mod movie;
mod net;
mod profile;
mod sdl;
mod settings;
mod sfx;
mod throttle;
mod window;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn init_patch() {
    let exe_path = match std::env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(e) => panic!("failed to get current exe path: {e}"),
    };

    args::init();
    config::init(exe_path.parent().unwrap());

    logger::init();
    sdl::init();
    event::init();
    file::init();
    throttle::init();
    window::init();

    event::register_handler(handle_exit_event);
    event::register_handler(window::handle_event);
    // NOTE: input event handler is registered when input is initialized

    if config::get_bool("Miscellaneous", "Debug", false) {
        partymod_common::console::init_console();
    }

    println!("PARTYMOD for THPS3 {}", VERSION);
    logger::log(LogLevel::Info, &format!("PARTYMOD for THPS3 {}", VERSION));

    misc::init();
    gfx::init();
    ilmode::init();
    gameplay::init();
    glyph::init();
    net::init();
    profile::init();
}

extern "C" fn init_and_get_version() -> u32 {
    init_patch();

    net::get_server_version()
}

fn handle_exit_event(e: &sdl3::event::Event) {
    match e {
        sdl3::event::Event::Quit { .. } => {
            unsafe {
                // call the script reset engine function to gracefully exit the main loop
                let script_reset_engine: unsafe extern "C" fn() =
                    std::mem::transmute(0x0041ba40 as *const ());
                script_reset_engine();
            }
        }
        _ => {}
    }
}

unsafe fn patch_init() {
    unsafe {
        // patch in a call to our initialization function
        patch::patch_jmp(0x00411d70 as *mut (), init_and_get_version as *const ());

        // remove the launcher check/startup
        patch::patch_nop(0x0040b9da as *mut (), 7); // remove call to run launcher
        patch::patch_byte(0x0040b9e1 as *mut (), 0xeb); // change launcher branch from JZ to JMP
        patch::patch_nop(0x0040b9fc as *mut (), 12); // remove call to change registry
    }
}

extern "C" fn fast_quit() {
    std::process::exit(0);
}

unsafe fn patch_fast_quit() {
    unsafe {
        // patch in a function to quickly exit the program and skip cleanup since the OS handles that just fine
        patch::patch_call(0x0040ae28 as *mut (), fast_quit as *const ());
    }
}

extern "C" fn get_version_number(script: *const std::ffi::c_void) -> u32 {
    unsafe {
        let unk_func: unsafe extern "thiscall" fn(
            *const std::ffi::c_void,
            *const std::ffi::c_char,
            *mut *const std::ffi::c_void,
            u32,
        ) = std::mem::transmute(0x00429dc0);
        let set_string: unsafe extern "C" fn(*const std::ffi::c_void, *const std::ffi::c_char) =
            std::mem::transmute(0x004ce940);

        let mut ptr = std::ptr::null();
        unk_func(script, c"id".as_ptr(), &mut ptr, 1);

        let mut str = format!("{}", VERSION);

        if gameplay::get_compatibility_mode() {
            str = format!("{} (1.01 compatibility mode)", str);
        }

        if ilmode::is_enabled() {
            str = format!("{} - IL MODE ENABLED", str);
        }

        if let Some(profile_name) = profile::get_profile_name() {
            str = format!("{} - profile: {}", str, profile_name);
        }

        let cstr = std::ffi::CString::new(str).unwrap();

        set_string(ptr, cstr.as_ptr());
    }

    1
}

// patches the version number string on the main menu
unsafe fn patch_version_number() {
    unsafe {
        patch::patch_jmp(0x00425e10 as *mut (), get_version_number as *const ());
    }
}

//#[unsafe(export_name = "DllMain")]
#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_hinst_dll: usize, fdw_reason: u32, _lp_reserved: usize) -> i32 {
    match fdw_reason {
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH => unsafe {
            patch_init();
            patch_fast_quit();
            patch_version_number();
            event::patch();
            window::patch();
            settings::patch();
            file::patch();

            input::patch();

            gameplay::patch();
            gfx::patch();
            sfx::patch();
            throttle::patch();
            misc::patch();
            movie::patch();
        },
        windows_sys::Win32::System::SystemServices::DLL_THREAD_ATTACH => {}
        windows_sys::Win32::System::SystemServices::DLL_THREAD_DETACH => {}
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_DETACH => {}
        _ => {}
    }

    1
}

use partymod_common::patch;

mod config;
mod event;
mod input;
mod logger;
mod sdl;
mod settings;
mod window;

const VERSION: &str = env!("CARGO_PKG_VERSION");

extern "C" fn init_patch() {
    let exe_path = match std::env::current_exe() {
        Ok(exe_path) => exe_path,
        Err(e) => panic!("failed to get current exe path: {e}"),
    };

    config::init(exe_path.parent().unwrap());

    /*if config::get_config_bool("Graphics", "UseNewRenderer", true) {
        unsafe { gfx_vk::patch() };
        gfx_vk::init();
    } else {
        unsafe { gfx_d3d::patch() };
    }*/

    logger::init();
    sdl::init();
    event::init();
    window::init();

    event::register_handler(handle_exit_event);
    event::register_handler(window::handle_event);
    //event::register_handler(input::handle_event);

    if config::get_bool("Miscellaneous", "Debug", false) {
        partymod_common::console::init_console();
    }

    println!("PARTYMOD for THPS3 {}", VERSION);
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
        // replaces the launcher check/startup
        patch::patch_nop(0x0040b9da as *mut (), 7); // remove call to run launcher
        patch::patch_byte(0x0040b9e1 as *mut (), 0xeb); // change launcher branch from JZ to JMP
        patch::patch_nop(0x0040b9fc as *mut (), 12); // remove call to change registry

        patch::patch_call(0x0040b9da as *mut (), init_patch as *const ()); // use now unused space to call our init func
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

// TODO: patch 00411d70 to return a version number (default should be 0x00010001)

//#[unsafe(export_name = "DllMain")]
#[unsafe(no_mangle)]
pub extern "stdcall" fn DllMain(_hinst_dll: usize, fdw_reason: u32, _lp_reserved: usize) -> i32 {
    match fdw_reason {
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH => {
            unsafe {
                patch_init();
                patch_fast_quit();
                event::patch();
                window::patch();
                settings::patch();

                patch::patch_nop(0x004079a8 as *mut (), 0x004079f4 + 5 - 0x004079a8); // TEMPORARY - remove video playback
                input::patch();
            }
        }
        windows_sys::Win32::System::SystemServices::DLL_THREAD_ATTACH => {}
        windows_sys::Win32::System::SystemServices::DLL_THREAD_DETACH => {}
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_DETACH => {}
        _ => {}
    }

    1
}

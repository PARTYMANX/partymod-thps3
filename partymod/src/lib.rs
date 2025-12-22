use partymod_common::{patch, config};

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

    //sdl::init_sdl();
    //gameplay::init();
    //event::init();

    //event::register_handler(window::handle_event);
    //event::register_handler(input::handle_event);

    if config::get_config_bool("Miscellaneous", "Debug", false) {
        partymod_common::console::init_console();
    }

    println!("PARTYMOD for THPS3 {}", VERSION);
}

unsafe fn patch_init() {
    // patch in a call to our initialization function
    // replaces the launcher check/startup

    patch::patch_nop(0x0040b9da as *mut (), 7); // remove call to run launcher
    patch::patch_byte(0x0040b9e1 as *mut (), 0xeb);   // change launcher branch from JZ to JMP
    patch::patch_nop(0x0040b9fc as *mut (), 12);    // remove call to change registry

    patch::patch_call(0x0040b9da as *mut (), init_patch as *const ());  // use now unused space to call our init func
}

//#[unsafe(export_name = "DllMain")]
#[no_mangle]
pub extern "stdcall" fn DllMain(_hinst_dll: usize, fdw_reason: u32, _lp_reserved: usize) -> i32 {
    match fdw_reason {
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH => {
            unsafe {
                // window MUST be patched before initializer
                patch_init();
                //window::patch_window();
                //event::patch_event_handler();
                //input::patch();
            }   
    
        },
        windows_sys::Win32::System::SystemServices::DLL_THREAD_ATTACH => {

        },
        windows_sys::Win32::System::SystemServices::DLL_THREAD_DETACH => {

        },
        windows_sys::Win32::System::SystemServices::DLL_PROCESS_DETACH => {

        },
        _ => {},
    }

    1
}
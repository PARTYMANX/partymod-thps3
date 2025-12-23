use std::ffi::CString;

struct ConfigContext {
    filename: std::ffi::CString,
}

unsafe impl Send for ConfigContext{}

static mut CONFIG_CONTEXT: std::mem::MaybeUninit<ConfigContext> = std::mem::MaybeUninit::uninit();

pub fn init(exe_path: &std::path::Path) {
    let filename = std::ffi::CString::new(exe_path.join("partymod.ini").to_str().unwrap()).unwrap();

    unsafe {
        CONFIG_CONTEXT = std::mem::MaybeUninit::new(ConfigContext {
            filename,
        });
    }
}

#[allow(static_mut_refs)]
pub fn get_config_int(section: &str, key: &str, default: i32) -> u32 {
    let app_name = CString::new(section).unwrap();
    let key_name = CString::new(key).unwrap();

    unsafe {
        windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileIntA(
            app_name.as_ptr() as *const u8, 
            key_name.as_ptr() as *const u8, 
            default, 
            CONFIG_CONTEXT.assume_init_ref().filename.as_ptr() as *const u8
        )
    }
}

#[allow(static_mut_refs)]
pub fn get_config_bool(section: &str, key: &str, default: bool) -> bool {
    let app_name = CString::new(section).unwrap();
    let key_name = CString::new(key).unwrap();

    let result = unsafe {
        windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileIntA(
            app_name.as_ptr() as *const u8, 
            key_name.as_ptr() as *const u8, 
            default as i32, 
            CONFIG_CONTEXT.assume_init_ref().filename.as_ptr() as *const u8
        )
    };

    result != 0
}

#[allow(static_mut_refs)]
pub fn get_config_string(section: &str, key: &str, default: &str) -> String {
    let app_name = CString::new(section).unwrap();
    let key_name = CString::new(key).unwrap();
    let default = CString::new(default).unwrap();
    let mut result_buf = [0u8; 256];

    let result = unsafe {
        windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileStringA(
            app_name.as_ptr() as *const u8, 
            key_name.as_ptr() as *const u8, 
            default.as_ptr() as *const u8, 
            result_buf.as_mut_ptr(), 
            256, 
            CONFIG_CONTEXT.assume_init_ref().filename.as_ptr() as *const u8
        )
    };

    if result != 0 {
        String::from_utf8(result_buf.to_vec()).unwrap()
    } else {
        panic!("AAAHHHH");
    }
}
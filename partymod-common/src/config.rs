use std::ffi::CString;

pub struct ConfigFile {
    filename: std::ffi::CString,
}

impl ConfigFile {
    pub fn new(filepath: &std::path::Path) -> Self {
        let filename = std::ffi::CString::new(filepath.to_str().unwrap()).unwrap();

        Self { filename }
    }

    pub fn get_config_int(&self, section: &str, key: &str, default: i32) -> i32 {
        let app_name = CString::new(section).unwrap();
        let key_name = CString::new(key).unwrap();

        let result = unsafe {
            windows_sys::Win32::System::WindowsProgramming::GetPrivateProfileIntA(
                app_name.as_ptr() as *const u8,
                key_name.as_ptr() as *const u8,
                default,
                self.filename.as_ptr() as *const u8,
            )
        };

        result as i32
    }

    pub fn get_config_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.get_config_int(section, key, default as i32) != 0
    }

    pub fn get_config_string(&self, section: &str, key: &str, default: &str) -> String {
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
                self.filename.as_ptr() as *const u8,
            )
        };

        if result != 0 {
            let result_slice = &result_buf[0..result as usize];
            String::from_utf8(result_slice.to_vec()).unwrap()
        } else {
            panic!("Failed to parse utf8 string");
        }
    }
}

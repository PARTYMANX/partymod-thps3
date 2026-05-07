use windows::{Win32::System::WindowsProgramming::{GetPrivateProfileIntW, GetPrivateProfileStringW, WritePrivateProfileStringW}, core::HSTRING};

pub struct ConfigFile {
    filename: HSTRING,
}

impl ConfigFile {
    pub fn new(exe_path: &std::path::Path) -> Self {
        let filename = HSTRING::from(exe_path);
        Self {
            filename,
        }
    }

    pub fn get_config_int(&self, section: &str, key: &str, default: i32) -> i32 {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);

        unsafe {
            GetPrivateProfileIntW(
                &app_name,
                &key_name,
                default,
                &self.filename,
            )
        }
    }

    pub fn set_config_int(&self, section: &str, key: &str, value: i32) {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);
        let value_str = HSTRING::from(value.to_string());

        unsafe {
            let _ = WritePrivateProfileStringW(
                &app_name,
                &key_name,
                &value_str,
                &self.filename,
            );
        }
    }

    pub fn get_config_bool(&self, section: &str, key: &str, default: bool) -> bool {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);

        let result = unsafe {
            GetPrivateProfileIntW(
                &app_name,
                &key_name,
                default as i32,
                &self.filename,
            )
        };

        result != 0
    }

    pub fn set_config_bool(&self, section: &str, key: &str, value: bool) {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);
        let value_str = HSTRING::from((value as i32).to_string());

        unsafe {
            let _ = WritePrivateProfileStringW(
                &app_name,
                &key_name,
                &value_str,
                &self.filename,
            );
        }
    }

    pub fn get_config_string(&self, section: &str, key: &str, default: &str) -> String {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);
        let default = HSTRING::from(default);
        let mut result_buf = [0u16; 256];

        let result = unsafe {
            GetPrivateProfileStringW(
                &app_name,
                &key_name,
                &default,
                Some(&mut result_buf),
                &self.filename,
            )
        };

        if result != 0 {
            String::from_utf16(&result_buf).unwrap()
        } else {
            panic!("Failed to parse utf16 string");
        }
    }

    pub fn set_config_string(&self, section: &str, key: &str, value: &str) {
        let app_name = HSTRING::from(section);
        let key_name = HSTRING::from(key);
        let value_str = HSTRING::from(value);

        unsafe {
            let _ = WritePrivateProfileStringW(
                &app_name,
                &key_name,
                &value_str,
                &self.filename,
            );
        }
    }
}
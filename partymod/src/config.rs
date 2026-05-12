use partymod_common::{config::ConfigFile, syncunsafecell::SyncUnsafeCell};

pub static CONFIG_CONTEXT: SyncUnsafeCell<Option<ConfigFile>> = SyncUnsafeCell::new(None);

pub fn init(exe_path: &std::path::Path) {
    let filepath = exe_path.join("partymod.ini");

    unsafe {
        let ctx = &mut *CONFIG_CONTEXT.get();
        *ctx = Some(ConfigFile::new(filepath.as_path()));
    }
}

pub fn get_int(section: &str, key: &str, default: i32) -> i32 {
    unsafe {
        let ctx = &*CONFIG_CONTEXT.get();

        match ctx {
            Some(v) => v.get_config_int(section, key, default),
            None => panic!("Tried to get uninitialized config file!"),
        }
    }
}

pub fn get_bool(section: &str, key: &str, default: bool) -> bool {
    unsafe {
        let ctx = &*CONFIG_CONTEXT.get();

        match ctx {
            Some(v) => v.get_config_bool(section, key, default),
            None => panic!("Tried to get uninitialized config file!"),
        }
    }
}

pub fn get_string(section: &str, key: &str, default: &str) -> String {
    unsafe {
        let ctx = &*CONFIG_CONTEXT.get();

        match ctx {
            Some(v) => v.get_config_string(section, key, default),
            None => panic!("Tried to get uninitialized config file!"),
        }
    }
}

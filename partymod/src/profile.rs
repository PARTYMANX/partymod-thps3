use std::{ffi::CString, fs, path::Path, pin::Pin};

use partymod_common::{
    logger::LogLevel, patch, syncunsafecell::SyncUnsafeCell,
};

use crate::{args, logger};

pub struct ProfileContext {
    profile_name: Option<String>,
    _save_path_long: Pin<CString>,
    _save_path_short: Pin<CString>,
}

pub static PROFILE_CONTEXT: SyncUnsafeCell<Option<ProfileContext>> = SyncUnsafeCell::new(None);

pub fn init() {
    let (profile_name, save_path_short, save_path_long) = match args::get_arg_value("profile") {
        Some(profile) => {
            let mut profile_safe = profile.to_lowercase();
            profile_safe.retain(|c| c.is_ascii_alphanumeric() || c == '-');

            logger::log(LogLevel::Info, &format!("Using profile \"{}\"", profile_safe));

            let pinned_profile_long = Pin::new(CString::new(format!("%ssettings\\OptionsAndPros\\{}\\main.opt", profile_safe)).unwrap());
            let pinned_profile_short = Pin::new(CString::new(format!("/OptionsAndPros/{}/main.opt", profile_safe)).unwrap());

            // TODO: create path if it doesn't already exist
            let path_string = format!("./data/settings/OptionsAndPros/{}", profile_safe);
            let path = Path::new(&path_string);
            let _ = fs::create_dir_all(&path);

            unsafe {
                patch_save_file_path(&pinned_profile_long, &pinned_profile_short);
            }

            (Some(profile_safe), pinned_profile_short, pinned_profile_long)
        }
        None => {
            (None, Pin::new(CString::from(c"")), Pin::new(CString::from(c"")))
        }
    };

    unsafe {
        let ctx = &mut *PROFILE_CONTEXT.get();
        *ctx = Some(ProfileContext {
            profile_name,
            _save_path_short: save_path_short,
            _save_path_long: save_path_long,
        });
    }
}

pub fn get_profile_name() -> Option<String> {
    let ctx = match unsafe { &mut *PROFILE_CONTEXT.get() } {
        Some(v) => v,
        None => panic!("Tried to use uninitialized file context!"),
    };

    ctx.profile_name.clone()
}

unsafe fn patch_save_file_path(profile_string_long: &Pin<CString>, profile_string_short: &Pin<CString>) {
    unsafe {
        patch::patch_u32((0x0040a152 + 1) as *mut (), profile_string_long.as_ptr() as *const () as u32);
        patch::patch_u32((0x0041481e + 1) as *mut (), profile_string_short.as_ptr() as *const () as u32);
    }
}

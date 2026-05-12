use partymod_common::{
    logger::{LogLevel, Logger, PrintlnLogger},
    syncunsafecell::SyncUnsafeCell,
};

use crate::config;

pub static LOGGER_CONTEXT: SyncUnsafeCell<Option<PrintlnLogger>> = SyncUnsafeCell::new(None);

pub fn init() {
    let level = match config::get_int("Logging", "Level", 5) {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    unsafe {
        let ctx = &mut *LOGGER_CONTEXT.get();
        *ctx = Some(PrintlnLogger::new(level));
    }
}

pub fn log(level: LogLevel, msg: &str) {
    unsafe {
        let ctx = &*LOGGER_CONTEXT.get();

        match ctx {
            Some(v) => v.log(level, msg),
            None => panic!("Tried to use uninitialized logger context!"),
        }
    }
}

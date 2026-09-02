use std::{path::Path, sync::Arc};

use partymod_common::{
    logger::{FileLogger, LogLevel, Logger, MultiLogger, PrintlnLogger},
    syncunsafecell::SyncUnsafeCell,
};

use crate::config;

pub static LOGGER_CONTEXT: SyncUnsafeCell<Option<Arc<MultiLogger>>> = SyncUnsafeCell::new(None);

pub fn init() {
    let mut loggers: Vec<Arc<dyn Logger>> = Vec::new();

    let level = match config::get_int("Logging", "Level", 5) {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    if config::get_bool("Logging", "LogToFile", false) {
        let file_path_str = config::get_string("Logging", "LogFile", "partymod.log");

        let file_path = Path::new(&file_path_str);

        match FileLogger::new(level, file_path) {
            Some(v) => loggers.push(Arc::new(v)),
            None => {}
        }
    }

    loggers.push(Arc::new(PrintlnLogger::new(level)));

    unsafe {
        let ctx = &mut *LOGGER_CONTEXT.get();
        *ctx = Some(Arc::new(MultiLogger::new(loggers)));
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

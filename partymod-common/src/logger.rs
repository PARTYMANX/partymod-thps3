use std::{
    fmt,
    fs::File,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, msg: &str);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn priority(&self) -> u32 {
        match self {
            LogLevel::Trace => 4,
            LogLevel::Debug => 3,
            LogLevel::Info => 2,
            LogLevel::Warn => 1,
            LogLevel::Error => 0,
        }
    }

    pub fn from_priority(priority: u32) -> LogLevel {
        match priority {
            0 => LogLevel::Error,
            1 => LogLevel::Warn,
            2 => LogLevel::Info,
            3 => LogLevel::Debug,
            4 => LogLevel::Trace,
            _ => LogLevel::Trace,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "   TRACE"),
            LogLevel::Debug => write!(f, "   DEBUG"),
            LogLevel::Info => write!(f, "    INFO"),
            LogLevel::Warn => write!(f, "    WARN"),
            LogLevel::Error => write!(f, "   ERROR"),
        }
    }
}

pub struct PrintlnLogger {
    level: LogLevel,
}

impl PrintlnLogger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

impl Logger for PrintlnLogger {
    fn log(&self, level: LogLevel, msg: &str) {
        if level.priority() <= self.level.priority() {
            println!("{} {}", level, msg);
        }
    }
}

pub struct FileLogger {
    level: LogLevel,
    file: Mutex<File>,
}

impl FileLogger {
    pub fn new(level: LogLevel, file_path: &Path) -> Option<Self> {
        match File::create(file_path) {
            Ok(file) => Some(Self {
                level,
                file: Mutex::new(file),
            }),
            Err(_) => None,
        }
    }
}

impl Logger for FileLogger {
    fn log(&self, level: LogLevel, msg: &str) {
        if level.priority() <= self.level.priority() {
            let mut file = self.file.lock().unwrap();

            let _ = file.write(&format!("{} {}\n", level, msg).as_bytes());
        }
    }
}

pub struct MultiLogger {
    loggers: Vec<Arc<dyn Logger>>,
}

impl MultiLogger {
    pub fn new(loggers: Vec<Arc<dyn Logger>>) -> Self {
        Self { loggers }
    }
}

impl Logger for MultiLogger {
    fn log(&self, level: LogLevel, msg: &str) {
        for logger in &self.loggers {
            logger.log(level, msg);
        }
    }
}

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::Local;

pub struct Logger {
    log_file: Option<Arc<Mutex<fs::File>>>,
    log_level: LogLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl LogLevel {
    pub fn from_str(level: &str) -> Self {
        match level.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }
}

impl Logger {
    pub fn new(log_dir: &str, level: LogLevel) -> io::Result<Self> {
        // Create log directory if it doesn't exist
        fs::create_dir_all(log_dir)?;
        
        // Create log file with date suffix
        let date = Local::now().format("%Y-%m-%d");
        let log_path = PathBuf::from(log_dir).join(format!("vm-manager-{}.log", date));
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        Ok(Self {
            log_file: Some(Arc::new(Mutex::new(file))),
            log_level,
        })
    }
    
    pub fn console_only(level: LogLevel) -> Self {
        Self {
            log_file: None,
            log_level,
        }
    }
    
    pub fn log(&self, level: LogLevel, module: &str, message: &str) {
        if level > self.log_level {
            return;
        }
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = match level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };
        
        let log_line = format!("{} [{}] {}: {}\n", timestamp, level_str, module, message);
        
        // Print to console (with color)
        match level {
            LogLevel::Error => eprint!("\x1b[31m{}\x1b[0m", log_line),
            LogLevel::Warn => eprint!("\x1b[33m{}\x1b[0m", log_line),
            LogLevel::Info => print!("{}", log_line),
            LogLevel::Debug => print!("\x1b[36m{}\x1b[0m", log_line),
            LogLevel::Trace => print!("\x1b[90m{}\x1b[0m", log_line),
        }
        
        // Write to file if configured
        if let Some(log_file) = &self.log_file {
            if let Ok(mut file) = log_file.lock() {
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }
    
    pub fn error(&self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message);
    }
    
    pub fn warn(&self, module: &str, message: &str) {
        self.log(LogLevel::Warn, module, message);
    }
    
    pub fn info(&self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message);
    }
    
    pub fn debug(&self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message);
    }
    
    pub fn trace(&self, module: &str, message: &str) {
       
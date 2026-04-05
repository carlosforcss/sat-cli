use chrono;

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, message: &str) {
        let str_datetime = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        eprintln!("[INFO] {} {}", str_datetime, message);
    }
}

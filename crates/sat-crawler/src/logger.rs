use chrono;

pub struct Logger;

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Logger {
    pub fn new() -> Self {
        Self
    }

    pub fn info(&self, message: &str) {
        eprintln!("[INFO] {} {}", now_str(), message);
    }

    pub fn error(&self, message: &str) {
        eprintln!("[ERROR] {} {}", now_str(), message);
    }
}

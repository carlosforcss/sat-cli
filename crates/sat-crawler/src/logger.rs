use chrono;
use std::io::{self, Write};
use uuid::Uuid;

pub struct Logger {
    filename: String,
}

impl Logger {
    pub fn new(filename: Option<String>) -> Self {
        let filename = match filename {
            Some(filename) => filename,
            None => {
                let uuid = Uuid::new_v4();
                format!("log_{}.txt", uuid)
            }
        };
        Self { filename: filename }
    }

    pub fn info(&self, message: &str) {
        let str_datetime = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        eprintln!("[INFO] {} {}", str_datetime, message);
    }
}

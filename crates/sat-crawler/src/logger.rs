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
        println!("[INFO] {}", message);
    }
}

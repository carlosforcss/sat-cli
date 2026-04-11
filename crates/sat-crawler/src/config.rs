use dirs;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LoginType {
    #[default]
    Ciec,
    Fiel,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Credentials {
    pub login_type: LoginType,
    pub username: String,
    pub password: String,
    pub crt_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CrawlerOptions {
    pub headless: bool,
    pub sandbox: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub credentials: Credentials,
    pub options: CrawlerOptions,
}

pub struct CrawlerConfigBuilder {
    credentials: Option<Credentials>,
    headless: bool,
    sandbox: bool,
}

impl CrawlerConfigBuilder {
    pub fn new() -> Self {
        Self {
            credentials: None,
            headless: true,
            sandbox: true,
        }
    }

    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.credentials = Some(Credentials {
            login_type: LoginType::Ciec,
            username,
            password,
            crt_path: None,
            key_path: None,
        });
        self
    }

    pub fn with_head(mut self) -> Self {
        self.headless = false;
        self
    }

    pub fn no_sandbox(mut self) -> Self {
        self.sandbox = false;
        self
    }

    pub fn build(self) -> Result<CrawlerConfig, String> {
        if self.credentials.is_none() {
            return Err("Credentials are required".to_string());
        }
        Ok(CrawlerConfig {
            credentials: self.credentials.unwrap(),
            options: CrawlerOptions {
                headless: self.headless,
                sandbox: self.sandbox,
            },
        })
    }
}

impl CrawlerConfig {
    pub fn new(credentials: Credentials, opts: CrawlerOptions) -> Self {
        let instance = Self {
            credentials: credentials,
            options: opts,
        };
        instance.update_configuration_file();
        instance
    }

    pub fn new_from_file() -> Self {
        if !Self::check_if_config_json_exists() {
            return Self::default();
        }
        let home = dirs::home_dir().unwrap().join("sat-cli");
        let config_path = home.to_str().unwrap();
        let config_json = fs::read_to_string(format!("{}/config.json", config_path)).unwrap();
        serde_json::from_str(&config_json).unwrap()
    }

    pub fn update_configuration_file(&self) {
        let config_dir = dirs::home_dir().unwrap().join("sat-cli");
        let config_json = serde_json::to_string_pretty(&self).unwrap();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.json"), config_json).unwrap();
    }

    fn check_if_config_json_exists() -> bool {
        let home = dirs::home_dir().unwrap();
        let config_path = home.join("sat-cli").join("config.json");
        config_path.exists()
    }

    pub fn builder() -> CrawlerConfigBuilder {
        CrawlerConfigBuilder::new()
    }
}

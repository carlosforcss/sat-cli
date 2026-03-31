pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone)]
pub struct CrawlerOptions {
    pub headless: bool,
    pub sandbox: bool,
}

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
            username: username,
            password: password,
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
        Self {
            credentials: credentials,
            options: opts,
        }
    }
}

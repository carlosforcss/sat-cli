pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Default, Clone)]
pub struct CrawlerOptions {
    pub headless: Option<bool>,
    pub sandbox: Option<bool>,
}

pub struct CrawlerConfig {
    pub credentials: Credentials,
    pub options: CrawlerOptions,
}

impl CrawlerConfig {
    pub fn new(credentials: Credentials, opts: CrawlerOptions) -> Self {
        Self {
            credentials: credentials,
            options: opts,
        }
    }
}

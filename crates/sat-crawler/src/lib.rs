mod crawls;
mod utils;

#[derive(Debug, Clone)]
pub struct CrawlerResponse {
    pub success: bool,
    pub message: String,
}


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

pub enum Crawler {
    ValidateCredentials(CrawlerConfig),
    DownloadInvoices(CrawlerConfig),
}

impl Crawler {
    pub async fn run(&self) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
        match &self {
            Crawler::ValidateCredentials(config) => {
                let response = crawls::run_validate_credentials_crawler(config).await?;
                Ok(response)
            }
            Crawler::DownloadInvoices(config) => {
                let response = crawls::run_download_invoices_crawler(
                    config.credentials.username.clone(), 
                    config.credentials.password.clone(),
                ).await?;
                Ok(response)
            }
        }
    }
}

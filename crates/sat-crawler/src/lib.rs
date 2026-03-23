mod crawls;
mod logger;
mod utils;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub enum CrawlerType {
    DownloadInvoices,
    ValidateCredentials,
}

pub struct Crawler {
    pub crawler_type: CrawlerType,
    pub config: CrawlerConfig,
    logger: logger::Logger,
}

impl Crawler {
    pub fn new(crawler_type: CrawlerType, config: CrawlerConfig) -> Self {
        Self {
            crawler_type: crawler_type,
            config: config,
            logger: logger::Logger::new(None),
        }
    }

    pub async fn run(&self) -> CrawlerResponse {
        let response = match &self.crawler_type {
            CrawlerType::ValidateCredentials => {
                let response = crawls::run_validate_credentials_crawler(&self).await;
                response
            }
            CrawlerType::DownloadInvoices => {
                let response = crawls::run_download_invoices_crawler(&self).await;
                response
            }
        };
        match response {
            Ok(response) => response,
            Err(err) => {
                info!("Crawler error: {:?}", &err);
                return CrawlerResponse {
                    success: false,
                    message: format!("Crawler error: {:?}", err),
                };
            }
        }
    }
}

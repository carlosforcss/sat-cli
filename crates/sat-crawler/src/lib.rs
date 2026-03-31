mod config;
mod crawls;
mod logger;
mod utils;
pub use crate::config::CrawlerConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlerResponse {
    pub success: bool,
    pub message: String,
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
                return CrawlerResponse {
                    success: false,
                    message: format!("Crawler error: {:?}", err),
                };
            }
        }
    }
}

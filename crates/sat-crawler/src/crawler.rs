use crate::config::CrawlerConfig;
use crate::crawls;
use crate::logger;
use chromiumoxide::{Browser, BrowserConfig, Handler};
use serde::{Deserialize, Serialize};
use std::error::Error;

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
    pub logger: logger::Logger,
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

    pub async fn build_browser(&self) -> Result<(Browser, Handler), Box<dyn Error>> {
        let dir = tempfile::tempdir()?;

        let mut browser_config_builder = BrowserConfig::builder().user_data_dir(dir.path());

        if !self.config.options.headless {
            browser_config_builder = browser_config_builder.with_head();
        }

        if !self.config.options.sandbox {
            browser_config_builder = browser_config_builder.no_sandbox();
        }

        let browser_config = browser_config_builder.build()?;
        let (browser, handler) = Browser::launch(browser_config).await?;

        Ok((browser, handler))
    }
}

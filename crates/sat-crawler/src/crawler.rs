use crate::config::{CrawlerConfig, CrawlerFilters};
use crate::crawls;
use crate::events::{SharedInvoiceDownloadDecider, SharedInvoiceEventHandler};
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
    DownloadIssuedInvoices,
    DownloadReceivedInvoices,
    ValidateCredentials,
}

pub struct Crawler {
    pub crawler_type: CrawlerType,
    pub config: CrawlerConfig,
    pub logger: logger::Logger,
    pub event_handler: Option<SharedInvoiceEventHandler>,
    pub download_decider: Option<SharedInvoiceDownloadDecider>,
    pub filters: CrawlerFilters,
}

impl Crawler {
    pub fn new(crawler_type: CrawlerType, config: CrawlerConfig) -> Self {
        Self {
            crawler_type: crawler_type,
            config: config,
            logger: logger::Logger::new(),
            event_handler: None,
            download_decider: None,
            filters: CrawlerFilters::default(),
        }
    }

    pub fn with_event_handler(mut self, handler: SharedInvoiceEventHandler) -> Self {
        self.event_handler = Some(handler);
        self
    }

    pub fn with_download_decider(mut self, decider: SharedInvoiceDownloadDecider) -> Self {
        self.download_decider = Some(decider);
        self
    }

    pub fn with_filters(mut self, filters: Option<CrawlerFilters>) -> Self {
        if let Some(f) = filters {
            self.filters = f;
        }
        self
    }

    pub async fn run(&self) -> CrawlerResponse {
        if let Err(e) = self.config.validate() {
            return CrawlerResponse {
                success: false,
                message: e,
            };
        }
        let response = match &self.crawler_type {
            CrawlerType::ValidateCredentials => {
                crawls::run_validate_credentials_crawler(&self).await
            }
            CrawlerType::DownloadInvoices => crawls::run_download_invoices_crawler(&self).await,
            CrawlerType::DownloadIssuedInvoices => {
                crawls::run_download_issued_invoices_crawler(&self).await
            }
            CrawlerType::DownloadReceivedInvoices => {
                crawls::run_download_received_invoices_crawler(&self).await
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

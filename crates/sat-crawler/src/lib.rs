mod crawls;
mod utils;

#[derive(Debug, Clone)]
pub struct CrawlerResponse {
    pub success: bool,
    pub message: String,
}

pub enum Crawler {
    ValidateCredentials { username: String, password: String },
    DownloadInvoices { username: String, password: String },
}

impl Crawler {
    pub async fn run(&self) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
        match &self {
            Crawler::ValidateCredentials { username, password } => {
                let response =
                    crawls::run_validate_credentials_crawler(username.clone(), password.clone())
                        .await?;
                Ok(response)
            }
            Crawler::DownloadInvoices { username, password } => {
                let response =
                    crawls::run_download_invoices_crawler(username.clone(), password.clone())
                        .await?;
                Ok(response)
            }
        }
    }
}

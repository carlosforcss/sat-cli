use crate::crawls::steps::login::login;
use crate::utils::do_sleep;
use crate::{Crawler, CrawlerResponse};
use futures::StreamExt;

pub async fn run_validate_credentials_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
    crawler
        .logger
        .info("run_validate_credentials Starting crawler...");

    let (browser, mut handler) = crawler.build_browser().await?;
    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    crawler
        .logger
        .info("Browser launched, navingating to login page");

    match login(&browser, &crawler).await {
        Ok(_) => {
            crawler.logger.info("Login succesful");
            return Ok(CrawlerResponse {
                success: true,
                message: "Credentials are valid".to_string(),
            });
        }
        Err(error) => {
            crawler
                .logger
                .info(&format!("Eror during login: {}", error));
            return Ok(CrawlerResponse {
                success: false,
                message: format!("Error during login: {}", error),
            });
        }
    }
}

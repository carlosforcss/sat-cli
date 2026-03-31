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

    let page = login(&browser, &crawler).await?;
    do_sleep(2).await;
    page.wait_for_navigation().await?;

    crawler
        .logger
        .info("CHecking for error message on the page");

    match page.find_element("#msgError").await {
        Ok(error_element) => {
            let error_text = error_element.inner_text().await?;
            let mut message = "Unkown error".to_string();
            if let Some(error_text) = error_text {
                message = error_text;
            }
            Ok(CrawlerResponse {
                success: true,
                message: message,
            })
        }
        Err(_) => {
            println!("Credentials are valid!");
            Ok(CrawlerResponse {
                success: true,
                message: "Credentials are valid".to_string(),
            })
        }
    }
}

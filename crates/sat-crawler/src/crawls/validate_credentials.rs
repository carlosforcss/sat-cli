use crate::crawls::steps::login::login;
use crate::utils::do_sleep;
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use tempfile::tempdir;
use tracing::info;

pub async fn run_validate_credentials_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
    info!("run_validate_credentials Starting crawler...");
    let dir = tempdir()?;

    let mut browser_config_builder = BrowserConfig::builder().user_data_dir(dir.path());

    if crawler.config.options.headless.unwrap_or(true) {
        browser_config_builder = browser_config_builder.with_head().clone();
    }

    if crawler.config.options.sandbox.unwrap_or(false) == false {
        browser_config_builder = browser_config_builder.no_sandbox().clone();
    }

    let browser_config = browser_config_builder.build()?;

    let (browser, mut handler) = Browser::launch(browser_config).await?;

    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    info!("Browser launched, navingating to login page");

    let page = login(
        &browser,
        crawler.config.credentials.username.clone(),
        crawler.config.credentials.password.clone(),
    )
    .await?;
    do_sleep(2).await;
    page.wait_for_navigation().await?;

    info!("CHecking for error message on the page");

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

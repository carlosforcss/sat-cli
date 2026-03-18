use crate::crawls::steps::login::login;
use crate::utils::do_sleep;
use crate::CrawlerResponse;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use std::error::Error;

pub async fn run_download_invoices_crawler(
    username: String,
    password: String,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = login(&browser, username, password).await?;
    do_sleep(10).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Invoices downloaded successfully".to_string(),
    })
}

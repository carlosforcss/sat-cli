use crate::crawls::steps::login::login;
use crate::utils::do_sleep;
use crate::CrawlerResponse;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;

pub async fn run_validate_credentials_crawler(
    username: String,
    password: String,
) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
    println!("Validating credentials...");

    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = login(&browser, username, password).await?;
    do_sleep(2).await;
    page.wait_for_navigation().await?;
    match page.find_element("#msgError").await {
        Ok(error_element) => {
            let error_text = error_element.inner_text().await?;
            let mut message = "Unkown error".to_string();
            if let Some(error_text) = error_text {
                println!("Error: {}", error_text);
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

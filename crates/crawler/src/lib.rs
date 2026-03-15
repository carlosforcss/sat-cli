use std::{env, fs};
use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
use tokio::time::{sleep, Duration};
use twocaptcha::{TwoCaptcha, TwoCaptchaConfig};
use tempfile;
use base64::{engine::general_purpose, Engine as _};

async fn do_sleep(duration: u64) {
    sleep(Duration::from_secs(duration)).await;
}


async fn create_tmp_file(image_base64: &str) -> Result<tempfile::NamedTempFile, Box<dyn std::error::Error>> {
    let file_content = general_purpose::STANDARD.decode(image_base64)?;
    let tmp_file = tempfile::Builder::new()
        .suffix(".jpg")
        .tempfile()?;
    dbg!("Tep file: ", &tmp_file);
    fs::write(&tmp_file, file_content)?;
    Ok(tmp_file)
}


async fn solve_captcha(image_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let twocaptcha_api_key = env::var("TWOCAPTCHA_API_KEY").expect("NO TWOCAPTCHA_API_KEY env variable found");
    let solver = TwoCaptcha::new(twocaptcha_api_key, TwoCaptchaConfig::default());
    let result = solver.normal(
        image_path,
        None,
    ).await?;
    
    if let Some(code) = result.code {
        return Ok(code);
    }
    return Err("NO code found in captcha result".into());
}


#[derive(Debug, Clone)]
pub struct CrawlerResponse {
    pub success: bool,
    pub message: String,
}


pub enum Crawler {
    ValidateCredentials {
        username: String,
        password: String
    }
}


impl Crawler {
    pub async fn run (&self) -> Result<CrawlerResponse, Box<dyn std::error::Error>> {
        match &self {
            Crawler::ValidateCredentials { username, password } => {
                println!("Validating credentials...");
                let (browser, mut handler) =
                    Browser::launch(BrowserConfig::builder().with_head().build()?).await?;
                
                let _ = tokio::spawn(async move {
                    loop {
                        let _ = handler.next().await.unwrap();
                    }
                });

                let page = browser.new_page("https://login.siat.sat.gob.mx/").await?;
                page.wait_for_navigation().await?;
                do_sleep(5).await;
               
                // Set username
                dbg!("Getting element input#rfc");
                let rfc_input = page.find_element("#rfc").await?;
                rfc_input.focus().await?;
                rfc_input.type_str(username.clone()).await?;

                // Set password
                dbg!("Getting elememnt input#password");
                let password_input = page.find_element("input#password").await?;
                password_input.focus().await?;
                password_input.type_str(password.clone()).await?;

                // Getting catpcha image to solve
                dbg!("Getting element #divCaptcha img");
                let img_captcha = page.find_element("#divCaptcha img").await?;
                let img_src_content = img_captcha.attribute("src").await?.unwrap();

                // Getting code fromm twocaptcha
                let img_base64 = img_src_content.replace("data:image/jpeg;base64,", "");
                let img_tmp_file = create_tmp_file(&img_base64).await?;
                let img_tmp_path = img_tmp_file.path().to_str().unwrap();
                let captcha_code = solve_captcha(img_tmp_path).await?;
               
                dbg!("Fullfilling captcha input in page");
                // Fullfilling captcha input
                let captcha_input = page.find_element("input#userCaptcha").await?;
                captcha_input.focus().await?;
                captcha_input.type_str(&captcha_code).await?;

                // Pressing submit button
                dbg!("Submitting survey");
                let submit_button = page.find_element("input#submit").await?;
                submit_button.focus().await?;
                submit_button.click().await?;
                
                do_sleep(1).await;
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
                    },
                    Err(_) => {
                        println!("Credentials are valid!");
                        Ok(CrawlerResponse {
                            success: true,
                            message: "Credentials are valid".to_string(),
                        })
                    }
                }
            }
        }
    }
}


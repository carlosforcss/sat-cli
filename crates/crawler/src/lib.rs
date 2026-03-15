use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
use tokio::time::{sleep, Duration};
use twocaptcha::{TwoCaptcha, TwoCaptchaConfig, CaptchaResult};

async fn do_sleep(duration: u64) {
    sleep(Duration::from_secs(duration)).await;
}


async fn solve_captcha(image_url: &str) -> Result<String, Box<dyn std::error::Error>> { 
    let solver = TwoCaptcha::new("my_api_key".to_string(), TwoCaptchaConfig::default());
    let result = solver.normal(
        "file.jpeg",
        None,
    ).await?;
    
    if let Some(code) = result.code {
        return Ok(code);
    }
    return Err("NO code found in captcha result".into());
}


pub enum Crawler {
    ValidateCredentials {
        username: String,
        password: String
    }
}
impl Crawler {
    pub async fn run (&self) -> Result<(), Box<dyn std::error::Error>> {
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

                do_sleep(5).await;
            }
        }
        Ok(())
    }
}


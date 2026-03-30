use base64::{engine::general_purpose, Engine as _};
use std::{env, fs};
use tempfile;
use tokio::time::{sleep, Duration};
use tracing::info;
use twocaptcha::{TwoCaptcha, TwoCaptchaConfig};

pub async fn do_sleep(duration: u64) {
    sleep(Duration::from_secs(duration)).await;
}

pub async fn create_tmp_file(
    image_base64: &str,
) -> Result<tempfile::NamedTempFile, Box<dyn std::error::Error>> {
    let file_content = general_purpose::STANDARD.decode(image_base64)?;
    let tmp_file = tempfile::Builder::new().suffix(".jpg").tempfile()?;
    info!("Tep file: {:?}", &tmp_file);
    fs::write(&tmp_file, file_content)?;
    Ok(tmp_file)
}

pub async fn solve_captcha(image_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let twocaptcha_api_key =
        env::var("TWOCAPTCHA_API_KEY").expect("NO TWOCAPTCHA_API_KEY env variable found");
    let solver = TwoCaptcha::new(twocaptcha_api_key, TwoCaptchaConfig::default());
    let result = solver.normal(image_path, None).await?;

    if let Some(code) = result.code {
        return Ok(code);
    }
    return Err("NO code found in captcha result".into());
}

pub fn get_download_folder() -> String {
    env::var("DOWNLOAD_FOLDER").unwrap_or_else(|_| "downloads".to_string())
}

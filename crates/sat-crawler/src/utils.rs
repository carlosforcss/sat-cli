use crate::config::CrawlerFilters;
use crate::constants::{
    DEFAULT_DOCUMENTS_FOLDER, DOCUMENTS_ENV_VAR, FILTER_START_YEAR, MX_DATE_FORMAT,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Datelike;
use dirs;
use std::future::Future;
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

pub fn get_download_folder(addition: Option<String>) -> String {
    let home = dirs::home_dir();
    let mut default_folder = DEFAULT_DOCUMENTS_FOLDER.to_string();
    let mut download_path = default_folder.clone();
    if let Some(addition) = addition {
        default_folder.push_str(&addition);
        download_path.push_str(&addition);
    }
    if env::var(DOCUMENTS_ENV_VAR).is_ok() {
        download_path = env::var(DOCUMENTS_ENV_VAR).unwrap();
    } else if let Some(home_path) = home {
        download_path = home_path.join(default_folder).to_str().unwrap().to_string();
    }
    return download_path;
}

pub fn set_mx_date_format(date: chrono::NaiveDate) -> String {
    date.format(MX_DATE_FORMAT).to_string()
}

pub fn get_all_date_filters() -> Vec<(String, String)> {
    let mut filters: Vec<(String, String)> = vec![];
    let range_end = chrono::Utc::now().date_naive();
    let range_start = chrono::NaiveDate::from_ymd_opt(FILTER_START_YEAR, 1, 1).unwrap();
    let mut year = range_start.year();
    while year < range_end.year() {
        filters.push((
            set_mx_date_format(chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap()),
            set_mx_date_format(chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap()),
        ));
        year += 1
    }
    filters.push((
        set_mx_date_format(chrono::NaiveDate::from_ymd_opt(range_end.year(), 1, 1).unwrap()),
        set_mx_date_format(range_end),
    ));
    filters
}

pub fn parse_date(s: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(s, MX_DATE_FORMAT)
        .map_err(|_| format!("Invalid date '{}': expected format dd/mm/YYYY (e.g. 01/01/2024)", s))
}

pub fn apply_date_filter(
    ranges: Vec<(String, String)>,
    filters: &CrawlerFilters,
) -> Vec<(chrono::NaiveDate, chrono::NaiveDate)> {
    let parse = |s: &str| chrono::NaiveDate::parse_from_str(s, MX_DATE_FORMAT).ok();

    ranges
        .into_iter()
        .filter_map(|(start_str, end_str)| {
            let range_start = parse(&start_str)?;
            let range_end = parse(&end_str)?;

            let effective_start = filters
                .start_date
                .map(|fs| fs.max(range_start))
                .unwrap_or(range_start);
            let effective_end = filters
                .end_date
                .map(|fe| fe.min(range_end))
                .unwrap_or(range_end);

            (effective_start <= effective_end).then_some((effective_start, effective_end))
        })
        .collect()
}

pub async fn retry<Fut, T, E, F>(mut f: F, max_retries: u32, mil_delay: u64) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    let delay = Duration::from_millis(mil_delay);
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                println!(
                    "Attempt {} failed: {:?}. Retrying in {} seconds...",
                    attempts,
                    e,
                    delay.as_secs()
                );
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

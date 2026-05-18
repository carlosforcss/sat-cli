use crate::constants::CSF_LOGIN_URL;
use crate::utils::{create_tmp_file, do_sleep, retry, solve_captcha};
use crate::Crawler;
use chromiumoxide::{Browser, Page};

async fn try_to_login_csf(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    let page = browser.new_page(CSF_LOGIN_URL).await?;
    page.wait_for_navigation().await?;
    do_sleep(1).await;
    // Second wait — SAT pages often have late JS init (matches existing CFDI login pattern)
    page.wait_for_navigation().await?;
    do_sleep(1).await;

    crawler.logger.info("CSF login: filling RFC");
    let rfc_input = page.find_element("input[name='Ecom_User_ID']").await?;
    rfc_input.focus().await?;
    rfc_input
        .type_str(crawler.config.credentials.username.clone())
        .await?;

    crawler.logger.info("CSF login: filling password");
    let password_input = page.find_element("input[name='Ecom_Password']").await?;
    password_input.focus().await?;
    password_input
        .type_str(crawler.config.credentials.password.clone())
        .await?;

    crawler.logger.info("CSF login: solving CAPTCHA");
    let img_captcha = page.find_element("#divCaptcha img").await?;
    let img_src_content = img_captcha.attribute("src").await?.unwrap();
    let img_base64 = img_src_content.replace("data:image/jpeg;base64,", "");
    let img_tmp_file = create_tmp_file(&img_base64).await?;
    let img_tmp_path = img_tmp_file.path().to_str().unwrap();
    let captcha_code = solve_captcha(img_tmp_path).await?;

    // Use the same ID-based selector as the CFDI portal login
    crawler.logger.info("CSF login: filling CAPTCHA");
    let captcha_input = page.find_element("input#userCaptcha").await?;
    captcha_input.focus().await?;
    captcha_input.type_str(&captcha_code).await?;

    crawler.logger.info("CSF login: submitting form");
    let submit_button = page.find_element("input#submit").await?;
    submit_button.focus().await?;
    submit_button.click().await?;
    do_sleep(1).await;
    page.wait_for_navigation().await?;

    let page_url = page.url().await?.unwrap_or_default();
    crawler
        .logger
        .info(&format!("CSF login: post-submit URL = {}", page_url));

    // Login succeeded when we land on /nidp/portal — still same domain but no longer the login form
    if !page_url.contains("/nidp/app/login") {
        crawler.logger.info("CSF login successful");
        return Ok(page);
    }

    let err_message = match page.find_element("#msgError").await {
        Ok(el) => el
            .inner_text()
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "Unknown error".to_string()),
        Err(_) => "Unknown error".to_string(),
    };
    crawler
        .logger
        .info(&format!("CSF login failed: {}", &err_message));

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("CSF login failed: {}", err_message),
    )))
}

pub async fn login_csf(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    crawler.logger.info("CSF: attempting login...");
    retry(|| try_to_login_csf(browser, crawler), 3, 500).await
}

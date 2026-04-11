use crate::constants::LOGIN_URL;
use crate::utils::{create_tmp_file, do_sleep, retry, solve_captcha};
use crate::Crawler;
use chromiumoxide::{Browser, Page};

pub async fn try_to_login(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    let page = browser.new_page(LOGIN_URL).await?;
    page.wait_for_navigation().await?;
    do_sleep(1).await;
    page.wait_for_navigation().await?;

    // Set username
    crawler.logger.info("Getting element input#rfc");
    let rfc_input = page.find_element("#rfc").await?;
    rfc_input.focus().await?;
    rfc_input
        .type_str(crawler.config.credentials.username.clone())
        .await?;

    // Set password
    crawler.logger.info("Getting elememnt input#password");
    let password_input = page.find_element("input#password").await?;
    password_input.focus().await?;
    password_input
        .type_str(crawler.config.credentials.password.clone())
        .await?;

    // Getting catpcha image to solve
    crawler.logger.info("Getting element #divCaptcha img");
    let img_captcha = page.find_element("#divCaptcha img").await?;
    let img_src_content = img_captcha.attribute("src").await?.unwrap();

    // Getting code fromm twocaptcha
    let img_base64 = img_src_content.replace("data:image/jpeg;base64,", "");
    let img_tmp_file = create_tmp_file(&img_base64).await?;
    let img_tmp_path = img_tmp_file.path().to_str().unwrap();
    let captcha_code = solve_captcha(img_tmp_path).await?;

    crawler.logger.info("Fullfilling captcha input in page");
    // Fullfilling captcha input
    let captcha_input = page.find_element("input#userCaptcha").await?;
    captcha_input.focus().await?;
    captcha_input.type_str(&captcha_code).await?;

    // Pressing submit button
    crawler.logger.info("Submitting survey");
    let submit_button = page.find_element("input#submit").await?;
    submit_button.focus().await?;
    submit_button.click().await?;
    do_sleep(1).await;
    page.wait_for_navigation().await?;
    let page_url = page.url().await?.unwrap();
    if page_url.contains("portalcfdi.facturaelectronica.sat.gob.mx") {
        crawler.logger.info("Credentials are valid");
        return Ok(page);
    }
    dbg!(page_url);
    let err_message = match page.find_element("#msgError").await {
        Ok(error_element) => {
            let error_text = error_element.inner_text().await;
            let mut message = "Unkown error".to_string();
            if let Ok(Some(error_text)) = error_text {
                message = error_text;
            }
            crawler
                .logger
                .info(&format!("Login failed with error: {}", &message));
            message
        }
        Err(_) => {
            crawler.logger.info("Login failedd with unkown message");
            "Unkown error".to_string()
        }
    };

    return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Login failed with message: ".to_string() + &err_message,
    )));
}

pub async fn try_to_login_with_fiel(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    let crt_path = crawler
        .config
        .credentials
        .crt_path
        .as_deref()
        .ok_or("FIEL login requires a certificate path")?;
    let key_path = crawler
        .config
        .credentials
        .key_path
        .as_deref()
        .ok_or("FIEL login requires a key path")?;

    let page = browser.new_page(LOGIN_URL).await?;
    page.wait_for_navigation().await?;

    crawler.logger.info("Switching to FIEL login");
    page.find_element("#buttonFiel").await?.click().await?;
    page.wait_for_navigation().await?;

    crawler.logger.info("Uploading certificate (.cer)");
    let crt_input = page.find_element("#fileCertificate").await?;
    page.execute(
        chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams::builder()
            .file(crt_path)
            .backend_node_id(crt_input.backend_node_id)
            .build()?,
    )
    .await?;
    crawler.logger.info("Uploading private key (.key)");

    let key_input = page.find_element("#filePrivateKey").await?;
    page.execute(
        chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams::builder()
            .file(key_path)
            .backend_node_id(key_input.backend_node_id)
            .build()?,
    )
    .await?;

    crawler.logger.info("Entering private key password");
    let password_input = page.find_element("#privateKeyPassword").await?;
    password_input.focus().await?;
    password_input
        .type_str(crawler.config.credentials.password.clone())
        .await?;

    crawler.logger.info("Submitting FIEL login");
    page.find_element("#submit").await?.click().await?;
    do_sleep(5).await;
    page.wait_for_navigation().await?;

    let page_url = page.url().await?.unwrap();
    if page_url.contains("portalcfdi.facturaelectronica.sat.gob.mx") {
        crawler.logger.info("FIEL login successful");
        return Ok(page);
    }

    let err_message = match page.find_element("#divError").await {
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
        .info(&format!("FIEL login failed: {}", &err_message));
    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("FIEL login failed: {}", err_message),
    )))
}

pub async fn login(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    crawler.logger.info("Trying to login...");
    match crawler.config.credentials.login_type {
        crate::config::LoginType::Ciec => {
            retry(|| try_to_login(browser, crawler), 3, 500 as u64).await
        }
        crate::config::LoginType::Fiel => {
            retry(|| try_to_login_with_fiel(browser, crawler), 3, 500).await
        }
    }
}

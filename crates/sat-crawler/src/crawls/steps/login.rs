use crate::utils::do_sleep;
use crate::utils::{create_tmp_file, solve_captcha};
use crate::Crawler;
use chromiumoxide::{Browser, Page};

pub const LOGIN_URL: &str = "https://portalcfdi.facturaelectronica.sat.gob.mx/";

pub async fn login(
    browser: &Browser,
    crawler: &Crawler,
) -> Result<Page, Box<dyn std::error::Error>> {
    let page = browser.new_page(LOGIN_URL).await?;
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
    page.wait_for_navigation().await?;

    Ok(page)
}

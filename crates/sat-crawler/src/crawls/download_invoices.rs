use crate::crawls::steps::login::login;
use crate::utils::do_sleep;
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::error::Error;
use tempfile::tempdir;

const ISSUED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx";

async fn filter_by_date(crawler: &Crawler, page: &Page) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Navigating to issued invoices page");
    page.goto(ISSUED_INVOICES_URL).await?;
    page.wait_for_navigation().await?;
    do_sleep(5).await;

    crawler.logger.info("Selecting date filter option");
    // Use JS click to bypass Bootstrap's absolute-positioned radio hit-test offset
    page.evaluate(
        "document.querySelector('#ctl00_MainContent_RdoFechas').click()",
    )
    .await?;
    do_sleep(5).await;

    crawler.logger.info("Getting start date input");
    let start_date_input = page
        .find_element("#ctl00_MainContent_CldFechaInicial2_Calendario_text")
        .await?;
    start_date_input.click().await?;
    page.evaluate(r#"
        document.querySelector('#ctl00_MainContent_CldFechaInicial2_Calendario_text').value = '01/01/2025'
    "#).await?;
    for (selector, value) in [
        ("#ctl00_MainContent_CldFechaInicial2_DdlHora", "0"),
        ("#ctl00_MainContent_CldFechaInicial2_DdlMinuto", "0"),
        ("#ctl00_MainContent_CldFechaInicial2_DdlSegundo", "0"),
    ] {
        page.evaluate_expression(&format!(
            "document.querySelector('{selector}').value = '{value}'"
        ))
        .await?;
    }
    crawler.logger.info("Typing start date");
    start_date_input.type_str("01/01/2025").await?;

    crawler.logger.info("Getting end date input");
    let end_date_input = page
        .find_element("#ctl00_MainContent_CldFechaFinal2_Calendario_text")
        .await?;
    end_date_input.click().await?;
    page.evaluate(r#"
        document.querySelector('#ctl00_MainContent_CldFechaFinal2_Calendario_text').value = '31/12/2025'
    "#).await?;
    for (selector, value) in [
        ("#ctl00_MainContent_CldFechaFinal2_DdlHora", "23"),
        ("#ctl00_MainContent_CldFechaFinal2_DdlMinuto", "59"),
        ("#ctl00_MainContent_CldFechaFinal2_DdlSegundo", "59"),
    ] {
        page.evaluate_expression(&format!(
            "document.querySelector('{selector}').value = '{value}'"
        ))
        .await?;
    }
    crawler.logger.info("Clicking search button");
    page.evaluate(
        "document.querySelector('#ctl00_MainContent_BtnBusqueda').click()",
    )
    .await?;
    do_sleep(10).await;
    Ok(())
}

pub async fn run_download_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    let dir = tempdir()?;

    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .user_data_dir(dir.path())
            .build()?,
    )
    .await?;

    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let page = login(&browser, &crawler).await?;
    do_sleep(1).await;
    filter_by_date(&crawler, &page).await?;
    do_sleep(10).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Invoices downloaded successfully".to_string(),
    })
}

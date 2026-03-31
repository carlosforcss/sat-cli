use crate::crawls::steps::login::login;
use crate::utils::{do_sleep, get_all_date_filters, get_download_folder};
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::cdp::browser_protocol::browser::{
    SetDownloadBehaviorBehavior, SetDownloadBehaviorParams,
};
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::error::Error;
use tempfile::tempdir;

const ISSUED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx";

async fn filter_by_date(
    crawler: &Crawler,
    page: &Page,
    start_date: String,
    end_date: String,
) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Navigating to issued invoices page");
    page.goto(ISSUED_INVOICES_URL).await?;
    page.wait_for_navigation().await?;
    do_sleep(5).await;

    crawler.logger.info("Selecting date filter option");
    // Use JS click to bypass Bootstrap's absolute-positioned radio hit-test offset
    page.evaluate("document.querySelector('#ctl00_MainContent_RdoFechas').click()")
        .await?;
    do_sleep(5).await;

    crawler.logger.info("Getting start date input");
    let start_date_input = page
        .find_element("#ctl00_MainContent_CldFechaInicial2_Calendario_text")
        .await?;
    start_date_input.click().await?;
    page.evaluate(format!(
        r#"
        document.querySelector('#ctl00_MainContent_CldFechaInicial2_Calendario_text').value = '{}'
    "#,
        start_date
    ))
    .await?;
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
    page.evaluate(format!(
        r#"
        document.querySelector('#ctl00_MainContent_CldFechaFinal2_Calendario_text').value = '{}'
    "#,
        end_date
    ))
    .await?;
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
    page.evaluate("document.querySelector('#ctl00_MainContent_BtnBusqueda').click()")
        .await?;
    do_sleep(10).await;
    Ok(())
}

pub async fn download_current_page_invoices(
    crawler: &Crawler,
    page: &Page,
) -> Result<(), Box<dyn Error>> {
    crawler
        .logger
        .info("Downloading invoices metadata from current page");
    let invoice_rows = page
        .find_elements("#ctl00_MainContent_tblResult tbody tr")
        .await?;
    crawler
        .logger
        .info(&format!("Found {} rows on page", invoice_rows.len()));
    let download_path = get_download_folder();
    for row in invoice_rows.into_iter() {
        let cells = row.find_elements(":scope > td").await?;
        if cells.len() < 13 {
            continue; // skip header row (<th> only) and any short footer/pagination rows
        }
        let uuid = cells[0]
            .find_element("input.ListaFolios")
            .await?
            .attribute("value")
            .await?
            .unwrap_or_default();
        let fiscal_id = cells[1]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let issuer_tax_id = cells[2]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let issuer_name = cells[3]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let receiver_tax_id = cells[4]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let receiver_name = cells[5]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let issued_at = cells[6]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let certified_at = cells[7]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let _pac = cells[8]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let total = cells[9]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let invoice_type = cells[10]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        let invoice_status = cells[12]
            .find_element("span")
            .await?
            .inner_text()
            .await?
            .unwrap_or_default();
        crawler.logger.info(&format!(
            "Invoice {} | {} | {} | {} -> {} | {} | {} | {} | {} | {} | {}",
            uuid,
            fiscal_id,
            issuer_tax_id,
            issuer_name,
            receiver_tax_id,
            receiver_name,
            issued_at,
            certified_at,
            total,
            invoice_type,
            invoice_status
        ));
        cells[0].find_element("#BtnDescarga").await?.click().await?;
        do_sleep(1).await;
        cells[0].find_element("#BtnRI").await?.click().await?;
        do_sleep(1).await;
        crawler
            .logger
            .info(&format!("Downloaded {} to {}", uuid, download_path));
    }
    Ok(())
}

pub async fn run_download_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler.logger.info("Setting up browser configuration");
    let dir = tempdir()?;
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .user_data_dir(dir.path())
            .build()?,
    )
    .await?;

    crawler.logger.info("Starting browser event handler");
    let _ = tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    crawler
        .logger
        .info("Setting up download behavior for browser");
    browser
        .execute(SetDownloadBehaviorParams {
            behavior: SetDownloadBehaviorBehavior::Allow,
            download_path: Some(get_download_folder()),
            browser_context_id: None,
            events_enabled: Some(true),
        })
        .await?;

    crawler.logger.info("Starting download invoices crawler");
    let page = login(&browser, &crawler).await?;
    do_sleep(1).await;
    let ranges = get_all_date_filters();
    for (range_start, range_end) in ranges {
        crawler.logger.info(&format!(
            "Processing date range: {} - {}",
            range_start, range_end
        ));
        filter_by_date(&crawler, &page, range_start, range_end).await?;
        download_current_page_invoices(&crawler, &page).await?;
    }
    do_sleep(10).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Invoices downloaded successfully".to_string(),
    })
}

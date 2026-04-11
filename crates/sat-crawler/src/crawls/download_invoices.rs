use crate::constants::{ISSUED_AT_FORMAT, ISSUED_INVOICES_URL, RECEIVED_INVOICES_URL};
use crate::crawls::steps::login::login;
use crate::utils::{apply_date_filter, do_sleep, get_all_date_filters, get_download_folder, retry, set_mx_date_format};
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::cdp::browser_protocol::browser::{
    SetDownloadBehaviorBehavior, SetDownloadBehaviorParams,
};
use chromiumoxide::element::Element;
use chromiumoxide::Page;
use chrono::Datelike;
use futures::StreamExt;
use std::error::Error;
use std::path::Path;

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
    do_sleep(1).await;

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
    do_sleep(5).await;
    Ok(())
}

fn should_download_invoice(uuid: &str, issued_at: &str, download_path: &str) -> bool {
    let xml = Path::new(download_path).join(format!("{}.xml", uuid));
    let pdf = Path::new(download_path).join(format!("{}.pdf", uuid));
    let both_exist = xml.exists() && pdf.exists();

    if !both_exist {
        return true;
    }

    let now = chrono::Utc::now();
    let is_current_period = chrono::NaiveDateTime::parse_from_str(issued_at, ISSUED_AT_FORMAT)
        .map(|dt| dt.year() == now.year() && dt.month() == now.month())
        .unwrap_or(false);

    is_current_period
}

async fn process_invoice_row(
    crawler: &Crawler,
    row: Element,
    download_path: &str,
) -> Result<(), Box<dyn Error>> {
    let row_style = row.attribute("style").await?.unwrap_or_default();
    if row_style.contains("display: none") || row_style.contains("display:none") {
        return Ok(());
    }

    let cells = row.find_elements(":scope > td").await?;
    if cells.len() < 13 {
        return Ok(());
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

    if !should_download_invoice(&uuid, &issued_at, download_path) {
        crawler
            .logger
            .info(&format!("Skipping invoice {} (already exists)", uuid));
        return Ok(());
    }

    match cells[0].find_element("#BtnDescarga").await {
        Ok(download_button) => {
            download_button.click().await?;
            do_sleep(1).await;
        }
        Err(_) => {
            crawler
                .logger
                .info(&format!("Download button not found for invoice {}", uuid));
        }
    }
    match cells[0].find_element("#BtnRI").await {
        Ok(download_button) => {
            download_button.click().await?;
        }
        Err(_) => {
            crawler.logger.info(&format!(
                "RI download button not found for invoice {}",
                uuid
            ));
        }
    }
    crawler
        .logger
        .info(&format!("Downloaded {} to {}", uuid, download_path));
    Ok(())
}

pub async fn download_current_page_invoices(
    crawler: &Crawler,
    page: &Page,
) -> Result<(), Box<dyn Error>> {
    crawler
        .logger
        .info("Downloading invoices metadata from current page");

    let num_pages: u64 = page
        .evaluate(
            "document.querySelectorAll('#ctl00_MainContent_pageNavPosition li[id^=\"pg\"]').length",
        )
        .await?
        .into_value()
        .unwrap_or(0);
    let num_pages = num_pages.max(1);
    crawler
        .logger
        .info(&format!("Found {} pages with this filter", &num_pages));
    let download_path = get_download_folder(Some(crawler.config.credentials.username.clone()));

    for page_num in 1..=num_pages {
        if num_pages > 1 {
            crawler.logger.info(&format!(
                "Navigating to pagination page {}/{}",
                page_num, num_pages
            ));
            page.evaluate(format!("pager.showPage({})", page_num))
                .await?;
            do_sleep(1).await;
        }

        let invoice_rows = page
            .find_elements("#ctl00_MainContent_tblResult tbody tr")
            .await?;
        crawler.logger.info(&format!(
            "Found {} rows on page {}",
            invoice_rows.len(),
            page_num
        ));

        for row in invoice_rows.into_iter() {
            if let Err(e) = process_invoice_row(crawler, row, &download_path).await {
                crawler
                    .logger
                    .info(&format!("Failed to process invoice row: {}", e));
            }
        }
    }
    Ok(())
}

async fn download_issued_invoices(crawler: &Crawler, page: &Page) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Downloading issued invoices");
    do_sleep(1).await;
    let ranges = apply_date_filter(get_all_date_filters(), &crawler.config.filters);
    for (range_start, range_end) in ranges {
        crawler.logger.info(&format!(
            "Processing date range: {} - {}",
            set_mx_date_format(range_start), set_mx_date_format(range_end)
        ));
        retry(
            || filter_by_date(&crawler, &page, set_mx_date_format(range_start), set_mx_date_format(range_end)),
            3,
            500,
        )
        .await?;
        retry(|| download_current_page_invoices(&crawler, &page), 3, 500).await?;
        do_sleep(1).await;
    }
    Ok(())
}

async fn filter_by_year_month(
    crawler: &Crawler,
    page: &Page,
    year: u32,
    month: u32,
) -> Result<(), Box<dyn Error>> {
    crawler.logger.info(&format!(
        "Navigating to received invoices page for {}/{}",
        month, year
    ));

    // Click RdoFechas to trigger the UpdatePanel that renders the year/month selectors
    crawler
        .logger
        .info("Selecting date filter (Fecha de Emisión)");
    page.evaluate("document.querySelector('#ctl00_MainContent_RdoFechas').click()")
        .await?;
    do_sleep(1).await;

    crawler.logger.info("Setting year filter");
    page.evaluate(format!(
        "document.querySelector('#DdlAnio').value = '{}'; ValidateYear();",
        year
    ))
    .await?;

    crawler.logger.info("Setting month filter");
    page.evaluate(format!(
        "document.querySelector('#ctl00_MainContent_CldFecha_DdlMes').value = '{}'; asignaDia();",
        month
    ))
    .await?;

    crawler.logger.info("Clicking search button");
    page.evaluate("document.querySelector('#ctl00_MainContent_BtnBusqueda').click()")
        .await?;
    do_sleep(1).await;
    Ok(())
}

async fn download_received_invoices(crawler: &Crawler, page: &Page) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Downloading received invoices");
    do_sleep(1).await;
    let ranges = apply_date_filter(get_all_date_filters(), &crawler.config.filters);

    page.wait_for_navigation().await?;
    do_sleep(1).await;
    for (start_parsed, end_parsed) in ranges {
        let year = start_parsed.year() as u32;
        let start_month = start_parsed.month();
        let end_month = end_parsed.month();

        page.goto(RECEIVED_INVOICES_URL).await?;

        for month in start_month..=end_month {
            crawler
                .logger
                .info(&format!("Processing received invoices {}/{}", month, year));
            match retry(|| filter_by_year_month(crawler, page, year, month), 3, 500).await {
                Ok(_) => {}
                Err(e) => {
                    crawler.logger.info(&format!(
                        "Failed to filter receivedi nvoices for {}/{}: {}",
                        month, year, e
                    ));
                    continue;
                }
            };
            match retry(|| download_current_page_invoices(crawler, page), 3, 500).await {
                Ok(_) => {}
                Err(e) => {
                    crawler.logger.info(&format!(
                        "Failed to download received invoices for {}/{}: {}",
                        month, year, e
                    ));
                    continue;
                }
            };
        }
        do_sleep(10).await;
    }
    do_sleep(10).await;
    Ok(())
}

pub async fn run_download_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler.logger.info("Setting up browser configuration");

    let (browser, mut handler) = crawler.build_browser().await?;

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
            download_path: Some(get_download_folder(Some(
                crawler.config.credentials.username.clone(),
            ))),
            browser_context_id: None,
            events_enabled: Some(true),
        })
        .await?;
    let page = login(&browser, &crawler).await?;
    download_received_invoices(&crawler, &page).await?;
    download_issued_invoices(&crawler, &page).await?;
    crawler.logger.info("Starting download invoices crawler");
    // Delay to ensure all download are completed before closing the browser
    crawler.logger.info("Waiting for download to complete...");
    do_sleep(120).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Invoices downloaded successfully".to_string(),
    })
}

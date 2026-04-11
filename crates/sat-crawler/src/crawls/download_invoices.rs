use crate::constants::{ISSUED_AT_FORMAT, ISSUED_INVOICES_URL, RECEIVED_INVOICES_URL};
use crate::crawls::steps::login::login;
use crate::utils::{
    apply_date_filter, do_sleep, get_all_date_filters, get_download_folder, retry,
    set_mx_date_format,
};
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

async fn set_date_field(
    page: &Page,
    text_selector: &str,
    value: &str,
    time_selectors: &[(&str, &str)],
) -> Result<(), Box<dyn Error>> {
    page.evaluate(format!(
        "{{ const e = document.querySelector('{text_selector}'); e.click(); e.value = '{value}'; }}"
    ))
    .await?;
    for (selector, v) in time_selectors {
        page.evaluate_expression(&format!(
            "document.querySelector('{selector}').value = '{v}'"
        ))
        .await?;
    }
    Ok(())
}

async fn cell_span_text(cell: &Element) -> String {
    match cell.find_element("span").await {
        Ok(el) => el.inner_text().await.ok().flatten().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

async fn click_if_present(cell: &Element, selector: &str) -> Result<bool, Box<dyn Error>> {
    match cell.find_element(selector).await {
        Ok(btn) => {
            btn.click().await?;
            Ok(true)
        }
        Err(_) => Ok(false),
    }
}

async fn filter_by_date(
    page: &Page,
    start_date: String,
    end_date: String,
) -> Result<(), Box<dyn Error>> {
    page.goto(ISSUED_INVOICES_URL).await?;
    page.wait_for_navigation().await?;
    do_sleep(5).await;

    // Use JS click to bypass Bootstrap's absolute-positioned radio hit-test offset
    page.evaluate("document.querySelector('#ctl00_MainContent_RdoFechas').click()")
        .await?;
    do_sleep(1).await;

    set_date_field(
        page,
        "#ctl00_MainContent_CldFechaInicial2_Calendario_text",
        &start_date,
        &[
            ("#ctl00_MainContent_CldFechaInicial2_DdlHora", "0"),
            ("#ctl00_MainContent_CldFechaInicial2_DdlMinuto", "0"),
            ("#ctl00_MainContent_CldFechaInicial2_DdlSegundo", "0"),
        ],
    )
    .await?;
    set_date_field(
        page,
        "#ctl00_MainContent_CldFechaFinal2_Calendario_text",
        &end_date,
        &[
            ("#ctl00_MainContent_CldFechaFinal2_DdlHora", "23"),
            ("#ctl00_MainContent_CldFechaFinal2_DdlMinuto", "59"),
            ("#ctl00_MainContent_CldFechaFinal2_DdlSegundo", "59"),
        ],
    )
    .await?;
    page.evaluate("document.querySelector('#ctl00_MainContent_BtnBusqueda').click()")
        .await?;
    do_sleep(5).await;
    Ok(())
}

fn should_download_invoice(uuid: &str, issued_at: &str, download_path: &str) -> bool {
    let xml = Path::new(download_path).join(format!("{}.xml", uuid));
    let pdf = Path::new(download_path).join(format!("{}.pdf", uuid));
    if !xml.exists() || !pdf.exists() {
        return true;
    }
    let now = chrono::Utc::now();
    chrono::NaiveDateTime::parse_from_str(issued_at, ISSUED_AT_FORMAT)
        .map(|dt| dt.year() == now.year() && dt.month() == now.month())
        .unwrap_or(false)
}

async fn process_invoice_row(
    crawler: &Crawler,
    row: Element,
    download_path: &str,
) -> Result<(), Box<dyn Error>> {
    let row_style = row
        .attribute("style")
        .await
        .map_err(|e| format!("row attribute 'style': {e}"))?
        .unwrap_or_default();
    if row_style.contains("display: none") || row_style.contains("display:none") {
        return Ok(());
    }

    let cells = row
        .find_elements(":scope > td")
        .await
        .map_err(|e| format!("row find_elements ':scope > td': {e}"))?;
    if cells.len() < 13 {
        return Ok(());
    }

    let uuid = cells[0]
        .find_element("input.ListaFolios")
        .await
        .map_err(|e| format!("cells[0] find_element 'input.ListaFolios': {e}"))?
        .attribute("value")
        .await
        .map_err(|e| format!("'input.ListaFolios' attribute 'value': {e}"))?
        .unwrap_or_default();
    let fiscal_id = cell_span_text(&cells[1]).await;
    let issuer_tax_id = cell_span_text(&cells[2]).await;
    let issuer_name = cell_span_text(&cells[3]).await;
    let receiver_tax_id = cell_span_text(&cells[4]).await;
    let receiver_name = cell_span_text(&cells[5]).await;
    let issued_at = cell_span_text(&cells[6]).await;
    let certified_at = cell_span_text(&cells[7]).await;
    let total = cell_span_text(&cells[9]).await;
    let invoice_type = cell_span_text(&cells[10]).await;
    let invoice_status = cell_span_text(&cells[12]).await;
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

    if !click_if_present(&cells[0], "#BtnDescarga")
        .await
        .map_err(|e| format!("clicking '#BtnDescarga' for invoice {uuid}: {e}"))?
    {
        crawler.logger.info(&format!(
            "Download button '#BtnDescarga' not found for invoice {}",
            uuid
        ));
    } else {
        do_sleep(1).await;
    }
    if !click_if_present(&cells[0], "#BtnRI")
        .await
        .map_err(|e| format!("clicking '#BtnRI' for invoice {uuid}: {e}"))?
    {
        crawler.logger.info(&format!(
            "RI download button '#BtnRI' not found for invoice {}",
            uuid
        ));
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
            set_mx_date_format(range_start),
            set_mx_date_format(range_end)
        ));
        retry(
            || {
                filter_by_date(
                    &page,
                    set_mx_date_format(range_start),
                    set_mx_date_format(range_end),
                )
            },
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

    // Use JS click to bypass Bootstrap's absolute-positioned radio hit-test offset
    page.evaluate("document.querySelector('#ctl00_MainContent_RdoFechas').click()")
        .await?;
    do_sleep(1).await;

    page.evaluate(format!(
        "document.querySelector('#DdlAnio').value = '{}'; ValidateYear();",
        year
    ))
    .await?;

    page.evaluate(format!(
        "document.querySelector('#ctl00_MainContent_CldFecha_DdlMes').value = '{}'; asignaDia();",
        month
    ))
    .await?;

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
                        "Failed to filter received invoices for {}/{}: {}",
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

async fn setup_browser_and_login(
    crawler: &Crawler,
) -> Result<(chromiumoxide::Browser, Page), Box<dyn Error>> {
    let (browser, mut handler) = crawler.build_browser().await?;
    tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });
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
    let page = login(&browser, crawler).await?;
    Ok((browser, page))
}

pub async fn run_download_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler.logger.info("Starting download invoices crawler");
    let (_browser, page) = setup_browser_and_login(crawler).await?;
    download_received_invoices(crawler, &page).await?;
    download_issued_invoices(crawler, &page).await?;
    // Wait for all downloads to complete before closing the browser
    crawler.logger.info("Waiting for downloads to complete...");
    do_sleep(120).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Invoices downloaded successfully".to_string(),
    })
}

pub async fn run_download_issued_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler
        .logger
        .info("Starting download issued invoices crawler");
    let (_browser, page) = setup_browser_and_login(crawler).await?;
    download_issued_invoices(crawler, &page).await?;
    // Wait for all downloads to complete before closing the browser
    crawler.logger.info("Waiting for downloads to complete...");
    do_sleep(120).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Issued invoices downloaded successfully".to_string(),
    })
}

pub async fn run_download_received_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler
        .logger
        .info("Starting download received invoices crawler");
    let (_browser, page) = setup_browser_and_login(crawler).await?;
    download_received_invoices(crawler, &page).await?;
    // Wait for all downloads to complete before closing the browser
    crawler.logger.info("Waiting for downloads to complete...");
    do_sleep(120).await;
    Ok(CrawlerResponse {
        success: true,
        message: "Received invoices downloaded successfully".to_string(),
    })
}

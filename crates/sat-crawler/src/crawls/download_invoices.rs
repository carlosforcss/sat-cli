use crate::constants::{
    ISSUED_INVOICES_URL, RECEIVED_INVOICES_URL, RECOVER_CFDI_URL, RECOVER_RI_TOKEN_URL,
    SAT_PORTAL_BASE_URL, VALIDATE_DOWNLOAD_URL,
};
use crate::crawls::steps::login::login;
use crate::events::{Invoice, InvoiceEvent};
use crate::utils::{
    apply_date_filter, do_sleep, get_all_date_filters, get_download_folder, retry,
    set_mx_date_format,
};
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use chromiumoxide::element::Element;
use chromiumoxide::Page;
use chrono::Datelike;
use futures::StreamExt;
use std::error::Error;

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

fn extract_xml_token(onclick: &str) -> Option<String> {
    let prefix = "RecuperaCfdi.aspx?Datos=";
    let start = onclick.find(prefix)? + prefix.len();
    let rest = &onclick[start..];
    Some(rest[..rest.find('\'')?].to_string())
}

fn extract_pdf_token(onclick: &str) -> Option<String> {
    let start = onclick.find('\'')? + 1;
    let rest = &onclick[start..];
    Some(rest[..rest.find('\'')?].to_string())
}

fn build_http_client(cookies: Vec<Cookie>) -> Result<reqwest::Client, Box<dyn Error>> {
    let cookie_header = cookies
        .iter()
        .map(|c| format!("{}={}", c.name, c.value))
        .collect::<Vec<_>>()
        .join("; ");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&cookie_header)?,
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        ),
    );
    // SAT portal serves an incomplete certificate chain; the browser handles it
    // via AIA but reqwest/OpenSSL cannot resolve the missing intermediate.
    Ok(reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers)
        .build()?)
}

async fn validate_download(http_client: &reqwest::Client) -> Result<(), Box<dyn Error>> {
    let resp = http_client
        .post(VALIDATE_DOWNLOAD_URL)
        .json(&serde_json::json!({"id": 1}))
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(format!("validate_download HTTP {}: {}", status, text).into());
    }
    Ok(())
}

async fn download_xml(client: &reqwest::Client, token: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = client
        .get(format!("{}?Datos={}", RECOVER_CFDI_URL, token))
        .send()
        .await?;
    let status = resp.status();
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(format!("XML download HTTP {}: {}", status, body).into());
    }
    Ok(bytes.to_vec())
}

async fn download_pdf(client: &reqwest::Client, token: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let resp = client
        .post(RECOVER_RI_TOKEN_URL)
        .json(&serde_json::json!({"datos": token}))
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Err(format!("PDF token exchange HTTP {}: {}", status, text).into());
    }
    let body: serde_json::Value = serde_json::from_str(&text)?;
    // The server returns the full relative URL in "d", e.g.
    // "RepresentacionImpresa.aspx?Datos=<encoded_token>"
    let rel_url = body
        .get("d")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("PDF token response missing 'd': {}", text))?;
    let url = format!("{}/{}", SAT_PORTAL_BASE_URL, rel_url);

    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let is_html = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);
    let bytes = resp.bytes().await?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(format!("PDF download HTTP {}: {}", status, body).into());
    }
    if is_html {
        let preview = String::from_utf8_lossy(&bytes);
        return Err(format!(
            "PDF download returned HTML error page: {}",
            &preview[..preview.len().min(300)]
        )
        .into());
    }
    Ok(bytes.to_vec())
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

async fn process_invoice_row(
    crawler: &Crawler,
    row: Element,
    http_client: &reqwest::Client,
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
    let invoice = Invoice {
        uuid: uuid.clone(),
        fiscal_id,
        issuer_tax_id,
        issuer_name,
        receiver_tax_id,
        receiver_name,
        issued_at: issued_at.clone(),
        certified_at,
        total,
        invoice_type,
        invoice_status,
    };
    crawler.logger.info(&format!(
        "Invoice {} | {} | {} | {} -> {} | {} | {} | {} | {} | {} | {}",
        invoice.uuid,
        invoice.fiscal_id,
        invoice.issuer_tax_id,
        invoice.issuer_name,
        invoice.receiver_tax_id,
        invoice.receiver_name,
        invoice.issued_at,
        invoice.certified_at,
        invoice.total,
        invoice.invoice_type,
        invoice.invoice_status
    ));

    let should_download = match &crawler.download_decider {
        Some(decider) => {
            decider
                .should_download_invoice(&invoice, download_path)
                .await
        }
        None => true,
    };
    if !should_download {
        crawler
            .logger
            .info(&format!("Skipping invoice {} (already exists)", uuid));
        if let Some(handler) = &crawler.event_handler {
            handler
                .on_invoice_event(InvoiceEvent::Skipped { invoice })
                .await;
        }
        return Ok(());
    }

    let xml_token = match cells[0].find_element("#BtnDescarga").await {
        Ok(btn) => btn
            .attribute("onclick")
            .await
            .ok()
            .flatten()
            .and_then(|oc| extract_xml_token(&oc)),
        Err(_) => None,
    };
    let pdf_token = match cells[0].find_element("#BtnRI").await {
        Ok(btn) => btn
            .attribute("onclick")
            .await
            .ok()
            .flatten()
            .and_then(|oc| extract_pdf_token(&oc)),
        Err(_) => None,
    };

    let (xml_result, pdf_result) = tokio::join!(
        async {
            match xml_token {
                Some(token) => Some(download_xml(http_client, &token).await),
                None => None,
            }
        },
        async {
            match pdf_token {
                Some(token) => Some(download_pdf(http_client, &token).await),
                None => None,
            }
        },
    );

    match xml_result {
        Some(Ok(content)) => {
            if let Some(handler) = &crawler.event_handler {
                handler
                    .on_invoice_event(InvoiceEvent::XmlDownloaded {
                        invoice: invoice.clone(),
                        content,
                    })
                    .await;
            }
        }
        Some(Err(e)) => crawler
            .logger
            .error(&format!("XML download failed for {}: {}", uuid, e)),
        None => crawler
            .logger
            .error(&format!("No XML token found for {}", uuid)),
    }

    match pdf_result {
        Some(Ok(content)) => {
            if let Some(handler) = &crawler.event_handler {
                handler
                    .on_invoice_event(InvoiceEvent::PdfDownloaded {
                        invoice: invoice.clone(),
                        content,
                    })
                    .await;
            }
        }
        Some(Err(e)) => crawler
            .logger
            .error(&format!("PDF download failed for {}: {}", uuid, e)),
        None => crawler
            .logger
            .error(&format!("No PDF token found for {}", uuid)),
    }

    Ok(())
}

pub async fn download_current_page_invoices(
    crawler: &Crawler,
    page: &Page,
    http_client: &reqwest::Client,
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
            if let Err(e) = process_invoice_row(crawler, row, http_client, &download_path).await {
                crawler
                    .logger
                    .error(&format!("Failed to process invoice row: {}", e));
            }
        }
    }
    Ok(())
}

async fn download_issued_invoices(
    crawler: &Crawler,
    page: &Page,
    http_client: &reqwest::Client,
) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Downloading issued invoices");
    validate_download(http_client).await?;
    let ranges = apply_date_filter(get_all_date_filters(), &crawler.filters);
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
        retry(
            || download_current_page_invoices(&crawler, &page, http_client),
            3,
            500,
        )
        .await?;
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

async fn download_received_invoices(
    crawler: &Crawler,
    page: &Page,
    http_client: &reqwest::Client,
) -> Result<(), Box<dyn Error>> {
    crawler.logger.info("Downloading received invoices");
    validate_download(http_client).await?;
    let ranges = apply_date_filter(get_all_date_filters(), &crawler.filters);

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
                    crawler.logger.error(&format!(
                        "Failed to filter received invoices for {}/{}: {}",
                        month, year, e
                    ));
                    continue;
                }
            };
            match retry(
                || download_current_page_invoices(crawler, page, http_client),
                3,
                500,
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    crawler.logger.error(&format!(
                        "Failed to download received invoices for {}/{}: {}",
                        month, year, e
                    ));
                    continue;
                }
            };
        }
    }
    Ok(())
}

async fn setup_browser_and_login(
    crawler: &Crawler,
) -> Result<(chromiumoxide::Browser, Page, reqwest::Client), Box<dyn Error>> {
    let (browser, mut handler) = crawler.build_browser().await?;
    tokio::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });
    let page = login(&browser, crawler).await?;
    let cookies = page.get_cookies().await?;
    let http_client = build_http_client(cookies)?;
    Ok((browser, page, http_client))
}

pub async fn run_download_invoices_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    crawler.logger.info("Starting download invoices crawler");
    let (_browser, page, http_client) = setup_browser_and_login(crawler).await?;
    download_received_invoices(crawler, &page, &http_client).await?;
    download_issued_invoices(crawler, &page, &http_client).await?;
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
    let (_browser, page, http_client) = setup_browser_and_login(crawler).await?;
    download_issued_invoices(crawler, &page, &http_client).await?;
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
    let (_browser, page, http_client) = setup_browser_and_login(crawler).await?;
    download_received_invoices(crawler, &page, &http_client).await?;
    Ok(CrawlerResponse {
        success: true,
        message: "Received invoices downloaded successfully".to_string(),
    })
}

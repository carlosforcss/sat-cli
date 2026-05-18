use crate::constants::{CSF_CERTIFICATE_URL, CSF_PDF_URL, CSF_SSO_ENTRY_URL};
use crate::crawls::steps::login_csf::login_csf;
use crate::events::CsfEvent;
use crate::utils::{build_http_client, do_sleep, get_download_folder};
use crate::{Crawler, CrawlerResponse};
use chromiumoxide::Page;
use futures::StreamExt;
use std::error::Error;

/// Push SAML forms on the current page until none remain or the target domain is reached.
async fn push_saml_forms(page: &Page, target_domain: &str, label: &str, crawler: &Crawler) {
    for i in 0..3 {
        let url = page.url().await.ok().flatten().unwrap_or_default();
        if url.contains(target_domain) {
            break;
        }
        let has_form: bool = page
            .evaluate("document.forms.length > 0")
            .await
            .ok()
            .and_then(|v| v.into_value().ok())
            .unwrap_or(false);
        if !has_form {
            break;
        }
        crawler
            .logger
            .info(&format!("{} SAML form {} — submitting", label, i + 1));
        let _ = page.evaluate("document.forms[0].submit();").await;
        do_sleep(3).await;
        let _ = page.wait_for_navigation().await;
    }
}

pub async fn run_download_csf_crawler(
    crawler: &Crawler,
) -> Result<CrawlerResponse, Box<dyn Error>> {
    let (browser, handler) = crawler.build_browser().await?;
    let _handler_task = tokio::spawn(async move {
        let mut handler = handler;
        loop {
            if handler.next().await.is_none() {
                break;
            }
        }
    });

    // Step 1: Login
    let page = login_csf(&browser, crawler).await?;

    crawler
        .logger
        .info("CSF: waiting for post-login redirects to settle");
    do_sleep(3).await;
    let _ = page.wait_for_navigation().await;
    do_sleep(2).await;

    // Step 2: Navigate to the SSO thrower (full URL with query params required by SAT)
    crawler
        .logger
        .info("CSF: navigating to SSO thrower (lanzador.jsf)");
    page.goto(CSF_SSO_ENTRY_URL).await?;
    do_sleep(3).await;
    let _ = page.wait_for_navigation().await;
    do_sleep(2).await;

    // Step 3: First round of SAML forms (from thrower response)
    crawler.logger.info("CSF: first SAML form round");
    push_saml_forms(&page, "rfcampc.siat.sat.gob.mx", "CSF first-round", crawler).await;

    // Step 4: Follow iframe#iframetoload if present (second SSO hop)
    let current_url = page.url().await?.unwrap_or_default();
    if !current_url.contains("rfcampc.siat.sat.gob.mx") {
        let iframe_src: String = page
            .evaluate(
                "(function() { \
                   var f = document.getElementById('iframetoload'); \
                   return f ? f.src : ''; \
                 })()",
            )
            .await?
            .into_value()
            .unwrap_or_default();

        if !iframe_src.is_empty() {
            crawler
                .logger
                .info(&format!("CSF: following iframe → {}", iframe_src));
            page.goto(&iframe_src).await?;
            do_sleep(3).await;
            let _ = page.wait_for_navigation().await;
            do_sleep(2).await;

            crawler.logger.info("CSF: second SAML form round (iframe)");
            push_saml_forms(
                &page,
                "rfcampc.siat.sat.gob.mx",
                "CSF second-round",
                crawler,
            )
            .await;
        }
    }

    // Step 5: Navigate to the certificate form page if not already there
    let current_url = page.url().await?.unwrap_or_default();
    crawler
        .logger
        .info(&format!("CSF: URL after SSO = {}", current_url));
    if !current_url.contains("rfcampc.siat.sat.gob.mx") {
        crawler
            .logger
            .info("CSF: navigating directly to certificate form page");
        page.goto(CSF_CERTIFICATE_URL).await?;
        do_sleep(3).await;
        let _ = page.wait_for_navigation().await;
    }

    do_sleep(2).await;

    // Step 6: Find the "Generar Constancia" button by its label text
    let button_id: String = page
        .evaluate(
            "(function() { \
               var form = document.getElementById('formReimpAcuse'); \
               if (!form) return ''; \
               var btns = Array.from(form.querySelectorAll('button')); \
               var generar = btns.find(function(b) { \
                 return (b.innerText || b.textContent || '').toLowerCase().indexOf('generar') >= 0; \
               }); \
               return generar ? (generar.id || generar.name || '') : ''; \
             })()",
        )
        .await?
        .into_value()
        .unwrap_or_default();

    crawler
        .logger
        .info(&format!("CSF: clicking button id={}", button_id));

    if button_id.is_empty() {
        return Err("CSF: 'Generar Constancia' button not found on formReimpAcuse".into());
    }

    // Step 7: Click the button — triggers a PrimeFaces AJAX request that prepares the PDF
    // server-side and rotates the ViewState.
    let btn = page.find_element(&format!("[id='{}']", button_id)).await?;
    btn.click().await?;
    do_sleep(5).await;

    // Step 8: Grab fresh cookies from the browser (session state was updated by the click)
    // and GET IdcGeneraConstancia.jsf — this is the PHP scraper's getFileContent() call.
    crawler
        .logger
        .info("CSF: GET IdcGeneraConstancia.jsf for PDF");
    let fresh_cookies = page.get_cookies().await?;
    let http_client = build_http_client(fresh_cookies)?;

    let pdf_resp = http_client.get(CSF_PDF_URL).send().await?;
    let status = pdf_resp.status();
    let pdf_bytes = pdf_resp.bytes().await?;

    if !status.is_success() {
        return Err(format!("CSF: PDF request failed with status {}", status).into());
    }

    if pdf_bytes.len() < 4 || &pdf_bytes[..4] != b"%PDF" {
        return Err(format!(
            "CSF: response is not a PDF (first chars: {})",
            std::str::from_utf8(&pdf_bytes[..pdf_bytes.len().min(60)]).unwrap_or("(binary)")
        )
        .into());
    }

    // Step 9: Deliver the PDF — via handler if wired, otherwise save to disk as fallback
    if let Some(handler) = &crawler.csf_event_handler {
        handler
            .on_csf_event(CsfEvent::PdfDownloaded {
                content: pdf_bytes.to_vec(),
            })
            .await;
        Ok(CrawlerResponse {
            success: true,
            message: "CSF PDF delivered to handler".to_string(),
        })
    } else {
        let rfc = crawler.config.credentials.username.clone();
        let save_path = get_download_folder(Some(rfc));
        std::fs::create_dir_all(&save_path)?;
        let dest = std::path::Path::new(&save_path).join("csf.pdf");
        std::fs::write(&dest, &pdf_bytes)?;
        crawler
            .logger
            .info(&format!("CSF: saved to {}", dest.display()));
        Ok(CrawlerResponse {
            success: true,
            message: format!("CSF saved to {}", dest.display()),
        })
    }
}

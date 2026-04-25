# satcrawler

Automates login and invoice retrieval from Mexico's SAT portal (`portalcfdi.facturaelectronica.sat.gob.mx`). Uses a headless Chromium browser for authentication, then switches to direct HTTP requests for downloading XML and PDF invoice files.

## Quick start

```rust
use satcrawler::{
    Crawler, CrawlerConfig, CrawlerType, Credentials, CrawlerOptions, LoginType,
    Invoice, InvoiceEvent, InvoiceEventHandler, SharedInvoiceEventHandler,
};
use async_trait::async_trait;
use std::sync::Arc;

struct MyHandler;

#[async_trait]
impl InvoiceEventHandler for MyHandler {
    async fn should_download(&self, invoice: &Invoice) -> bool {
        true // download everything; override to skip already-processed invoices
    }

    async fn on_invoice_event(&self, event: InvoiceEvent) {
        match event {
            InvoiceEvent::XmlDownloaded { invoice, content } => { /* save content */ }
            InvoiceEvent::PdfDownloaded { invoice, content } => { /* save content */ }
            InvoiceEvent::XmlDownloadFailed { invoice, error } => { /* log or retry */ }
            InvoiceEvent::PdfDownloadFailed { invoice, error } => { /* log or retry */ }
            InvoiceEvent::Skipped { invoice } => { /* already handled */ }
        }
    }
}

#[tokio::main]
async fn main() {
    let config = CrawlerConfig::new(
        Credentials {
            login_type: LoginType::Ciec,
            username: "RFC123456789".to_string(),
            password: "secret".to_string(),
            crt_path: None,
            key_path: None,
        },
        CrawlerOptions { headless: true, sandbox: true },
    );

    let handler: SharedInvoiceEventHandler = Arc::new(MyHandler);
    let response = Crawler::new(CrawlerType::DownloadInvoices, config)
        .with_event_handler(handler)
        .run()
        .await;

    println!("{}", response.success);
}
```

Captcha solving requires the `TWOCAPTCHA_API_KEY` environment variable to be set.

---

## Configuration

### `CrawlerConfig`

| Method | Description |
|---|---|
| `CrawlerConfig::new(credentials, options)` | Creates config and writes it to `~/sat-cli/config.json` |
| `CrawlerConfig::new_from_file()` | Loads config from `~/sat-cli/config.json`; returns default if missing |
| `CrawlerConfig::builder()` | Returns a `CrawlerConfigBuilder` for a fluent API |
| `config.validate()` | Returns `Err` if FIEL login is selected but cert/key paths are missing or not found |

### `CrawlerConfigBuilder`

```rust
let config = CrawlerConfig::builder()
    .with_credentials("RFC123456789".to_string(), "secret".to_string())
    .with_head()    // show browser window (default: headless)
    .no_sandbox()   // disable Chrome sandbox (needed in some Docker/CI environments)
    .build()
    .unwrap();
```

### `Credentials`

| Field | Type | Description |
|---|---|---|
| `login_type` | `LoginType` | `LoginType::Ciec` (password) or `LoginType::Fiel` (certificate) |
| `username` | `String` | RFC (tax ID) |
| `password` | `String` | CIEC password or FIEL private key password |
| `crt_path` | `Option<String>` | Path to `.cer` file — required for FIEL |
| `key_path` | `Option<String>` | Path to `.key` file — required for FIEL |

### `CrawlerFilters`

Narrows which invoices are processed. Without filters, all invoices from 2020 to today are fetched.

```rust
use satcrawler::{CrawlerFilters, parse_date};

let filters = CrawlerFilters {
    start_date: Some(parse_date("01/01/2024").unwrap()),
    end_date:   Some(parse_date("31/12/2024").unwrap()),
};

Crawler::new(CrawlerType::DownloadIssuedInvoices, config)
    .with_filters(Some(filters))
    .run()
    .await;
```

`parse_date` expects `dd/mm/YYYY` format.

---

## Running a crawl

```rust
Crawler::new(crawler_type, config)
    .with_filters(Some(filters))    // optional
    .with_event_handler(handler)    // optional — receives events and controls skip logic
    .run()
    .await
```

### `CrawlerType`

| Variant | Description |
|---|---|
| `DownloadInvoices` | Downloads both issued and received invoices |
| `DownloadIssuedInvoices` | Downloads only invoices you issued |
| `DownloadReceivedInvoices` | Downloads only invoices you received |
| `ValidateCredentials` | Logs in and exits; use to verify credentials work |

### `CrawlerResponse`

```rust
pub struct CrawlerResponse {
    pub success: bool,
    pub message: String,
}
```

---

## Handling invoice events

Implement `InvoiceEventHandler` to control the full invoice lifecycle. This is the single integration point — the crawler never writes files itself.

```rust
#[async_trait]
pub trait InvoiceEventHandler: Send + Sync {
    async fn should_download(&self, invoice: &Invoice) -> bool {
        true // default: download everything
    }
    async fn on_invoice_event(&self, event: InvoiceEvent);
}
```

`should_download` is called before any HTTP request is made. Return `false` to skip an invoice — a `Skipped` event will fire so the handler can log or track it.

### `InvoiceEvent`

```rust
pub enum InvoiceEvent {
    XmlDownloaded { invoice: Invoice, content: Vec<u8> },
    PdfDownloaded { invoice: Invoice, content: Vec<u8> },
    XmlDownloadFailed { invoice: Invoice, error: String },
    PdfDownloadFailed { invoice: Invoice, error: String },
    Skipped { invoice: Invoice },
}
```

- `XmlDownloaded` and `PdfDownloaded` fire in parallel per invoice.
- `XmlDownloadFailed` / `PdfDownloadFailed` fire when an HTTP error occurs or no download token was found in the page.
- `Skipped` fires when `should_download` returns `false`.

### `Invoice`

| Field | Description |
|---|---|
| `uuid` | Folio fiscal (UUID) |
| `fiscal_id` | Folio |
| `issuer_tax_id` | RFC of the issuer |
| `issuer_name` | Name of the issuer |
| `receiver_tax_id` | RFC of the receiver |
| `receiver_name` | Name of the receiver |
| `issued_at` | Emission timestamp |
| `certified_at` | Certification timestamp |
| `total` | Total amount |
| `invoice_type` | Type (e.g. ingreso, egreso) |
| `invoice_status` | Status (e.g. vigente, cancelado) |

---

## Skipping already-downloaded invoices

Override `should_download` on your handler to avoid re-downloading files that were already saved:

```rust
#[async_trait]
impl InvoiceEventHandler for MyHandler {
    async fn should_download(&self, invoice: &Invoice) -> bool {
        let base = std::path::Path::new(&self.download_path);
        !base.join(format!("{}.xml", invoice.uuid)).exists()
            || !base.join(format!("{}.pdf", invoice.uuid)).exists()
    }

    async fn on_invoice_event(&self, event: InvoiceEvent) { /* ... */ }
}
```

---

## Download folder utility

`get_download_folder` resolves the directory where files should be saved. It respects the `SATCLI_DOCUMENTS_FOLDER` environment variable; otherwise it defaults to `~/sat-cli/documents/<addition>`.

```rust
use satcrawler::get_download_folder;

// ~/sat-cli/documents/RFC123456789
let path = get_download_folder(Some("RFC123456789".to_string()));

// ~/sat-cli/documents/  (no sub-folder)
let path = get_download_folder(None);
```

Set `SATCLI_DOCUMENTS_FOLDER` to override the base path entirely (the `addition` argument is ignored when the env var is set).

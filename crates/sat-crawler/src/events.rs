use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Invoice {
    pub uuid: String,
    pub fiscal_id: String,
    pub issuer_tax_id: String,
    pub issuer_name: String,
    pub receiver_tax_id: String,
    pub receiver_name: String,
    pub issued_at: String,
    pub certified_at: String,
    pub total: String,
    pub invoice_type: String,
    pub invoice_status: String,
}

#[derive(Debug, Clone)]
pub enum InvoiceEvent {
    Downloaded {
        invoice: Invoice,
        download_path: String,
    },
    Skipped {
        invoice: Invoice,
        download_path: String,
    },
}

#[async_trait]
pub trait InvoiceEventHandler: Send + Sync {
    async fn on_invoice_event(&self, event: InvoiceEvent);
}

pub type SharedInvoiceEventHandler = Arc<dyn InvoiceEventHandler>;

#[async_trait]
pub trait InvoiceDownloadDecider: Send + Sync {
    async fn should_download_invoice(&self, invoice: &Invoice, download_path: &str) -> bool;
}

pub type SharedInvoiceDownloadDecider = Arc<dyn InvoiceDownloadDecider>;

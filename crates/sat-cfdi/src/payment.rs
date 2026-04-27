use crate::catalogs::{Currency, PaymentForm, TaxFactor, TaxObject, TaxType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentsComplement {
    #[serde(rename(deserialize = "@Version"))]
    pub version: String,
    #[serde(rename(deserialize = "Totales"))]
    pub totals: PaymentTotals,
    #[serde(rename(deserialize = "Pago"), default)]
    pub payments: Vec<Payment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTotals {
    #[serde(rename(deserialize = "@TotalRetencionesIVA"), default)]
    pub total_iva_withheld: Option<String>,
    #[serde(rename(deserialize = "@TotalRetencionesISR"), default)]
    pub total_isr_withheld: Option<String>,
    #[serde(rename(deserialize = "@TotalRetencionesIEPS"), default)]
    pub total_ieps_withheld: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosBaseIVA16"), default)]
    pub total_transferred_iva_base_16: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosImpuestoIVA16"), default)]
    pub total_transferred_iva_tax_16: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosBaseIVA8"), default)]
    pub total_transferred_iva_base_8: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosImpuestoIVA8"), default)]
    pub total_transferred_iva_tax_8: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosBaseIVA0"), default)]
    pub total_transferred_iva_base_0: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosImpuestoIVA0"), default)]
    pub total_transferred_iva_tax_0: Option<String>,
    #[serde(rename(deserialize = "@TotalTrasladosBaseIVAExento"), default)]
    pub total_transferred_iva_base_exempt: Option<String>,
    #[serde(rename(deserialize = "@MontoTotalPagos"))]
    pub total_payments_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    #[serde(rename(deserialize = "@FechaPago"))]
    pub payment_date: String,
    #[serde(rename(deserialize = "@FormaDePagoP"))]
    pub payment_form: PaymentForm,
    #[serde(rename(deserialize = "@MonedaP"))]
    pub currency: Currency,
    #[serde(rename(deserialize = "@TipoCambioP"), default)]
    pub exchange_rate: Option<String>,
    #[serde(rename(deserialize = "@Monto"))]
    pub amount: String,
    #[serde(rename(deserialize = "@NumOperacion"), default)]
    pub operation_number: Option<String>,
    #[serde(rename(deserialize = "@RfcEmisorCtaOrd"), default)]
    pub ordering_account_issuer_tax_id: Option<String>,
    #[serde(rename(deserialize = "@NomBancoOrdExt"), default)]
    pub bank_name: Option<String>,
    #[serde(rename(deserialize = "@CtaOrdenante"), default)]
    pub ordering_account: Option<String>,
    #[serde(rename(deserialize = "@RfcEmisorCtaBen"), default)]
    pub beneficiary_account_issuer_tax_id: Option<String>,
    #[serde(rename(deserialize = "@CtaBeneficiario"), default)]
    pub beneficiary_account: Option<String>,
    #[serde(rename(deserialize = "@TipoCadPago"), default)]
    pub payment_chain_type: Option<String>,
    #[serde(rename(deserialize = "@CertPago"), default)]
    pub payment_certificate: Option<String>,
    #[serde(rename(deserialize = "@CadPago"), default)]
    pub payment_chain: Option<String>,
    #[serde(rename(deserialize = "@SelloPago"), default)]
    pub payment_seal: Option<String>,

    #[serde(rename(deserialize = "DoctoRelacionado"), default)]
    pub related_documents: Vec<RelatedDocument>,
    #[serde(rename(deserialize = "ImpuestosP"), default)]
    pub taxes: Option<PaymentTaxes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocument {
    #[serde(rename(deserialize = "@IdDocumento"))]
    pub document_id: String,
    #[serde(rename(deserialize = "@Serie"), default)]
    pub series: Option<String>,
    #[serde(rename(deserialize = "@Folio"), default)]
    pub fiscal_id: Option<String>,
    #[serde(rename(deserialize = "@MonedaDR"))]
    pub document_currency: Currency,
    #[serde(rename(deserialize = "@EquivalenciaDR"), default)]
    pub exchange_equivalence: Option<String>,
    #[serde(rename(deserialize = "@NumParcialidad"))]
    pub installment_number: String,
    #[serde(rename(deserialize = "@ImpSaldoAnt"))]
    pub previous_balance: String,
    #[serde(rename(deserialize = "@ImpPagado"))]
    pub paid_amount: String,
    #[serde(rename(deserialize = "@ImpSaldoInsoluto"))]
    pub outstanding_balance: String,
    #[serde(rename(deserialize = "@ObjetoImpDR"))]
    pub tax_object: TaxObject,

    #[serde(rename(deserialize = "ImpuestosDR"), default)]
    pub taxes: Option<RelatedDocumentTaxes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentTaxes {
    #[serde(rename(deserialize = "RetencionesDR"), default)]
    pub withholdings: Option<RelatedDocumentWithholdingList>,
    #[serde(rename(deserialize = "TrasladosDR"), default)]
    pub transfers: Option<RelatedDocumentTransferList>,
}

impl RelatedDocumentTaxes {
    pub fn transfers(&self) -> &[RelatedDocumentTransfer] {
        self.transfers
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
    pub fn withholdings(&self) -> &[RelatedDocumentWithholding] {
        self.withholdings
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentWithholdingList {
    #[serde(rename(deserialize = "RetencionDR"), default)]
    pub items: Vec<RelatedDocumentWithholding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentTransferList {
    #[serde(rename(deserialize = "TrasladoDR"), default)]
    pub items: Vec<RelatedDocumentTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentWithholding {
    #[serde(rename(deserialize = "@BaseDR"))]
    pub base: String,
    #[serde(rename(deserialize = "@ImpuestoDR"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@TipoFactorDR"))]
    pub factor_type: TaxFactor,
    #[serde(rename(deserialize = "@TasaOCuotaDR"))]
    pub rate_or_amount: String,
    #[serde(rename(deserialize = "@ImporteDR"))]
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedDocumentTransfer {
    #[serde(rename(deserialize = "@BaseDR"))]
    pub base: String,
    #[serde(rename(deserialize = "@ImpuestoDR"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@TipoFactorDR"))]
    pub factor_type: TaxFactor,
    #[serde(rename(deserialize = "@TasaOCuotaDR"), default)]
    pub rate_or_amount: Option<String>,
    #[serde(rename(deserialize = "@ImporteDR"), default)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTaxes {
    #[serde(rename(deserialize = "RetencionesP"), default)]
    pub withholdings: Option<PaymentWithholdingList>,
    #[serde(rename(deserialize = "TrasladosP"), default)]
    pub transfers: Option<PaymentTransferList>,
}

impl PaymentTaxes {
    pub fn transfers(&self) -> &[PaymentTransfer] {
        self.transfers
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
    pub fn withholdings(&self) -> &[PaymentWithholding] {
        self.withholdings
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentWithholdingList {
    #[serde(rename(deserialize = "RetencionP"), default)]
    pub items: Vec<PaymentWithholding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransferList {
    #[serde(rename(deserialize = "TrasladoP"), default)]
    pub items: Vec<PaymentTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentWithholding {
    #[serde(rename(deserialize = "@ImpuestoP"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@ImporteP"))]
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransfer {
    #[serde(rename(deserialize = "@BaseP"))]
    pub base: String,
    #[serde(rename(deserialize = "@ImpuestoP"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@TipoFactorP"))]
    pub factor_type: TaxFactor,
    #[serde(rename(deserialize = "@TasaOCuotaP"), default)]
    pub rate_or_amount: Option<String>,
    #[serde(rename(deserialize = "@ImporteP"), default)]
    pub amount: Option<String>,
}

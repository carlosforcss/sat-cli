use crate::catalogs::{
    CfdiUse, Currency, DocumentType, ExportType, FiscalRegime, InvoicePeriodicity, PaymentForm,
    PaymentMethod, RelationType, TaxFactor, TaxObject, TaxType,
};
use crate::complement::Complement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    #[serde(rename(deserialize = "@Version"))]
    pub version: String,
    #[serde(rename(deserialize = "@Serie"), default)]
    pub series: Option<String>,
    #[serde(rename(deserialize = "@Folio"), default)]
    pub fiscal_id: Option<String>,
    #[serde(rename(deserialize = "@Fecha"))]
    pub issued_at: String,
    #[serde(rename(deserialize = "@Sello"))]
    pub seal: String,
    #[serde(rename(deserialize = "@FormaPago"), default)]
    pub payment_form: Option<PaymentForm>,
    #[serde(rename(deserialize = "@NoCertificado"))]
    pub certificate_number: String,
    #[serde(rename(deserialize = "@Certificado"))]
    pub certificate: String,
    #[serde(rename(deserialize = "@CondicionesDePago"), default)]
    pub payment_conditions: Option<String>,
    #[serde(rename(deserialize = "@SubTotal"))]
    pub subtotal: String,
    #[serde(rename(deserialize = "@Descuento"), default)]
    pub discount: Option<String>,
    #[serde(rename(deserialize = "@Moneda"))]
    pub currency: Currency,
    #[serde(rename(deserialize = "@TipoCambio"), default)]
    pub exchange_rate: Option<String>,
    #[serde(rename(deserialize = "@Total"))]
    pub total: String,
    #[serde(rename(deserialize = "@TipoDeComprobante"))]
    pub document_type: DocumentType,
    #[serde(rename(deserialize = "@Exportacion"), default)]
    pub export: Option<ExportType>,
    #[serde(rename(deserialize = "@MetodoPago"), default)]
    pub payment_method: Option<PaymentMethod>,
    #[serde(rename(deserialize = "@LugarExpedicion"))]
    pub issue_place: String,
    #[serde(rename(deserialize = "@Confirmacion"), default)]
    pub confirmation: Option<String>,

    #[serde(rename(deserialize = "InformacionGlobal"), default)]
    pub global_info: Option<GlobalInfo>,
    #[serde(rename(deserialize = "CfdiRelacionados"), default)]
    pub related_cfdis: Vec<RelatedCfdis>,
    #[serde(rename(deserialize = "Emisor"))]
    pub issuer: Issuer,
    #[serde(rename(deserialize = "Receptor"))]
    pub recipient: Recipient,
    #[serde(rename(deserialize = "Conceptos"))]
    pub line_items: LineItems,
    #[serde(rename(deserialize = "Impuestos"), default)]
    pub taxes: Option<DocumentTaxes>,

    /// Typed complement — not read from XML directly; populated by `crate::parse()`.
    #[serde(skip_deserializing, default)]
    pub complement: Option<Complement>,
}

impl Invoice {
    pub fn line_items(&self) -> &[LineItem] {
        &self.line_items.items
    }

    pub fn payments(&self) -> Option<&crate::payment::PaymentsComplement> {
        self.complement.as_ref()?.payments.as_ref()
    }

    pub fn payroll(&self) -> Option<&crate::payroll::PayrollComplement> {
        self.complement.as_ref()?.payroll.as_ref()
    }

    pub fn freight_transport(&self) -> Option<&crate::freight::FreightTransportComplement> {
        self.complement.as_ref()?.freight_transport.as_ref()
    }

    pub fn fiscal_stamp_uuid(&self) -> Option<&str> {
        self.complement.as_ref()?.uuid.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalInfo {
    #[serde(rename(deserialize = "@Periodicidad"))]
    pub periodicity: InvoicePeriodicity,
    #[serde(rename(deserialize = "@Meses"))]
    pub months: String,
    #[serde(rename(deserialize = "@Año"))]
    pub year: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedCfdis {
    #[serde(rename(deserialize = "@TipoRelacion"))]
    pub relation_type: RelationType,
    #[serde(rename(deserialize = "CfdiRelacionado"), default)]
    pub items: Vec<RelatedCfdi>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedCfdi {
    #[serde(rename(deserialize = "@UUID"))]
    pub uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issuer {
    #[serde(rename(deserialize = "@Rfc"))]
    pub taxpayer_id: String,
    #[serde(rename(deserialize = "@Nombre"), default)]
    pub name: Option<String>,
    #[serde(rename(deserialize = "@RegimenFiscal"))]
    pub fiscal_regime: FiscalRegime,
    #[serde(rename(deserialize = "@FacAtrAdquirente"), default)]
    pub acquirer_fac_attr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    #[serde(rename(deserialize = "@Rfc"))]
    pub taxpayer_id: String,
    #[serde(rename(deserialize = "@Nombre"), default)]
    pub name: Option<String>,
    #[serde(rename(deserialize = "@DomicilioFiscalReceptor"), default)]
    pub fiscal_domicile: Option<String>,
    #[serde(rename(deserialize = "@ResidenciaFiscal"), default)]
    pub tax_residence: Option<String>,
    #[serde(rename(deserialize = "@NumRegIdTrib"), default)]
    pub foreign_tax_id: Option<String>,
    #[serde(rename(deserialize = "@RegimenFiscalReceptor"), default)]
    pub fiscal_regime: Option<FiscalRegime>,
    #[serde(rename(deserialize = "@UsoCFDI"))]
    pub cfdi_use: CfdiUse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LineItems {
    #[serde(rename(deserialize = "Concepto"), default)]
    pub items: Vec<LineItem>,
}

impl Serialize for LineItems {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.items.serialize(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    #[serde(rename(deserialize = "@ClaveProdServ"))]
    pub product_service_key: String,
    #[serde(rename(deserialize = "@NoIdentificacion"), default)]
    pub id_number: Option<String>,
    #[serde(rename(deserialize = "@Cantidad"))]
    pub quantity: String,
    #[serde(rename(deserialize = "@ClaveUnidad"))]
    pub unit_key: String,
    #[serde(rename(deserialize = "@Unidad"), default)]
    pub unit: Option<String>,
    #[serde(rename(deserialize = "@Descripcion"))]
    pub description: String,
    #[serde(rename(deserialize = "@ValorUnitario"))]
    pub unit_value: String,
    #[serde(rename(deserialize = "@Importe"))]
    pub amount: String,
    #[serde(rename(deserialize = "@Descuento"), default)]
    pub discount: Option<String>,
    #[serde(rename(deserialize = "@ObjetoImp"), default)]
    pub tax_object: Option<TaxObject>,

    #[serde(rename(deserialize = "Impuestos"), default)]
    pub taxes: Option<LineItemTaxes>,
    #[serde(rename(deserialize = "ACuentaTerceros"), default)]
    pub third_party: Option<ThirdParty>,
    #[serde(rename(deserialize = "InformacionAduanera"), default)]
    pub customs_info: Vec<CustomsInfo>,
    #[serde(rename(deserialize = "CuentaPredial"), default)]
    pub property_tax_accounts: Vec<PropertyTaxAccount>,
    #[serde(rename(deserialize = "Parte"), default)]
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItemTaxes {
    #[serde(rename(deserialize = "Traslados"), default)]
    pub transfers: Option<TransferList>,
    #[serde(rename(deserialize = "Retenciones"), default)]
    pub withholdings: Option<WithholdingList>,
}

impl LineItemTaxes {
    pub fn transfers(&self) -> &[Transfer] {
        self.transfers
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
    pub fn withholdings(&self) -> &[Withholding] {
        self.withholdings
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTaxes {
    #[serde(rename(deserialize = "@TotalImpuestosRetenidos"), default)]
    pub total_withheld: Option<String>,
    #[serde(rename(deserialize = "@TotalImpuestosTrasladados"), default)]
    pub total_transferred: Option<String>,
    #[serde(rename(deserialize = "Retenciones"), default)]
    pub withholdings: Option<WithholdingList>,
    #[serde(rename(deserialize = "Traslados"), default)]
    pub transfers: Option<TransferList>,
}

impl DocumentTaxes {
    pub fn transfers(&self) -> &[Transfer] {
        self.transfers
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
    pub fn withholdings(&self) -> &[Withholding] {
        self.withholdings
            .as_ref()
            .map(|l| l.items.as_slice())
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferList {
    #[serde(rename(deserialize = "Traslado"), default)]
    pub items: Vec<Transfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithholdingList {
    #[serde(rename(deserialize = "Retencion"), default)]
    pub items: Vec<Withholding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    #[serde(rename(deserialize = "@Base"), default)]
    pub base: Option<String>,
    #[serde(rename(deserialize = "@Impuesto"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@TipoFactor"))]
    pub factor_type: TaxFactor,
    #[serde(rename(deserialize = "@TasaOCuota"), default)]
    pub rate_or_amount: Option<String>,
    #[serde(rename(deserialize = "@Importe"), default)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Withholding {
    #[serde(rename(deserialize = "@Base"), default)]
    pub base: Option<String>,
    #[serde(rename(deserialize = "@Impuesto"))]
    pub tax: TaxType,
    #[serde(rename(deserialize = "@TipoFactor"), default)]
    pub factor_type: Option<TaxFactor>,
    #[serde(rename(deserialize = "@TasaOCuota"), default)]
    pub rate_or_amount: Option<String>,
    #[serde(rename(deserialize = "@Importe"))]
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdParty {
    #[serde(rename(deserialize = "@RfcACuentaTerceros"))]
    pub taxpayer_id: String,
    #[serde(rename(deserialize = "@NombreACuentaTerceros"), default)]
    pub name: Option<String>,
    #[serde(rename(deserialize = "@RegimenFiscalACuentaTerceros"))]
    pub fiscal_regime: FiscalRegime,
    #[serde(rename(deserialize = "@DomicilioFiscalACuentaTerceros"))]
    pub fiscal_domicile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomsInfo {
    #[serde(rename(deserialize = "@NumeroPedimento"))]
    pub customs_document_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyTaxAccount {
    #[serde(rename(deserialize = "@Numero"))]
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    #[serde(rename(deserialize = "@ClaveProdServ"))]
    pub product_service_key: String,
    #[serde(rename(deserialize = "@NoIdentificacion"), default)]
    pub id_number: Option<String>,
    #[serde(rename(deserialize = "@Cantidad"))]
    pub quantity: String,
    #[serde(rename(deserialize = "@Unidad"), default)]
    pub unit: Option<String>,
    #[serde(rename(deserialize = "@Descripcion"))]
    pub description: String,
    #[serde(rename(deserialize = "@ValorUnitario"), default)]
    pub unit_value: Option<String>,
    #[serde(rename(deserialize = "@Importe"), default)]
    pub amount: Option<String>,
    #[serde(rename(deserialize = "InformacionAduanera"), default)]
    pub customs_info: Vec<CustomsInfo>,
}

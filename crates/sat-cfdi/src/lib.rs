mod catalogs;
mod complement;
mod error;
mod freight;
mod invoice;
mod payment;
mod payroll;

pub use catalogs::{
    CfdiUse, ContractType, Currency, DocumentType, ExportType, FiscalRegime, InvoicePeriodicity,
    PaymentForm, PaymentMethod, PaymentRegime, PayrollPeriodicity, RelationType, TaxFactor,
    TaxObject, TaxType,
};
pub use complement::{Complement, ComplementKind, FiscalStamp};
pub use error::CfdiError;
pub use freight::{
    Address, AirTransportShell, CustomsDoc, CustomsRegime, CustomsRegimes, FederalRoadTransport,
    FreightTransportComplement, Goods, Location, Locations, MaritimeTransportShell, Merchandise,
    MerchandiseDetail, QuantityTransport, RailTransportShell, RoadTransport, TrackingId, Trailer,
    Trailers, TransportFigure, TransportFigures, TransportPart, VehicleId, VehicleInsurance,
};
pub use invoice::{
    CustomsInfo, DocumentTaxes, GlobalInfo, Invoice, Issuer, LineItem, LineItemTaxes, LineItems,
    Part, PropertyTaxAccount, Recipient, RelatedCfdi, RelatedCfdis, ThirdParty, Transfer,
    TransferList, Withholding, WithholdingList,
};
pub use payment::{
    Payment, PaymentTaxes, PaymentTotals, PaymentTransfer, PaymentTransferList, PaymentWithholding,
    PaymentWithholdingList, PaymentsComplement, RelatedDocument, RelatedDocumentTaxes,
    RelatedDocumentTransfer, RelatedDocumentTransferList, RelatedDocumentWithholding,
    RelatedDocumentWithholdingList,
};
pub use payroll::{
    BalanceCompensation, Deduction, Deductions, Disabilities, Disability, Earning, Earnings,
    EmploymentSubsidy, OtherPayment, OtherPayments, Overtime, PayrollComplement, PayrollEmployee,
    PayrollEmployer, RetirementPayment, SeparationPayment, SncfEntity, StockOptions,
    Subcontracting,
};

/// Parse a CFDI XML document from a UTF-8 string.
pub fn parse(xml: &str) -> Result<Invoice, CfdiError> {
    // Strip UTF-8 BOM if present.
    let xml = xml.trim_start_matches('\u{FEFF}');

    // Reject HTML error pages saved with .xml extension.
    let trimmed = xml.trim_start();
    if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
        return Err(CfdiError::NotCfdi);
    }

    // Pre-extract the Complemento inner XML before serde sees the document.
    // quick-xml serde cannot handle xs:any children; we pull them out manually.
    let ns_decls = root_ns_decls(xml);
    let (xml_without_complement, complement_inner) = split_complement(xml);

    let mut doc: Invoice = quick_xml::de::from_str(&xml_without_complement)?;

    if let Some(inner) = complement_inner {
        if let Ok(c) = complement::parse_complement(&inner, &ns_decls) {
            doc.complement = Some(c);
        }
    }
    Ok(doc)
}

/// Extract all xmlns:* declarations from the root element of the XML document.
/// These are passed to parse_complement so inherited namespace prefixes (e.g.
/// xmlns:nomina12 declared on cfdi:Comprobante, not on the complement element)
/// are resolved correctly.
fn root_ns_decls(xml: &str) -> String {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let mut decls = String::new();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                    if key.starts_with("xmlns") {
                        let val = std::str::from_utf8(attr.value.as_ref()).unwrap_or("");
                        decls.push_str(&format!(" {}=\"{}\"", key, val));
                    }
                }
                return decls;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    String::new()
}

/// Parse a CFDI XML document from raw bytes (must be UTF-8).
pub fn parse_bytes(xml: &[u8]) -> Result<Invoice, CfdiError> {
    let s = std::str::from_utf8(xml)
        .map_err(|e| CfdiError::Xml(quick_xml::DeError::Custom(e.to_string())))?;
    parse(s)
}

// ── Complement extraction ────────────────────────────────────────────────────

/// Split a CFDI XML string into (xml_without_complemento, complemento_inner_xml).
/// Uses quick-xml's Reader to find exact byte offsets of the Complemento element.
fn split_complement(xml: &str) -> (String, Option<String>) {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut outer_start: Option<u64> = None;
    let mut inner_start: Option<u64> = None;
    let mut inner_end: Option<u64> = None;
    let mut outer_end: Option<u64> = None;
    let mut depth: i32 = 0;

    loop {
        let before = reader.buffer_position();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ln = e.local_name();
                let local = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                if local == "Complemento" && outer_start.is_none() {
                    outer_start = Some(before);
                    inner_start = Some(reader.buffer_position());
                    depth = 1;
                } else if outer_start.is_some() {
                    depth += 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let ln = e.local_name();
                let local = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                if local == "Complemento" && outer_start.is_none() {
                    outer_start = Some(before);
                    outer_end = Some(reader.buffer_position());
                    break;
                }
            }
            Ok(Event::End(ref e)) => {
                let ln = e.local_name();
                let local = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                if outer_start.is_some() && local == "Complemento" {
                    depth -= 1;
                    if depth == 0 {
                        inner_end = Some(before);
                        outer_end = Some(reader.buffer_position());
                        break;
                    }
                } else if outer_start.is_some() {
                    depth -= 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    match (outer_start, inner_start.zip(inner_end), outer_end) {
        (Some(os), Some((is, ie)), Some(oe)) => {
            let (os, is, ie, oe) = (os as usize, is as usize, ie as usize, oe as usize);
            let inner = xml[is..ie].trim().to_string();
            let stripped = format!("{}{}", &xml[..os], &xml[oe..]);
            (stripped, if inner.is_empty() { None } else { Some(inner) })
        }
        (Some(os), None, Some(oe)) => {
            let (os, oe) = (os as usize, oe as usize);
            let stripped = format!("{}{}", &xml[..os], &xml[oe..]);
            (stripped, None)
        }
        _ => (xml.to_string(), None),
    }
}

// ── Date/time helpers ────────────────────────────────────────────────────────

/// Parse a CFDI datetime string (`YYYY-MM-DDThh:mm:ss`) into a `NaiveDateTime`.
pub fn parse_cfdi_datetime(s: &str) -> Result<chrono::NaiveDateTime, chrono::ParseError> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
}

/// Parse a CFDI date string (`YYYY-MM-DD`) into a `NaiveDate`.
pub fn parse_cfdi_date(s: &str) -> Result<chrono::NaiveDate, chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

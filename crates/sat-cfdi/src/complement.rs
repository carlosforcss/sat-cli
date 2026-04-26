use crate::error::CfdiError;
use crate::freight::FreightTransportComplement;
use crate::payment::PaymentsComplement;
use crate::payroll::PayrollComplement;
use serde::{Deserialize, Serialize};

const NS_PAYMENTS: &str = "http://www.sat.gob.mx/Pagos20";
const NS_PAYROLL: &str = "http://www.sat.gob.mx/nomina12";
const NS_FREIGHT: &str = "http://www.sat.gob.mx/CartaPorte31";
const NS_FISCAL_STAMP: &str = "http://www.sat.gob.mx/TimbreFiscalDigital";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiscalStamp {
    #[serde(rename(deserialize = "@UUID"))]
    pub uuid: String,
    #[serde(rename(deserialize = "@FechaTimbrado"))]
    pub stamp_date: String,
    #[serde(rename(deserialize = "@NoCertificadoSAT"))]
    pub sat_certificate_number: String,
    #[serde(rename(deserialize = "@SelloCFD"))]
    pub cfdi_seal: String,
    #[serde(rename(deserialize = "@SelloSAT"))]
    pub sat_seal: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ComplementKind {
    Payments(PaymentsComplement),
    Payroll(PayrollComplement),
    FreightTransport(FreightTransportComplement),
    FiscalStamp(FiscalStamp),
    Unknown { namespace: String, raw_xml: String },
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Complement {
    pub items: Vec<ComplementKind>,
}

/// Parse the inner XML content of a `cfdi:Complemento` element into typed complement structs.
pub fn parse_complement(inner_xml: &str) -> Result<Complement, CfdiError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    if inner_xml.trim().is_empty() {
        return Ok(Complement::default());
    }

    let mut complement = Complement::default();
    let wrapped = format!("<root>{}</root>", inner_xml);
    let mut reader = Reader::from_str(&wrapped);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut depth = 0u32;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                if depth == 2 {
                    let kind = dispatch_complement_element(e, inner_xml);
                    complement.items.push(kind);
                }
            }
            Ok(Event::Empty(ref e)) => {
                if depth == 1 {
                    let kind = dispatch_complement_element(e, inner_xml);
                    complement.items.push(kind);
                }
            }
            Ok(Event::End(_)) => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(complement)
}

fn dispatch_complement_element(
    e: &quick_xml::events::BytesStart,
    inner_xml: &str,
) -> ComplementKind {
    let local_name = e.local_name();
    let local = std::str::from_utf8(local_name.as_ref()).unwrap_or("");
    let ns_uri = element_namespace(e);
    let elem_xml = capture_element(inner_xml, local);

    match ns_uri.as_str() {
        NS_PAYMENTS => match quick_xml::de::from_str::<PaymentsComplement>(&elem_xml) {
            Ok(p) => ComplementKind::Payments(p),
            Err(e) => ComplementKind::Unknown {
                namespace: ns_uri,
                raw_xml: format!("parse error: {e}"),
            },
        },
        NS_PAYROLL => match quick_xml::de::from_str::<PayrollComplement>(&elem_xml) {
            Ok(p) => ComplementKind::Payroll(p),
            Err(e) => ComplementKind::Unknown {
                namespace: ns_uri,
                raw_xml: format!("parse error: {e}"),
            },
        },
        NS_FREIGHT => match quick_xml::de::from_str::<FreightTransportComplement>(&elem_xml) {
            Ok(f) => ComplementKind::FreightTransport(f),
            Err(e) => ComplementKind::Unknown {
                namespace: ns_uri,
                raw_xml: format!("parse error: {e}"),
            },
        },
        NS_FISCAL_STAMP => match quick_xml::de::from_str::<FiscalStamp>(&elem_xml) {
            Ok(s) => ComplementKind::FiscalStamp(s),
            Err(e) => ComplementKind::Unknown {
                namespace: ns_uri,
                raw_xml: format!("parse error: {e}"),
            },
        },
        other => ComplementKind::Unknown {
            namespace: other.to_string(),
            raw_xml: elem_xml,
        },
    }
}

/// Resolve the namespace URI for an element using its prefix and inline xmlns declarations.
fn element_namespace(e: &quick_xml::events::BytesStart) -> String {
    let name = e.name();
    let full_name = std::str::from_utf8(name.as_ref()).unwrap_or("");
    let prefix = full_name.find(':').map(|i| &full_name[..i]).unwrap_or("");
    let xmlns_key = if prefix.is_empty() {
        "xmlns".to_string()
    } else {
        format!("xmlns:{prefix}")
    };

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        if key == xmlns_key {
            if let Ok(val) = std::str::from_utf8(attr.value.as_ref()) {
                return val.to_string();
            }
        }
    }
    String::new()
}

fn capture_element(xml: &str, local_name: &str) -> String {
    let open_pat_colon = format!(":{} ", local_name);
    let open_pat_bare = format!("<{} ", local_name);
    let open_pat_colon2 = format!(":{}>", local_name);
    let open_pat_bare2 = format!("<{}>", local_name);

    let start = xml
        .find(&open_pat_colon)
        .map(|i| xml[..i].rfind('<').unwrap_or(i))
        .or_else(|| xml.find(&open_pat_bare))
        .or_else(|| {
            xml.find(&open_pat_colon2)
                .map(|i| xml[..i].rfind('<').unwrap_or(i))
        })
        .or_else(|| xml.find(&open_pat_bare2));

    let Some(start) = start else {
        return String::new();
    };

    let close_pat_colon = format!(":{}>", local_name);
    let close_pat_bare = format!("</{}>", local_name);

    let end = xml[start..]
        .find(&close_pat_colon)
        .map(|i| start + i + close_pat_colon.len())
        .or_else(|| {
            xml[start..]
                .find(&close_pat_bare)
                .map(|i| start + i + close_pat_bare.len())
        });

    match end {
        Some(end) => xml[start..end].to_string(),
        None => xml[start..].to_string(),
    }
}

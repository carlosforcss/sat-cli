use crate::error::CfdiError;
use crate::freight::FreightTransportComplement;
use crate::payment::PaymentsComplement;
use crate::payroll::PayrollComplement;
use quick_xml::name::ResolveResult;
use serde::{Deserialize, Serialize};

const NS_PAYMENTS: &str = "http://www.sat.gob.mx/Pagos20";
const NS_PAYROLL: &str = "http://www.sat.gob.mx/nomina12";
const NS_FREIGHT: &str = "http://www.sat.gob.mx/CartaPorte31";
const NS_FISCAL_STAMP: &str = "http://www.sat.gob.mx/TimbreFiscalDigital";

#[derive(Debug, Clone, Deserialize)]
struct FiscalStamp {
    #[serde(rename(deserialize = "@UUID"))]
    uuid: String,
    #[serde(rename(deserialize = "@FechaTimbrado"))]
    stamp_date: String,
    #[serde(rename(deserialize = "@NoCertificadoSAT"))]
    sat_certificate_number: String,
    #[serde(rename(deserialize = "@SelloCFD"))]
    cfdi_seal: String,
    #[serde(rename(deserialize = "@SelloSAT"))]
    sat_seal: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Complement {
    pub uuid: Option<String>,
    pub stamp_date: Option<String>,
    pub sat_certificate_number: Option<String>,
    pub cfdi_seal: Option<String>,
    pub sat_seal: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payroll: Option<PayrollComplement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payments: Option<PaymentsComplement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freight_transport: Option<FreightTransportComplement>,
}

/// Parse the inner XML content of a `cfdi:Complemento` element into a flat Complement struct.
/// `inherited_ns` carries xmlns declarations from the outer document's root element so that
/// namespace prefixes declared there (e.g. xmlns:nomina12) resolve correctly for complement
/// elements that don't redeclare them.
pub fn parse_complement(inner_xml: &str, inherited_ns: &str) -> Result<Complement, CfdiError> {
    use quick_xml::events::Event;
    use quick_xml::NsReader;

    if inner_xml.trim().is_empty() {
        return Ok(Complement::default());
    }

    let mut complement = Complement::default();
    let wrapped = format!("<root {}>{}</root>", inherited_ns, inner_xml);
    let mut reader = NsReader::from_str(&wrapped);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut depth = 0u32;

    loop {
        match reader.read_resolved_event_into(&mut buf) {
            Ok((ns, Event::Start(ref e))) => {
                depth += 1;
                if depth == 2 {
                    let ns_uri = bound_ns_uri(ns);
                    let ln = e.local_name();
                    let local = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                    dispatch_into(&mut complement, &ns_uri, local, inner_xml);
                }
            }
            Ok((ns, Event::Empty(ref e))) => {
                if depth == 1 {
                    let ns_uri = bound_ns_uri(ns);
                    let ln = e.local_name();
                    let local = std::str::from_utf8(ln.as_ref()).unwrap_or("");
                    dispatch_into(&mut complement, &ns_uri, local, inner_xml);
                }
            }
            Ok((_, Event::End(_))) => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            Ok((_, Event::Eof)) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(complement)
}

fn bound_ns_uri(ns: ResolveResult<'_>) -> String {
    match ns {
        ResolveResult::Bound(n) => std::str::from_utf8(n.as_ref()).unwrap_or("").to_string(),
        _ => String::new(),
    }
}

fn dispatch_into(complement: &mut Complement, ns_uri: &str, local: &str, inner_xml: &str) {
    let elem_xml = capture_element(inner_xml, local);
    match ns_uri {
        NS_FISCAL_STAMP => {
            if let Ok(s) = quick_xml::de::from_str::<FiscalStamp>(&elem_xml) {
                complement.uuid = Some(s.uuid);
                complement.stamp_date = Some(s.stamp_date);
                complement.sat_certificate_number = Some(s.sat_certificate_number);
                complement.cfdi_seal = Some(s.cfdi_seal);
                complement.sat_seal = Some(s.sat_seal);
            }
        }
        NS_PAYROLL => {
            if let Ok(p) = quick_xml::de::from_str::<PayrollComplement>(&elem_xml) {
                complement.payroll = Some(p);
            }
        }
        NS_PAYMENTS => {
            if let Ok(p) = quick_xml::de::from_str::<PaymentsComplement>(&elem_xml) {
                complement.payments = Some(p);
            }
        }
        NS_FREIGHT => {
            if let Ok(f) = quick_xml::de::from_str::<FreightTransportComplement>(&elem_xml) {
                complement.freight_transport = Some(f);
            }
        }
        _ => {}
    }
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

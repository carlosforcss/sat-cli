# sat-cfdi

Parses Mexico SAT CFDI XML files (versions 3.3 and 4.0) into typed Rust structs and serializes them to JSON. Handles all major complement types: Pagos 2.0, Nómina 1.2, CartaPorte 3.1, and TimbreFiscalDigital.

## Quick start

```rust
use sat_cfdi::{parse, parse_bytes, CfdiError};

// From a string
let invoice = parse(xml_str)?;

// From raw bytes (must be UTF-8)
let invoice = parse_bytes(&bytes)?;

// Access common fields
println!("{}", invoice.total);                           // "12500.00"
println!("{}", invoice.issuer.taxpayer_id);              // RFC
println!("{:?}", invoice.document_type);                 // DocumentType::Income
println!("{:?}", invoice.fiscal_stamp_uuid());           // Some("uuid-...")
```

JSON output uses English snake_case field names regardless of the original SAT PascalCase XML attribute names.

---

## Entry points

| Function | Description |
|---|---|
| `parse(xml: &str) -> Result<Invoice, CfdiError>` | Parse from a UTF-8 string; strips BOM if present |
| `parse_bytes(xml: &[u8]) -> Result<Invoice, CfdiError>` | Parse from raw bytes |

Both functions reject HTML error pages (saved with `.xml` extension by old downloaders) with `CfdiError::NotCfdi`.

---

## `Invoice`

Top-level struct representing `cfdi:Comprobante`.

### Attributes

| Field | Type | Notes |
|---|---|---|
| `version` | `String` | `"3.3"` or `"4.0"` |
| `series` | `Option<String>` | Invoice series |
| `fiscal_id` | `Option<String>` | Invoice folio number |
| `issued_at` | `String` | Emission datetime, `"YYYY-MM-DDThh:mm:ss"` |
| `seal` | `String` | Issuer digital seal |
| `payment_form` | `Option<PaymentForm>` | How the invoice was paid |
| `certificate_number` | `String` | Issuer certificate number |
| `certificate` | `String` | Base64 certificate |
| `payment_conditions` | `Option<String>` | Free-text payment conditions |
| `subtotal` | `String` | Amount before taxes/discounts |
| `discount` | `Option<String>` | Discount amount |
| `currency` | `Currency` | ISO 4217 currency code |
| `exchange_rate` | `Option<String>` | Exchange rate to MXN |
| `total` | `String` | Total amount |
| `document_type` | `DocumentType` | `I/E/T/P/N` |
| `export` | `Option<ExportType>` | CFDI 4.0 only |
| `payment_method` | `Option<PaymentMethod>` | `PUE` or `PPD` |
| `issue_place` | `String` | Postal code of issue location |
| `confirmation` | `Option<String>` | SAT confirmation code |

### Child elements

| Field | Type | Notes |
|---|---|---|
| `global_info` | `Option<GlobalInfo>` | Global invoice periodicity info |
| `related_cfdis` | `Vec<RelatedCfdis>` | Linked CFDI documents |
| `issuer` | `Issuer` | Emisor |
| `recipient` | `Recipient` | Receptor |
| `line_items` | `LineItems` | Raw wrapper; use `.line_items()` method |
| `taxes` | `Option<DocumentTaxes>` | Document-level tax totals |
| `complement` | `Option<Complement>` | Typed complement — populated by `parse()` |

### Convenience methods

```rust
invoice.line_items()              -> &[LineItem]
invoice.payments()                -> Option<&PaymentsComplement>
invoice.payroll()                 -> Option<&PayrollComplement>
invoice.freight_transport()       -> Option<&FreightTransportComplement>
invoice.fiscal_stamp_uuid()       -> Option<&str>
```

---

## Complement system

Every stamped CFDI has at least a `FiscalStamp` complement. Pagos, Nómina, and CartaPorte invoices carry additional typed complements.

```rust
use sat_cfdi::ComplementKind;

if let Some(complement) = &invoice.complement {
    for item in &complement.items {
        match item {
            ComplementKind::FiscalStamp(s)       => println!("UUID: {}", s.uuid),
            ComplementKind::Payments(p)           => { /* PaymentsComplement */ }
            ComplementKind::Payroll(n)            => { /* PayrollComplement */ }
            ComplementKind::FreightTransport(cp)  => { /* FreightTransportComplement */ }
            ComplementKind::Unknown { namespace, raw_xml } => { /* unrecognized */ }
        }
    }
}
```

Or use the typed convenience methods on `Invoice` (see above).

### `FiscalStamp`

| Field | Description |
|---|---|
| `uuid` | Folio fiscal UUID |
| `stamp_date` | SAT stamp datetime |
| `sat_certificate_number` | SAT certificate number |
| `cfdi_seal` | CFDI digital seal |
| `sat_seal` | SAT digital seal |

---

## Catalog enums

All catalog enums serialize to their SAT code string (e.g. `"PUE"`, `"MXN"`, `"I"`). Unknown codes deserialize to `Unknown(String)` for forward compatibility.

### `DocumentType` — c_TipoDeComprobante

| Variant | Code |
|---|---|
| `Income` | `I` |
| `Expense` | `E` |
| `Transfer` | `T` |
| `Payment` | `P` |
| `Payroll` | `N` |

### `PaymentForm` — c_FormaPago

| Variant | Code |
|---|---|
| `Cash` | `01` |
| `NominativeCheck` | `02` |
| `ElectronicTransfer` | `03` |
| `CreditCard` | `04` |
| `DebitCard` | `28` |
| `ToBeDefined` | `99` |
| *(and 18 more — see `catalogs.rs`)* | |

### `PaymentMethod` — c_MetodoPago

| Variant | Code | Meaning |
|---|---|---|
| `SinglePayment` | `PUE` | Paid in full |
| `Installments` | `PPD` | Paid in parts |

### `TaxType` — c_Impuesto

| Variant | Code |
|---|---|
| `Isr` | `001` |
| `Iva` | `002` |
| `Ieps` | `003` |

### `TaxFactor` — c_TipoFactor

| Variant | Code |
|---|---|
| `Rate` | `Tasa` |
| `Amount` | `Cuota` |
| `Exempt` | `Exento` |

### `Currency` — c_Moneda

Common codes: `Mxn` → `"MXN"`, `Usd` → `"USD"`, `Eur` → `"EUR"`. See `catalogs.rs` for the full list.

### Other catalogs

`FiscalRegime`, `CfdiUse`, `TaxObject`, `ExportType`, `RelationType`, `PayrollPeriodicity`, `ContractType`, `PaymentRegime`, `InvoicePeriodicity` — all follow the same pattern.

---

## Error handling

```rust
use sat_cfdi::CfdiError;

match sat_cfdi::parse(xml) {
    Ok(invoice) => { /* use invoice */ }
    Err(CfdiError::NotCfdi) => eprintln!("File is HTML, not a CFDI"),
    Err(CfdiError::Xml(e))  => eprintln!("XML parse error: {}", e),
}
```

| Variant | Cause |
|---|---|
| `CfdiError::Xml(quick_xml::DeError)` | Malformed XML or missing required field |
| `CfdiError::NotCfdi` | File starts with `<!DOCTYPE html` or `<html` |

---

## Date helpers

Monetary amounts and dates are kept as `String` to avoid precision loss. Use these helpers to parse them:

```rust
use sat_cfdi::{parse_cfdi_datetime, parse_cfdi_date};

// "2024-01-15T12:00:00" → NaiveDateTime
let dt = parse_cfdi_datetime(&invoice.issued_at)?;

// "2024-01-15" → NaiveDate
let d = parse_cfdi_date("2024-01-15")?;
```

---

## CFDI version compatibility

- **CFDI 3.3**: `export`, `recipient.fiscal_domicile`, `recipient.fiscal_regime`, and `line_item.tax_object` are absent — all parse as `None`.
- **CFDI 4.0**: all fields present.

No version check is needed; the parser handles both transparently.

---

## Parsing a directory (NDJSON)

```bash
sat-cli parse ~/sat-cli/documents/RFC123456789/
```

Outputs one compact JSON object per line. Single-file mode outputs pretty-printed JSON unless `--compact` is passed.

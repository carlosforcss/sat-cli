pub const LOGIN_URL: &str = "https://portalcfdi.facturaelectronica.sat.gob.mx/";
pub const ISSUED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx";
pub const RECEIVED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaReceptor.aspx";

pub const ISSUED_AT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
pub const MX_DATE_FORMAT: &str = "%d/%m/%Y";

pub const DOCUMENTS_ENV_VAR: &str = "SATCLI_DOCUMENTS_FOLDER";
pub const DEFAULT_DOCUMENTS_FOLDER: &str = "sat-cli/documents/";

pub const FILTER_START_YEAR: i32 = 2020;

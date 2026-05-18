pub const ISSUED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx";
pub const LOGIN_URL: &str = "https://portalcfdi.facturaelectronica.sat.gob.mx/";
pub const RECEIVED_INVOICES_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaReceptor.aspx";

pub const ISSUED_AT_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
pub const MX_DATE_FORMAT: &str = "%d/%m/%Y";

pub const DOCUMENTS_ENV_VAR: &str = "SATCLI_DOCUMENTS_FOLDER";
pub const DEFAULT_DOCUMENTS_FOLDER: &str = "sat-cli/documents/";

pub const FILTER_START_YEAR: i32 = 2020;

pub const SAT_PORTAL_BASE_URL: &str = "https://portalcfdi.facturaelectronica.sat.gob.mx";

pub const VALIDATE_DOWNLOAD_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx/ValidarDescarga";
pub const RECOVER_CFDI_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/RecuperaCfdi.aspx";
pub const RECOVER_RI_TOKEN_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/ConsultaEmisor.aspx/RecuperaRepresentacionImpresa";
pub const RECOVER_RI_URL: &str =
    "https://portalcfdi.facturaelectronica.sat.gob.mx/RepresentacionImpresa.aspx";

pub const CSF_LOGIN_URL: &str = "https://login.siat.sat.gob.mx/nidp/app/login";
pub const CSF_FIEL_LOGIN_URL: &str =
    "https://login.siat.sat.gob.mx/nidp/idff/sso?id=fiel&sid=0&option=credential&sid=0";
pub const CSF_SSO_ENTRY_URL: &str = "https://wwwmat.sat.gob.mx/app/seg/faces/pages/lanzador.jsf\
     ?url=/operacion/53027/genera-tu-constancia-de-situacion-fiscal\
     &tipoLogeo=c&target=principal&hostServer=https://wwwmat.sat.gob.mx";
/// Page that contains formReimpAcuse with the download button
pub const CSF_CERTIFICATE_URL: &str =
    "https://rfcampc.siat.sat.gob.mx/PTSC/IdcSiat/autc/ReimpresionTramite/ConsultaTramite.jsf";
pub const CSF_PDF_URL: &str =
    "https://rfcampc.siat.sat.gob.mx/PTSC/IdcSiat/IdcGeneraConstancia.jsf";

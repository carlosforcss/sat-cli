use thiserror::Error;

#[derive(Debug, Error)]
pub enum CfdiError {
    #[error("XML deserialization error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("File is not a CFDI document (HTML content)")]
    NotCfdi,
}

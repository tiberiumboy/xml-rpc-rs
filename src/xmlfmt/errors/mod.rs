pub mod xml_error;
pub mod fmt_error;

pub use xml_error::{XmlError, on_decode_fail, on_encode_fail};
pub use fmt_error::FmtError;
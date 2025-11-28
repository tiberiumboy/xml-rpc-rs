#![allow(unknown_lints, unused_doc_comments)]
// pub use super::xmlfmt::error::{Error as FmtError, ErrorKind as FmtErrorKind};
use serde::{Deserialize, de, ser, Serialize};
use std::fmt;

pub type Result<T> = std::result::Result<T, XmlError>;

// TODO: Find another way to handle decode/encode without string arguments.
#[derive(Debug, Serialize, Deserialize)]
pub enum XmlError {
    Format(FmtError),
    Http(String), // TODO: force type to ureq::Error, but had issue. Replace to see compiler errors.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FmtError {
    Decoding(String),
    Encoding(String),
    UnsupportedFormat(String),
}

impl fmt::Display for FmtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FmtError::Decoding(t) => write!(f, "Issue while decoding data structure: {}", t),
            FmtError::Encoding(t) => write!(f, "Issue while encoding data structure: {}", t),
            FmtError::UnsupportedFormat(t) => write!(f, "Given structure is not supported: {}", t),
        }
    }
}


// ser+de complains that XmlError does not implement std::fmt::Display + Debug?
impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Format(e) => write!(f, "Format error: {}", e),
            XmlError::Http(t) => write!(f, "HTTP error: {}", t),
        }
    }
}

impl std::error::Error for FmtError {}

impl std::error::Error for XmlError {}

impl ser::Error for XmlError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        XmlError::Format(FmtError::Encoding(format!("{}", msg)))
    }
}

impl de::Error for XmlError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        XmlError::Format(FmtError::Decoding(format!("{}", msg)))
    }
}


// error_chain! {
//     foreign_links {
//         Fmt(fmt::Error);
//     }

//     errors {
//         Decoding(t: String) {
//             description("Issue while decoding data structure")
//             display("Issue while decoding data structure: {}", t)
//         }
//         Encoding(t: String) {
//             description("Issue while encoding data structure")
//             display("Issue while encoding data structure: {}", t)
//         }
//         UnsupportedData(t: String) {
//             description("Given structure is not supported")
//             display("Given structure is not supported: {}", t)
//         }
//     }
// }
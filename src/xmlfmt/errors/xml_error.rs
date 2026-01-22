use crate::xmlfmt::FmtError;
use serde::{Deserialize, Serialize, de, ser};
use std::fmt;

// TODO: Find another way to handle decode/encode without string arguments.
#[derive(Debug, Serialize, Deserialize)]
pub enum XmlError {
    Format(FmtError),
    Server(String), // TODO: Handle errors from tiny_http instead
    Http(String), // TODO: force type to ureq::Error, but had issue. Replace to see compiler errors.
}

impl std::error::Error for XmlError {}

// ser+de complains that XmlError does not implement std::fmt::Display + Debug?
impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::Format(e) => write!(f, "Format error: {}", e),
            XmlError::Http(t) => write!(f, "HTTP error: {}", t),
            XmlError::Server(s) => write!(f, "Server error: {}", s),
        }
    }
}

// Feature: how do I only include this implementation if the user request serde features?
// #[cfg(serde)]
impl ser::Error for XmlError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        XmlError::Format(FmtError::Encoding(format!("{}", msg)))
    }
}

// #[cfg(serde)]
impl de::Error for XmlError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        XmlError::Format(FmtError::Decoding(format!("{}", msg)))
    }
}

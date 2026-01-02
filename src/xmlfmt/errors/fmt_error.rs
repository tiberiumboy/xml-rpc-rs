use serde::{Deserialize, Serialize};
use std::fmt;

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

impl std::error::Error for FmtError {}

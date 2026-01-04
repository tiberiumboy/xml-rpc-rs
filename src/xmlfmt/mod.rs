use std::result::Result;
use serde::Serialize;

pub mod errors; // rust errors (local)
pub mod params; // method response/call
// DO not use parser - read the codeblock section near xmlvalue impl. @L94
// pub mod parse;
pub mod value;  // value type
mod call;
mod response;
mod member;
mod parse;
mod data;

pub(crate) mod ser;    // I need to bring this method back so that I can parse params correctly from any Deserialize input on client/server side.
pub(crate) mod de;
pub(crate) mod to_xml; // may not be needed?

pub use crate::xmlfmt::params::{Params, Param};
pub use crate::xmlfmt::value::Value;
pub use self::errors::{XmlError, FmtError, on_decode_fail, on_encode_fail};
pub use self::response::XmlResponse;
// scope project only - used for client
pub(crate) use self::to_xml::ToXml; 
pub(crate) use self::response::MethodResponse;
use self::member::Member;
use self::data::Data;

// interface point for making http request
pub(crate) use self::call::Call;

pub type XmlResult<T> = Result<T, XmlError>;

pub fn into_params<T: Serialize>(v: &T) -> XmlResult<Params> {
    let content = match v.serialize(ser::Serializer {}) ? {
        Value::Array(param) => Into::into(*param),
        value => vec![value],
    };
    Ok(Params::new(content))
}
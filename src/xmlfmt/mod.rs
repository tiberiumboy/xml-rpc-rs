use std::result::Result;

pub mod errors; // rust errors (local)
pub mod params; // method response/call
// DO not use parser - read the codeblock section near xmlvalue impl. @L94
// pub mod parse;
mod call;
mod data;
mod member;
mod parse;
mod response;
pub mod value; // value type

pub(crate) mod to_xml; // may not be needed?

pub use self::errors::{FmtError, XmlError};
pub use self::response::XmlResponse;
pub use crate::xmlfmt::params::{Param, Params};
pub use crate::xmlfmt::value::Value;
// scope project only - used for client
use self::data::Data;
use self::member::Member;
pub(crate) use self::response::MethodResponse;
pub(crate) use self::to_xml::ToXml;

// interface point for making http request
pub(crate) use self::call::Call;

pub type XmlResult<T> = Result<T, XmlError>;

// pub fn into_params<T: Serialize>(v: &T) -> XmlResult<Params> {
//     let content = match v.serialize(ser::Serializer {})? {
//         Value::Array(param) => Into::into(*param),
//         value => vec![value],
//     };
//     Ok(Params::new(content))
// }

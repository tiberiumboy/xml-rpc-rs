use std::result::Result;

mod call;
mod data;
pub mod errors; // rust errors (local)
mod member;
pub mod params; // method response/call
pub mod response;
pub mod value; // value type

pub(crate) mod to_xml; // may not be needed?

pub use self::errors::{FmtError, XmlError};
pub use self::response::{MethodResponse, XmlResponse};
pub use crate::xmlfmt::params::{Param, Params};
pub use crate::xmlfmt::value::Value;
// must be public to support value schema
pub use self::data::Data;
pub use self::member::Member;
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

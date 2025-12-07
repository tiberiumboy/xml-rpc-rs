use serde::{Deserialize, Serialize};

// TODO: How do I re-export this from this mod?
pub mod alias;

mod de;
pub mod error;
pub mod fault;
pub mod parse;
mod ser;
// TODO: Why do I need this here?
pub mod call;
#[cfg(test)]
mod tests;
pub mod to_xml;
pub mod value;
// pub mod xml; // FIXME: Doesn't seems to have a lot of info? It's being used for unit test.

pub use self::alias::{Params, XmlResult};
pub use self::call::Call;
pub use self::fault::Fault;
pub use self::value::Value;

pub fn from_params<'a, T: Deserialize<'a>>(mut params: Params) -> error::Result<T> {
    let data = if params.len() == 1 {
        params.pop().unwrap()
    } else {
        Value::Array(params)
    };

    T::deserialize(data).map_err(|e| {
        error::XmlError::Format(error::FmtError::Decoding(format!(
            "Failed to convert XML-RPC to structure. {}",
            e
        )))
    })
}

pub fn into_params<T: Serialize>(v: &T) -> error::Result<Params> {
    Ok(match v.serialize(ser::Serializer {})? {
        Value::Array(params) => params,
        data => vec![data],
    })
}

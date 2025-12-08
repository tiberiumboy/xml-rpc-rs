use serde::{Deserialize, Serialize};

// TODO: How do I re-export as alias?
pub mod alias;

pub mod call;
mod de;
pub mod error;
pub mod fault;
pub mod parse;
mod ser;
pub mod to_xml;
pub mod value;

// pub mod xml; // FIXME: Doesn't seems to have a lot of info? It's being used for unit test.

pub use self::call::Call;
pub use self::fault::Fault;
pub use self::value::{Params, Value};

pub type XmlResult = Result<Params, Fault>;

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

#[cfg(test)]
pub mod tests {
    use super::*;

    pub static BAD_DATA: &str = "Bad data provided";

    pub fn ser_and_de_response_value(value: XmlResult) {
        use crate::xmlfmt::to_xml::ToXml;
        let data = value.to_xml();
        let data = parse::response(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(value, data);
    }

    pub fn ser_and_de(value: Value) {
        ser_and_de_response_value(Ok(vec![value]));
    }

    pub fn ser_and_de_call_value(value: Call) {
        use crate::xmlfmt::to_xml::ToXml;
        let data = value.to_xml();
        let data = parse::call(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(value, data);
    }
}

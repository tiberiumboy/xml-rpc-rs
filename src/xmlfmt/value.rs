use crate::xmlfmt::{Data, Member, Param};
use serde::de::Unexpected;
use serde::{Deserialize, Serialize};

// TODO: Does serde_xml_rs handle box pointers? I'd like to run unit test on this one.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Value {
    I4(i32), // What's the difference between this and the latter? According to XmlValue, there's a distinguish between two?
    // officially declared in docs as A signed, 32-bit integer.
    Int(i32),
    Bool(bool),

    // ASCII string, may contain NULL bytes, supports Unicode.
    String(String),

    // Double precision floating point number.
    Double(f64),

    #[deprecated = "XML-RPC forbids use of timezones, use another method instead."]
    #[serde(rename = "dateTime.iso8601")]
    DateTime(String),

    // Raw binary data of any length; encoded using Base64 on the wire.
    Base64(Vec<u8>),

    // An one-dimensional array of values. Individual values may be of any type.
    Array(Box<Data>),

    // A collection of key-value pairs. The keys are strings; the values may be of any type.
    Struct {
        member: Box<Vec<Member>>,
    },
    Nil, // translate this into <nil/>
}

// This is considered as a "DataType"
impl Value {
    pub fn unexpected(&self) -> Unexpected<'_> {
        match *self {
            Value::I4(v) => Unexpected::Signed(i64::from(v)),
            Value::Int(v) => Unexpected::Signed(i64::from(v)),
            Value::Bool(v) => Unexpected::Bool(v),
            Value::String(ref v) => Unexpected::Str(v),
            Value::Double(v) => Unexpected::Float(v),
            // This is fine, we've deprecated the usage of the method, but this library needs to handle it gracefully.
            // FIXME: How can I suppress this lint warning?
            Value::DateTime(_) => Unexpected::Other("dateTime.iso8601"),
            Value::Base64(ref v) => Unexpected::Bytes(v),
            Value::Array(_) => Unexpected::Seq,
            Value::Struct { .. } => Unexpected::Map,
            Value::Nil => Unexpected::Unit,
        }
    }

    pub fn fault<T>(code: i32, message: T) -> Value
    where
        T: Into<String>,
    {
        let members = vec![
            Member::new("faultCode".to_owned(), Value::Int(code)),
            Member::new("faultString".to_owned(), Value::String(message.into())),
        ];
        Value::Struct {
            member: Box::new(members),
        }
    }

    pub fn to_array(values: Param) -> Value {
        let data = Data::new(values);
        Value::Array(Box::new(data))
    }

    pub fn to_struct(members: Vec<Member>) -> Value {
        Value::Struct {
            member: Box::new(members),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xmlfmt::Params;
    use crate::xmlfmt::response::{MethodResponse, tests::*};

    pub fn ser_and_de(value: Value) {
        let params = Params::new(vec![value]);
        ser_and_de_response_value(MethodResponse::Params(params));
    }

    #[test]
    fn writes_pod_xml_value() {
        ser_and_de(Value::String("South Dakota".into()));
        ser_and_de(Value::String("".into()));
        ser_and_de(Value::Int(-33));
        ser_and_de(Value::Bool(true));
        ser_and_de(Value::Bool(false));
        ser_and_de(Value::Double(-44.2));
        // Value::DateTime will be phased out, so no unit test will be ran against this.
        // ser_and_de(Value::DateTime("33".into()));
        ser_and_de(Value::Base64("ASDF=".into()));
    }

    #[test]
    fn writes_array_xml_value() {
        let values = vec![
            Value::Int(33),
            Value::Int(-12),
            Value::String("ASDF=".into()),
        ];
        let params = Data::new(values);
        let array = Value::Array(Box::new(params));
        ser_and_de(array);
    }

    #[test]
    fn writes_struct_xml_value() {
        let data = vec![
            Member::new("foo".to_owned(), Value::Int(42)),
            Member::new("bar".to_owned(), Value::String("baz".into())),
        ];

        let value = Value::Struct {
            member: Box::new(data),
        };
        ser_and_de(value);
    }
}

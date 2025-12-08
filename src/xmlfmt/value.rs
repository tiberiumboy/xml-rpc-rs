use serde::de::Unexpected;
use std::collections::HashMap;

pub type Params = Vec<Value>;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    DateTime(String),
    Base64(Vec<u8>),
    Array(Vec<Value>),
    Struct(HashMap<String, Value>),
}

impl Value {
    pub fn unexpected(&'_ self) -> Unexpected<'_> {
        match self {
            Value::Int(v) => Unexpected::Signed(i64::from(*v)),
            Value::Bool(v) => Unexpected::Bool(*v),
            Value::String(v) => Unexpected::Str(v),
            Value::Double(v) => Unexpected::Float(*v),
            Value::DateTime(_) => Unexpected::Other("dateTime.iso8601"),
            Value::Base64(v) => Unexpected::Bytes(v),
            Value::Array(_) => Unexpected::Seq,
            Value::Struct(_) => Unexpected::Map,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xmlfmt::tests::*;

    #[test]
    fn writes_pod_xml_value() {
        ser_and_de(Value::String("South Dakota".into()));
        ser_and_de(Value::String("".into()));
        ser_and_de(Value::String("".into()));
        ser_and_de(Value::Int(-33));
        ser_and_de(Value::Int(-33));
        ser_and_de(Value::Bool(true));
        ser_and_de(Value::Bool(false));
        ser_and_de(Value::Double(-44.2));
        ser_and_de(Value::DateTime("33".into()));
        ser_and_de(Value::Base64("ASDF=".into()));
    }

    #[test]
    fn writes_array_xml_value() {
        ser_and_de(Value::Array(vec![
            Value::Int(33),
            Value::Int(-12),
            Value::Int(44),
        ]));
    }

    #[test]
    fn writes_struct_xml_value() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        ser_and_de(Value::Struct(fields));
    }

    #[test]
    fn writes_response() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        let params = vec![Value::String("South Dakota".into()), Value::Struct(fields)];
        ser_and_de_response_value(Ok(params))
    }
}

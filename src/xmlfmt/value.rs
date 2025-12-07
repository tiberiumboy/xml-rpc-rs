use serde::de::Unexpected;
use std::collections::HashMap;

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

use serde::de::Unexpected;
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::{Serialize, Deserialize};
use serde::de::Visitor;
use std::collections::HashMap;
use serde_xml_rs::Deserializer;
use crate::xmlfmt::{Data, FmtError, Member, Param, XmlError, XmlResult, into_params};
use crate::xmlfmt::de::handle_integer;

// TODO: Does serde_xml_rs handle box pointers? I'd like to run unit test on this one.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all="camelCase")]
pub enum Value {
    I4(i32), // What's the difference between this and the latter? According to XmlValue, there's a distinguish between two?
    Int(i32),
    Bool(bool),
    String(String),
    Double(f64),
    #[serde(rename="dateTime.iso8601")]
    // TODO: See if we can store DateTime true value instead of string, see if the serde_xml_rs parser parse it correctly?
    DateTime(String),   // TODO: figure out how to parse this into ISO 8601 format and back
    Base64(Vec<u8>),
    // Using box pointer to avoid infinite recursive loop of Array -> Data -> value -> Array...
    Array(Box<Data>),
    // Using box pointer to avoid infinite recursive loop of Value -> Struct -> Member -> value
    Struct { 
        member: Box<Vec<Member>> 
    },    
    Nil,    // translate this into <nil/>
}
    
// This is considered as a "DataType"
impl Value {
    
    pub fn unexpected(&self) -> Unexpected {
        match *self {
            Value::I4(v) => Unexpected::Signed(i64::from(v)),
            Value::Int(v) => Unexpected::Signed(i64::from(v)),
            Value::Bool(v) => Unexpected::Bool(v),
            Value::String(ref v) => Unexpected::Str(v),
            Value::Double(v) => Unexpected::Float(v),
            Value::DateTime(_) => Unexpected::Other("dateTime.iso8601"),
            Value::Base64(ref v) => Unexpected::Bytes(v),
            Value::Array(_) => Unexpected::Seq,
            Value::Struct{ .. } => Unexpected::Map,
            Value::Nil => Unexpected::Unit,
        }
    }

    pub fn fault<T>(code: i32, message: T ) -> Value
    where T: Into<String> {
        let members = vec![
            Member::new("faultCode".to_owned(), Value::Int(code)),
            Member::new("faultString".to_owned(), Value::String(message.into()))
        ];
        Value::Struct { member: Box::new(members) }
    }

    pub fn to_array(values: Param) -> Value {
        let data = Data::new(values);
        Value::Array(Box::new(data))
    }

    pub fn to_struct(members: Vec<Member>) -> Value {
        Value::Struct { member: Box::new(members)}
    }

}

impl<'de> serde::Deserializer<'de> for Value {
    type Error = XmlError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::I4(v) | Value::Int(v) => visitor.visit_i32(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::DateTime(v) | Value::String(v) => visitor.visit_string(v),
            Value::Double(v) => visitor.visit_f64(v),
            Value::Base64(v) => visitor.visit_bytes(v.as_slice()),
            Value::Array(v) => {

                
                let len = Into::into(*v).len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut deserializer)?;
                // FIXME: this seems fishy?
                let remaining = deserializer.iter.len();    
                if remaining == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in array",
                    ))
                }

            }
            Value::Struct{ member } => {
                let len = member.len();
                let mut deserializer = MapDeserializer::new(member);
                let map = visitor.visit_map(&mut deserializer)?;
                let remaining = deserializer.iter.len();
                if remaining == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"fewer elements in map",
                    ))
                }
            }
            Value::Nil => visitor.visit_none()
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::String(v) => match v.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err( 
                    XmlError::Format(
                        FmtError::Decoding(
                            format!("Cannot deserialize into bool from \"{v}\"")))
                        ),
            },
            _ => Err(
                XmlError::Format(
                    FmtError::UnsupportedFormat(
                        format!("{self:?} value not supported for deserializing boolean!").to_owned()
                    ))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f32(v as f32),
            Value::String(v) => {
                let x: XmlResult<f32> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f32(x?)
            }
            _ => Err(XmlError::Format(FmtError::UnsupportedFormat(self.unexpected()))),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f64(v),
            Value::String(v) => {
                let x: XmlResult<f64> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f64(x?)
            }
            Value::Int(v) => visitor.visit_f64(v as f64),
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            if v.len() != 1 {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Str(&v),
                    &"string with a single character",
                ));
            }
            visitor.visit_char(v.chars().next().unwrap())
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    // TODO: What's the difference between this and deserialize_string?
    fn deserialize_str<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_str(&v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_string(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_bytes(v.as_slice())
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_byte_buf(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(mut v) => {
                let v1 = v.pop();
                if !v.is_empty() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Seq,
                        &"array with a single element",
                    ));
                }
                match v1 {
                    Some(x) => visitor.visit_some(x),
                    None => visitor.visit_none(),
                }
            }

            v => visitor.visit_some(v),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Struct(v) = self {
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"empty map",
                ));
            }
            visitor.visit_unit()
        } else {
            Err(serde::de::Error::invalid_value(
                self.unexpected(),
                &"empty map",
            ))
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
    

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Struct(members) => {
                let mut member_iter = members.into_iter();
                if let Some((key, value)) = member_iter.next() {
                    if member_iter.next().is_none() {
                        return visitor.visit_enum(EnumDeserializer {
                            variant: key,
                            value,
                        });
                    }
                }
                Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"map with a single key",
                ))
            }
            other => Err(serde::de::Error::invalid_value(
                other.unexpected(),
                &"map with a single key",
            )),
        }
    }

    
    forward_to_deserialize_any! {
        identifier ignored_any
    }
}

impl<'se, T: Into<String>, S: Serialize> Into<Value> for HashMap<T, S> 
    {
        fn into(self) -> Value {
            let members = self.iter().fold(
                Vec::with_capacity(self.len()), 
                |mut list, (key, val)| {                    
                    match into_params(val) {
                        Ok(param)=> {
                            let value = Member::new(key.into(), );
                            list.push(value);
                        },
                        Err(e) => {
                            println!("Fail to insert values to table! Should not happen but unit test should catch this!");
                        }
                    }
                    list
            });
            Value::Struct{ member: Box::new(members) }
        }
    }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xmlfmt::response::{MethodResponse, tests::*};
    use crate::xmlfmt::Params;

    pub fn ser_and_de(value: Value) {
        let params = Params::new( vec![value] );
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
        ser_and_de(Value::DateTime("33".into()));
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
            Member::new("bar".to_owned(), Value::String("baz".into()))
        ];
        
        let value = Value::Struct { member: Box::new(data) };
        ser_and_de(value);
    }
}

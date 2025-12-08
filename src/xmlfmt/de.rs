use crate::xmlfmt::Value;
use crate::xmlfmt::error::{Result, XmlError};
use serde::de::{
    DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::{self, Deserializer};
use std;
use std::collections::HashMap;
use std::vec;

impl<'de> serde::Deserializer<'de> for Value {
    type Error = XmlError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Int(v) => visitor.visit_i32(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::DateTime(v) | Value::String(v) => visitor.visit_string(v),
            Value::Double(v) => visitor.visit_f64(v),
            Value::Base64(v) => visitor.visit_bytes(v.as_slice()),
            Value::Array(v) => {
                let len = v.len();
                let mut deserializer = SeqDeserializer::new(v);
                let seq = visitor.visit_seq(&mut deserializer)?;
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
            Value::Struct(v) => {
                let len = v.len();
                let mut deserializer = MapDeserializer::new(v);
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
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            Value::String(v) => match v.as_str() {
                "true" => visitor.visit_bool(true),
                "false" => visitor.visit_bool(false),
                _ => Err(serde::de::Error::invalid_value(
                    Unexpected::Str(&v),
                    &visitor,
                )),
            },
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i8(v)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i16(v)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_i64(v)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u8(v)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u16(v)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u32(v)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let v = handle_integer(self, &visitor)?;
        visitor.visit_u64(v)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f32(v as f32),
            Value::String(v) => {
                let x: Result<f32> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f32(x?)
            }
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Double(v) => visitor.visit_f64(v),
            Value::String(v) => {
                let x: Result<f64> = v
                    .parse()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), &visitor));
                visitor.visit_f64(x?)
            }
            Value::Int(v) => visitor.visit_f64(v as f64),
            _ => Err(serde::de::Error::invalid_value(self.unexpected(), &visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
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

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_str(&v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::String(v) = self {
            visitor.visit_string(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_bytes(v.as_slice())
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Value::Base64(v) = self {
            visitor.visit_byte_buf(v)
        } else {
            Err(serde::de::Error::invalid_value(self.unexpected(), &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
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

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
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

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
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
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
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
    ) -> Result<V::Value>
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
    ) -> Result<V::Value>
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

struct SeqDeserializer {
    iter: vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for SeqDeserializer {
    type Error = XmlError;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        let ret = visitor.visit_seq(&mut self)?;
        let remaining = self.iter.len();
        if remaining == 0 {
            Ok(ret)
        } else {
            Err(serde::de::Error::invalid_length(
                len,
                &"fewer elements in array",
            ))
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = XmlError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct MapDeserializer {
    iter: <HashMap<String, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: HashMap<String, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = XmlError;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
    type Error = XmlError;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct EnumDeserializer {
    variant: String,
    value: Value,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = XmlError;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Value)>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value;
        let variant = Value::String(self.variant);
        seed.deserialize(variant).map(|v| (v, value))
    }
}

impl<'de> VariantAccess<'de> for Value {
    type Error = XmlError;

    fn unit_variant(self) -> Result<()> {
        if let Value::Struct(v) = self {
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"empty map",
                ));
            }
            Ok(())
        } else {
            Err(serde::de::Error::invalid_value(
                self.unexpected(),
                &"empty map",
            ))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", fields, visitor)
    }
}

trait FromI32 {
    fn from_i32(v: i32) -> Self;
}

macro_rules! impl_from_i32 {
    ($($ty:ty)*) => {
        $(
            impl FromI32 for $ty {
                #[inline]
                fn from_i32(v: i32) -> $ty {
                    v as $ty
                }
            }
        )*
    }
}

impl_from_i32!(u8 u16 u32 u64 i8 i16 i32);

impl FromI32 for i64 {
    #[inline]
    fn from_i32(v: i32) -> i64 {
        v.into()
    }
}

fn handle_integer<'de, T, V>(value: Value, visitor: &V) -> Result<T>
where
    T: FromI32 + std::str::FromStr,
    V: Visitor<'de>,
{
    match value {
        Value::Int(v) => Ok(T::from_i32(v)),
        Value::String(v) => v
            .parse()
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), visitor)),
        _ => Err(serde::de::Error::invalid_value(value.unexpected(), visitor)),
    }
}

#[cfg(test)]
mod tests {
    use crate::xmlfmt::Value;
    use serde::Deserialize;
    use serde_bytes;
    use std::collections::HashMap;

    #[test]
    fn reads_bool() {
        assert!(bool::deserialize(Value::Bool(true)).unwrap());
        assert!(!bool::deserialize(Value::Bool(false)).unwrap());
    }

    #[test]
    fn reads_integers_as_ints_or_strings_if_too_big() {
        assert_eq!(200u8, u8::deserialize(Value::Int(200)).unwrap());
        assert_eq!(42_000u16, u16::deserialize(Value::Int(42_000)).unwrap());
        assert_eq!(
            4_200_000_000u32,
            u32::deserialize(Value::String("4200000000".into())).unwrap()
        );
        assert_eq!(
            10_000_000_000_000_000_000u64,
            u64::deserialize(Value::String("10000000000000000000".into())).unwrap()
        );
        assert_eq!(-42_i8, i8::deserialize(Value::Int(-42)).unwrap());
        assert_eq!(-26_000_i16, i16::deserialize(Value::Int(-26_000)).unwrap());
        assert_eq!(
            -2_000_000_000_i32,
            i32::deserialize(Value::Int(-2000000000)).unwrap()
        );
        assert_eq!(
            -8_000_000_000_000_000_000_i64,
            i64::deserialize(Value::String("-8000000000000000000".into())).unwrap()
        );
        assert_eq!(42i8, i8::deserialize(Value::Int(42)).unwrap());
        assert_eq!(26_000i16, i16::deserialize(Value::Int(26_000)).unwrap());
        assert_eq!(
            2_000_000_000i32,
            i32::deserialize(Value::Int(2000000000)).unwrap()
        );
        assert_eq!(
            8_000_000_000_000_000_000i64,
            i64::deserialize(Value::String("8000000000000000000".into())).unwrap()
        );
    }

    #[test]
    fn reads_floats() {
        assert_eq!(3.25f32, f32::deserialize(Value::Double(3.25f64)).unwrap());
        assert_eq!(3.25f64, f64::deserialize(Value::Double(3.25f64)).unwrap());
    }

    #[test]
    fn reads_chars_as_strings() {
        assert_eq!('A', char::deserialize(Value::String("A".into())).unwrap());
        assert_eq!(' ', char::deserialize(Value::String(" ".into())).unwrap());
    }

    #[test]
    fn reads_strings() {
        assert_eq!(
            String::from("string object"),
            String::deserialize(Value::String("string object".into())).unwrap()
        );
    }

    #[test]
    fn reads_bytes_as_base64() {
        let data: Vec<u8> = serde_bytes::deserialize(Value::Base64(vec![48, 49, 50, 51])).unwrap();
        assert_eq!(data, vec![48, 49, 50, 51]);
    }

    #[test]
    fn reads_options_as_one_elem_or_empty_array() {
        let none: Option<i32> = None;
        assert_eq!(none, Option::deserialize(Value::Array(Vec::new())).unwrap());
        assert_eq!(
            Some(33i32),
            Option::deserialize(Value::Array(vec![Value::Int(33)])).unwrap()
        );
        assert_eq!(
            Some(String::from("txt")),
            Option::deserialize(Value::Array(vec![Value::String("txt".into())])).unwrap()
        );
    }

    #[test]
    fn reads_units_as_empty_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Helper;

        assert_eq!(
            Helper,
            Helper::deserialize(Value::Struct(HashMap::new())).unwrap()
        );
    }

    #[test]
    fn reads_newtype_struct_as_its_content() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct HelperInt(i32);
        #[derive(Debug, Deserialize, PartialEq)]
        struct HelperString(String);

        assert_eq!(
            HelperInt(33),
            HelperInt::deserialize(Value::Int(33)).unwrap()
        );
        assert_eq!(
            HelperString("txt".into()),
            HelperString::deserialize(Value::String("txt".into())).unwrap()
        );
    }

    #[test]
    fn reads_vector_as_array() {
        assert_eq!(
            vec![33, 15, 44, 12],
            Vec::<usize>::deserialize(Value::Array(vec![
                Value::Int(33),
                Value::Int(15),
                Value::Int(44),
                Value::Int(12),
            ]))
            .unwrap()
        );
        assert_eq!(
            vec!['a', 'b', 'c', 'd'],
            Vec::<char>::deserialize(Value::Array(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
            ]))
            .unwrap()
        );
    }

    #[test]
    fn reads_tuple_as_array() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Helper(u8, u64, String, bool);

        assert_eq!(
            Helper(4, 1_000_000_000_000u64, "hello".into(), true),
            Helper::deserialize(Value::Array(vec![
                Value::Int(4),
                Value::String("1000000000000".into()),
                Value::String("hello".into()),
                Value::Bool(true),
            ]))
            .unwrap()
        );
    }

    #[test]
    fn reads_struct_as_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Helper {
            foo: u8,
            bar: u64,
            baz: String,
            qux: bool,
        }

        let mut members = HashMap::new();
        members.insert("foo".into(), Value::Int(4));
        members.insert("bar".into(), Value::String("1000000000000".into()));
        members.insert("baz".into(), Value::String("hello".into()));
        members.insert("qux".into(), Value::Bool(true));

        assert_eq!(
            Helper {
                foo: 4,
                bar: 1_000_000_000_000u64,
                baz: "hello".into(),
                qux: true,
            },
            Helper::deserialize(Value::Struct(members)).unwrap()
        );
    }

    #[test]
    fn reads_map_as_struct() {
        let mut data = HashMap::new();
        data.insert(String::from("foo"), vec![44i8, 12]);
        data.insert(String::from("bar"), vec![]);
        data.insert(String::from("baz"), vec![-3, 44, 28]);

        let mut members = HashMap::new();
        members.insert(
            "foo".into(),
            Value::Array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("bar".into(), Value::Array(vec![]));
        members.insert(
            "baz".into(),
            Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
    }

    #[test]
    fn map_accepts_string_keys() {
        let mut data = HashMap::new();
        data.insert(String::from("foo"), vec![44i8, 12]);
        data.insert(String::from("bar"), vec![]);
        data.insert(String::from("baz"), vec![-3, 44, 28]);

        let mut members = HashMap::new();
        members.insert(
            "foo".into(),
            Value::Array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("bar".into(), Value::Array(vec![]));
        members.insert(
            "baz".into(),
            Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
    }

    #[test]
    fn map_accepts_integer_keys() {
        let mut data = HashMap::new();
        data.insert(12, vec![44i8, 12]);
        data.insert(-33, vec![]);
        data.insert(44, vec![-3, 44, 28]);

        let mut members = HashMap::new();
        members.insert(
            "12".into(),
            Value::Array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("-33".into(), Value::Array(vec![]));
        members.insert(
            "44".into(),
            Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
    }

    #[test]
    fn map_accepts_char_keys() {
        let mut data = HashMap::new();
        data.insert('a', vec![44i8, 12]);
        data.insert('b', vec![]);
        data.insert('c', vec![-3, 44, 28]);

        let mut members = HashMap::new();
        members.insert(
            "a".into(),
            Value::Array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("b".into(), Value::Array(vec![]));
        members.insert(
            "c".into(),
            Value::Array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
    }

    #[test]
    fn map_accepts_boolean_keys() {
        let mut data = HashMap::new();
        data.insert(true, vec![44i8, 12]);
        data.insert(false, vec![]);

        let mut members = HashMap::new();
        members.insert(
            "true".into(),
            Value::Array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("false".into(), Value::Array(vec![]));

        assert_eq!(data, HashMap::deserialize(Value::Struct(members)).unwrap());
    }

    #[test]
    fn reads_variant_as_one_member_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Helper {
            Foo,
            Bar(i32),
            Baz(bool, String),
            Qux { alpha: i32, beta: Vec<bool> },
        }

        let mut members = HashMap::new();
        members.insert("Foo".into(), Value::Struct(HashMap::new()));
        assert_eq!(
            Helper::Foo,
            Helper::deserialize(Value::Struct(members)).unwrap()
        );

        let mut members = HashMap::new();
        members.insert("Bar".into(), Value::Int(44));
        assert_eq!(
            Helper::Bar(44),
            Helper::deserialize(Value::Struct(members)).unwrap()
        );

        let mut members = HashMap::new();
        members.insert(
            "Baz".into(),
            Value::Array(vec![Value::Bool(false), Value::String("tsk".into())]),
        );
        assert_eq!(
            Helper::Baz(false, "tsk".into()),
            Helper::deserialize(Value::Struct(members)).unwrap()
        );

        let mut submembers = HashMap::new();
        submembers.insert("alpha".into(), Value::Int(-4));
        submembers.insert(
            "beta".into(),
            Value::Array(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ]),
        );

        let mut members = HashMap::new();
        members.insert("Qux".into(), Value::Struct(submembers));
        assert_eq!(
            Helper::Qux {
                alpha: -4,
                beta: vec![true, false, true],
            },
            Helper::deserialize(Value::Struct(members)).unwrap()
        );
    }
}

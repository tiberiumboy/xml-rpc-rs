use crate::xmlfmt::{XmlError, FmtError, XmlResult, Value};
use serde::de::{
    DeserializeSeed, Unexpected, VariantAccess, Visitor,
};
use serde::{self, Deserializer};
use std;

impl<'de> VariantAccess<'de> for Value {
    type Error = XmlError;

    fn unit_variant(self) -> XmlResult<()> {
        if let Value::Struct { member: v} = self {
            if !v.is_empty() {
                return Err(serde::de::Error::invalid_value(
                    Unexpected::Map,
                    &"empty map",
                ));
            }
            Ok(())
        } else {
            // TODO: figure out about the unexpected(), but otherwise.
            Err(XmlError::Format(super::FmtError::UnsupportedFormat("Empty map".to_owned())))
            // Err(serde::de::Error::invalid_value(
            //     self.unexpected(),
            //     &"empty map",
            // ))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> XmlResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> XmlResult<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_struct("", fields, visitor)
    }
}

pub(crate) trait FromI32 {
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

pub(crate) fn handle_integer<'de, T, V>(value: Value, visitor: &V) -> XmlResult<T>
where
    T: FromI32 + std::str::FromStr,
    V: Visitor<'de>,
{
    match value {
        Value::Int(v) => Ok(T::from_i32(v)),
        Value::String(v) => v
            .parse()
            .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(&v), visitor)),
        _ => Err(XmlError::Format(FmtError::Decoding("Unexpected".to_owned())))
            // _ => Err(serde::de::Error::invalid_value(value.unexpected(), visitor)),
    }
}

#[cfg(test)]
mod tests {
    use crate::xmlfmt::{Member, Value};
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
        assert_eq!(none, Option::deserialize(Value::to_array(Vec::new())).unwrap());
        assert_eq!(
            Some(33i32),
            Option::deserialize(Value::to_array(vec![Value::Int(33)])).unwrap()
        );
        assert_eq!(
            Some(String::from("txt")),
            Option::deserialize(Value::to_array(vec![Value::String("txt".into())])).unwrap()
        );
    }

    #[test]
    fn reads_units_as_empty_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Helper;

        assert_eq!(
            Helper,
            Helper::deserialize(Value::Struct{ member: Box::new(Vec::new())}).unwrap()
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
            Vec::<usize>::deserialize(Value::to_array(vec![
                Value::Int(33),
                Value::Int(15),
                Value::Int(44),
                Value::Int(12),
            ]))
            .unwrap()
        );
        assert_eq!(
            vec!['a', 'b', 'c', 'd'],
            Vec::<char>::deserialize(Value::to_array(vec![
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
            Helper::deserialize(Value::to_array(vec![
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

        let members = vec![
            Member::new("foo".to_owned(), Value::Int(4)),
            Member::new("bar".to_owned(), Value::String("1000000000000".into())),
            Member::new("baz".to_owned(), Value::String("hello".into())),
            Member::new("qux".to_owned(), Value::Bool(true)),
        ];

        assert_eq!(
            Helper {
                foo: 4,
                bar: 1_000_000_000_000u64,
                baz: "hello".into(),
                qux: true,
            },
            Helper::deserialize(Value::Struct{ member: Box::new(members) }).unwrap()
        );
    }

    #[test]
    fn reads_map_as_struct() {
        let data = vec![
            Member::new(String::from("foo"), Value::to_array(vec![Value::Int(44), Value::Int(12)])),
            Member::new(String::from("bar"), Value::to_array(vec![])),
            Member::new(String::from("baz"), Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)])),
        ];

        let members = vec![
            Member::new(
                "foo".to_owned(),
                Value::to_array(vec![Value::Int(44), Value::Int(12)]),
            ),
            Member::new("bar".to_owned(), Value::to_array(vec![])),
            Member::new(
                "baz".to_owned(),
                Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
            )
        ];

        assert_eq!(data, Vec::deserialize(Value::Struct{ member: Box::new(members) }).unwrap());
    }

    #[test]
    fn map_accepts_string_keys() {
        let mut data = HashMap::new();
        data.insert(String::from("foo"), vec![44i8, 12]);
        data.insert(String::from("bar"), vec![]);
        data.insert(String::from("baz"), vec![-3, 44, 28]);

        let members = vec![
            Member::new(
                "foo".to_string(),
                Value::to_array(vec![Value::Int(44), Value::Int(12)]),
            ),
            Member::new("bar".to_owned(), Value::to_array(vec![])),
            Member::new(
                "baz".to_owned(),
                Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
            )
        ];

        assert_eq!(data, HashMap::deserialize(Value::Struct{ member: Box::new(members)}).unwrap());
    }

    #[test]
    fn map_accepts_integer_keys() {
        let mut data = HashMap::new();
        data.insert("12".into(), vec![44i8, 12]);
        data.insert("-33".to_owned(), vec![]);
        data.insert("44".to_owned(), vec![-3, 44, 28]);

        let members = vec![
            Member::new(
                "12".to_owned(),
                Value::to_array(vec![Value::Int(44), Value::Int(12)]),
            ),
            Member::new("-33".to_owned(), Value::to_array(vec![])),
            Member::new(
                "44".to_owned(),
                Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
            ),
        ];

        assert_eq!(data, HashMap::deserialize(Value::Struct{ member: Box::new(members) }).unwrap());
    }

    #[test]
    fn map_accepts_char_keys() {
        let mut data = HashMap::new();
        data.insert('a', vec![44i8, 12]);
        data.insert('b', vec![]);
        data.insert('c', vec![-3, 44, 28]);

        let members = vec![
            Member::new(
                "a".to_owned(),
                Value::to_array(vec![Value::Int(44), Value::Int(12)]),
            ),
            Member::new("b".to_owned(), Value::to_array(vec![])),
            Member::new(
                "c".to_owned(),
                Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
            )
        ];

        assert_eq!(data, HashMap::deserialize(Value::Struct{ member: Box::new(members) }).unwrap());
    }

    #[test]
    fn map_accepts_boolean_keys() {
        let mut data = HashMap::new();
        data.insert(true, vec![44i8, 12]);
        data.insert(false, vec![]);

        let members = vec![
            Member::new(
                "true".to_owned(),
                Value::to_array(vec![Value::Int(44), Value::Int(12)]),
            ),
            Member::new("false".to_owned(), Value::to_array(vec![]))
        ];
        assert_eq!(data, HashMap::deserialize(Value::Struct{ member: Box::new(members) }).unwrap());
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

        let members = vec![
            Member::new("Foo".to_owned(), Value::Struct{ member: Box::new(vec![])})
        ];
        assert_eq!(
            Helper::Foo,
            Helper::deserialize(Value::Struct{ member: Box::new(members)}).unwrap()
        );

        let members = vec![
            Member::new("Bar".to_owned(), Value::Int(44))
        ];
        assert_eq!(
            Helper::Bar(44),
            Helper::deserialize(Value::Struct{ member: Box::new(members)}).unwrap()
        );

        let members = vec![
        Member::new(
            "Baz".to_owned(),
            Value::to_array(vec![Value::Bool(false), Value::String("tsk".into())]),
        )
        ];
        assert_eq!(
            Helper::Baz(false, "tsk".into()),
            Helper::deserialize(Value::Struct{ member: Box::new(members)}).unwrap()
        );

        let submembers = vec![
        Member::new("alpha".to_owned(), Value::Int(-4)),
        Member::new(
            "beta".to_owned(),
            Value::to_array(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ]))
        ];

        let mut members = vec![
            Member::new("Qux".to_owned(), Value::Struct{ member: Box::new(submembers)})
        ];
        assert_eq!(
            Helper::Qux {
                alpha: -4,
                beta: vec![true, false, true],
            },
            Helper::deserialize(Value::Struct{ member: Box::new(members)}).unwrap()
        );
    }
}

use crate::xmlfmt::{Data, FmtError, Member, Param, Value, XmlError};
// use crate::xmlfmt::;
use serde::{self, Serialize};
use std::collections::HashMap;

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = Value;
    type Error = XmlError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeVec;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeMap;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(i32::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Double(f64::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Double(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Base64(v.into()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let params = vec![value.serialize(self)?];
        let data = Data::new(params);
        Ok(Value::Array(Box::new(data)))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Struct(Box::new(HashMap::new())))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        let mut members = HashMap::new();
        members.insert(variant.into(), self.serialize_unit()?);
        Ok(Value::Struct(Box::new(members)))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let mut members = HashMap::new();
        members.insert(variant.into(), value.serialize(self)?);
        Ok(Value::Struct(Box::new(members)))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.serialize_tuple(len.unwrap_or(0))
        }
        
        fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Ok(SerializeVec {
                vec: Vec::with_capacity(len),
                variant: None,
            })
        }
                
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len),
            variant: Some(variant.into()),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
            variant: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeMap {
            map: HashMap::new(),
            next_key: None,
            variant: Some(variant.into()),
        })
    }

}

fn to_value<T>(value: &T) -> Result<Value, XmlError>
where
    T: Serialize,
{
    value.serialize(Serializer)
}

#[doc(hidden)]
pub struct SerializeVec {
    vec: Param,
    variant: Option<String>,
}

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = Value;
    type Error = XmlError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), XmlError>
    where
        T: Serialize + ?Sized,
    {
        self.vec.push(to_value(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Value, XmlError> {
        let data = Data::new(self.vec);
        let content = Value::Array(Box::new(data));
        let result = match self.variant {
            Some(variant) => {
                let member = Member::new(variant, content);
                Value::Struct{ member: Box::new(vec![member]) }
            }
            None => content,
        };
        Ok(result)
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = Value;
    type Error = XmlError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), XmlError>
    where
        T: Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, XmlError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = Value;
    type Error = XmlError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), XmlError>
    where
        T: Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, XmlError> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeVec {
    type Ok = Value;
    type Error = XmlError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), XmlError>
    where
        T: Serialize + ?Sized,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value, XmlError> {
        serde::ser::SerializeSeq::end(self)
    }
}

#[doc(hidden)]
pub struct SerializeMap {
    map: HashMap<String, Value>,
    next_key: Option<String>,
    variant: Option<String>,
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = XmlError;
    
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), XmlError>
    where
    T: Serialize + ?Sized,
    {
        match to_value(&key)? {
            Value::Bool(v) => self.next_key = Some(v.to_string()),
            Value::Int(v) => self.next_key = Some(v.to_string()),
            Value::Double(v) => self.next_key = Some(v.to_string()),
            Value::String(s) => self.next_key = Some(s),
            _ => Err(XmlError::Format(FmtError::UnsupportedFormat(
                "Key must be a bool, int, float, char or string.".into(),
            )))?,
        };
        Ok(())
    }
    
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), XmlError>
    where
    T: Serialize + ?Sized,
    {
        let key = self.next_key.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let key = key.expect("serialize_value called before serialize_key");
        self.map.insert(key, to_value(&value)?);
        Ok(())
    }
    
    // TODO: See about finalizing this somehow?
    fn end(self) -> Result<Value, XmlError> {
        Ok(match self.variant {
            // I'm not sure where or how variant comes to play for this implementation?
            Some(variant) => {
                let members = self.map.iter().fold(Vec::new(), |mut list, (name, value)| {
                    let member = Member::new(name, value.to_owned());
                    list.push(member);
                    list 
                });
                Value::Struct{ member: Box::new(members) }
            }
            None => todo!("What happens here?"),
        })
    }   
}
                        
impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = XmlError;
    
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), XmlError>
    where
    T: Serialize + ?Sized,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }
        
    fn end(self) -> Result<Value, XmlError> {
        serde::ser::SerializeMap::end(self)
    }
}
                    
impl serde::ser::SerializeStructVariant for SerializeMap {
    type Ok = Value;
    type Error = XmlError;
    
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), XmlError>
    where
    T: Serialize + ?Sized,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Value, XmlError> {
        serde::ser::SerializeMap::end(self)
    }
}


#[cfg(test)]
mod tests {
    use crate::xmlfmt::ser::Serializer;
    use crate::xmlfmt::*;
    use serde::Serialize;
    use serde_bytes::Bytes;
    use std::collections::HashMap;

    #[test]
    fn writes_bool() {
        assert_eq!(true.serialize(Serializer {}).unwrap(), Value::Bool(true));
        assert_eq!(false.serialize(Serializer {}).unwrap(), Value::Bool(false));
    }

    // FIXME: certain integer type can be string when received on python side! See bug report: https://github.com/issues/created?issue=adnanademovic|xml-rpc-rs|6
    // TODO: Find a way to test true integer value type against u16 and u32 types!
    #[test]
    fn writes_integers_as_ints_or_strings_if_too_big() {
        assert_eq!(200u8.serialize(Serializer {}).unwrap(), Value::Int(200));
        assert_eq!(
            42_000u16.serialize(Serializer {}).unwrap(),
            Value::Int(42_000)
        );
        assert_eq!(
            4_200_000_000u32.serialize(Serializer {}).unwrap(),
            Value::String("4200000000".into())
        );
        assert_eq!(
            10_000_000_000_000_000_000u64
                .serialize(Serializer {})
                .unwrap(),
            Value::String("10000000000000000000".into())
        );
        assert_eq!((-42_i8).serialize(Serializer {}).unwrap(), Value::Int(-42));
        assert_eq!(
            (-26_000_i16).serialize(Serializer {}).unwrap(),
            Value::Int(-26_000)
        );
        assert_eq!(
            (-2_000_000_000_i32).serialize(Serializer {}).unwrap(),
            Value::Int(-2000000000)
        );
        assert_eq!(
            (-8_000_000_000_000_000_000_i64)
                .serialize(Serializer {})
                .unwrap(),
            Value::String("-8000000000000000000".into())
        );
        assert_eq!(42i8.serialize(Serializer {}).unwrap(), Value::Int(42));
        assert_eq!(
            26_000i16.serialize(Serializer {}).unwrap(),
            Value::Int(26_000)
        );
        assert_eq!(
            2_000_000_000i32.serialize(Serializer {}).unwrap(),
            Value::Int(2000000000)
        );
        assert_eq!(
            8_000_000_000_000_000_000i64
                .serialize(Serializer {})
                .unwrap(),
            Value::String("8000000000000000000".into())
        );
    }

    #[test]
    fn writes_floats() {
        assert_eq!(
            3.25f32.serialize(Serializer {}).unwrap(),
            Value::Double(3.25f64)
        );
        assert_eq!(
            3.25f64.serialize(Serializer {}).unwrap(),
            Value::Double(3.25f64)
        );
    }

    #[test]
    fn writes_chars_as_strings() {
        assert_eq!(
            'A'.serialize(Serializer {}).unwrap(),
            Value::String("A".into())
        );
        assert_eq!(
            ' '.serialize(Serializer {}).unwrap(),
            Value::String(" ".into())
        );
    }

    #[test]
    fn writes_strings() {
        assert_eq!(
            "static string".serialize(Serializer {}).unwrap(),
            Value::String("static string".into())
        );
        assert_eq!(
            String::from("string object")
                .serialize(Serializer {})
                .unwrap(),
            Value::String("string object".into())
        );
    }

    #[test]
    fn writes_bytes_as_base64() {
        assert_eq!(
            Bytes::new(b"0123").serialize(Serializer {}).unwrap(),
            Value::Base64(vec![48, 49, 50, 51])
        );
    }

    #[test]
    fn writes_options_as_one_elem_or_empty_array() {
        let none: Option<i32> = None;
        assert_eq!(
            none.serialize(Serializer {}).unwrap(),
            Value::Array(Box::new(Data::new(Vec::new())))
        );
        assert_eq!(
            Some(33i32).serialize(Serializer {}).unwrap(),
            Value::Array(Box::new(Data::new(vec![Value::Int(33)])))
        );
        assert_eq!(
            Some("txt").serialize(Serializer {}).unwrap(),
            Value::Array(Box::new(Data::new(vec![Value::String("txt".into())])))
        );
    }

    #[test]
    fn writes_units_as_empty_struct() {
        assert_eq!(
            ().serialize(Serializer {}).unwrap(),
            Value::Struct(HashMap::new())
        );

        #[derive(Serialize)]
        struct Helper;

        assert_eq!(
            Helper.serialize(Serializer {}).unwrap(),
            Value::Struct(HashMap::new())
        );
    }

    #[test]
    fn writes_newtype_struct_as_its_content() {
        #[derive(Serialize)]
        struct HelperInt(i32);
        #[derive(Serialize)]
        struct HelperString(String);

        assert_eq!(
            HelperInt(33).serialize(Serializer {}).unwrap(),
            Value::Int(33)
        );
        assert_eq!(
            HelperString("txt".into()).serialize(Serializer {}).unwrap(),
            Value::String("txt".into())
        );
    }

    #[test]
    fn writes_vector_as_array() {
        let expected = Value::Array(Box::new(Data::new(vec![
                Value::Int(33),
                Value::Int(15),
                Value::Int(44),
                Value::Int(12),
            ])));
        assert_eq!(
            vec![33, 15, 44, 12].serialize(Serializer {}).unwrap(),
            expected
        );
        let expected = Value::Array(Box::new(Data::new(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
            ])));
        assert_eq!(
            vec!['a', 'b', 'c', 'd'].serialize(Serializer {}).unwrap(),
            expected
        );
    }

    #[test]
    fn writes_tuple_as_array() {
        let expected = Value::Array(Box::new(Data::new(vec![
                Value::Int(4),
                Value::String("1000000000000".into()),
                Value::String("hello".into()),
                Value::Bool(true),
            ])));
        assert_eq!(
            (4, 1_000_000_000_000i64, "hello", true)
                .serialize(Serializer {})
                .unwrap(),
            expected
        );

        #[derive(Serialize)]
        struct Helper(u8, u64, String, bool);

        let expected = Value::Array(Box::new(Data::new(vec![
                Value::Int(4),
                Value::String("1000000000000".into()),
                Value::String("hello".into()),
                Value::Bool(true),
            ])));
        assert_eq!(
            Helper(4, 1_000_000_000_000u64, "hello".into(), true)
                .serialize(Serializer {})
                .unwrap(),
            expected
        );
    }

    #[test]
    fn writes_struct_as_struct() {
        #[derive(Serialize)]
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
            }
            .serialize(Serializer {})
            .unwrap(),
            Value::Struct(members)
        );
    }

    #[test]
    fn writes_map_as_struct() {
        let mut data = HashMap::new();
        data.insert("foo", vec![44i8, 12]);
        data.insert("bar", vec![]);
        data.insert("baz", vec![-3, 44, 28]);

        let mut members = HashMap::new();
        members.insert(
            "foo".into(),
            Value::to_array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("bar".into(), Value::to_array(vec![]));
        members.insert(
            "baz".into(),
            Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(
            data.serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );
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
            Value::to_array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("bar".into(), Value::to_array(vec![]));
        members.insert(
            "baz".into(),
            Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(
            data.serialize(Serializer {}).unwrap(),
            Value::Struct(Box::new(members))
        );
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
            Value::to_array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("-33".into(), Value::to_array(vec![]));
        members.insert(
            "44".into(),
            Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(
            data.serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );
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
            Value::to_array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("b".into(), Value::to_array(vec![]));
        members.insert(
            "c".into(),
            Value::to_array(vec![Value::Int(-3), Value::Int(44), Value::Int(28)]),
        );

        assert_eq!(
            data.serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );
    }

    #[test]
    fn map_accepts_boolean_keys() {
        let mut data = HashMap::new();
        data.insert(true, vec![44i8, 12]);
        data.insert(false, vec![]);

        let mut members = HashMap::new();
        members.insert(
            "true".into(),
            Value::to_array(vec![Value::Int(44), Value::Int(12)]),
        );
        members.insert("false".into(), Value::to_array(vec![]));

        assert_eq!(
            data.serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );
    }

    #[test]
    fn rejects_maps_with_unsupported_keys() {
        let mut data = HashMap::new();
        data.insert(Some(4), vec![44i8, 12]);
        data.insert(Some(3), vec![]);
        data.insert(Some(2), vec![-3, 44, 28]);
        data.serialize(Serializer {}).unwrap_err();
    }

    #[test]
    fn writes_variant_as_one_member_struct() {
        #[derive(Debug, Serialize)]
        enum Helper {
            Foo,
            Bar(i32),
            Baz(bool, &'static str),
            Qux { alpha: i32, beta: Vec<bool> },
        }

        let mut members = HashMap::new();
        members.insert("Foo".into(), Value::Struct(HashMap::new()));
        assert_eq!(
            Helper::Foo.serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );

        let mut members = HashMap::new();
        members.insert("Bar".into(), Value::Int(44));
        assert_eq!(
            Helper::Bar(44).serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );

        let mut members = HashMap::new();
        members.insert(
            "Baz".into(),
            Value::to_array(vec![Value::Bool(false), Value::String("tsk".into())]),
        );
        assert_eq!(
            Helper::Baz(false, "tsk").serialize(Serializer {}).unwrap(),
            Value::Struct(members)
        );

        let mut submembers = HashMap::new();
        submembers.insert("alpha".into(), Value::Int(-4));
        submembers.insert(
            "beta".into(),
            Value::to_array(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
            ]),
        );

        let mut members = HashMap::new();
        members.insert("Qux".into(), Value::Struct(Box::new(submembers)));
        assert_eq!(
            Helper::Qux {
                alpha: -4,
                beta: vec![true, false, true],
            }
            .serialize(Serializer {})
            .unwrap(),
            Value::Struct(members)
        );
    }
}

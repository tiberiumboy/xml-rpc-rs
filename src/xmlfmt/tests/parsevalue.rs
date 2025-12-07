// TODO: This code smells fishy. Investigate why, but this should be placed inside unit test?
// FIXME: relocate these unit test into their respective struct files instead of isolating into a separate unit test file to run against.
#[cfg(test)]
mod tests {
    use crate::xmlfmt::*;

    use serde::Deserialize;
    use std::collections::HashMap;

    static BAD_DATA: &str = "Bad data provided";

    fn ser_and_de_response_value(value: XmlResult) {
        use crate::xmlfmt::to_xml::ToXml;
        let data = value.to_xml();
        let data = parse::response(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(value, data);
    }

    fn ser_and_de(value: Value) {
        ser_and_de_response_value(Ok(vec![value]));
    }

    fn ser_and_de_call_value(value: Call) {
        use crate::xmlfmt::to_xml::ToXml;
        let data = value.to_xml();
        let data = parse::call(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(value, data);
    }

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

    #[test]
    fn writes_fault() {
        ser_and_de_response_value(Err(Fault {
            code: 4,
            message: "Too many parameters.".into(),
        }));
    }

    #[test]
    fn writes_call() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        ser_and_de_call_value(Call {
            name: String::from("foobar"),
            params: vec![Value::String("South Dakota".into()), Value::Struct(fields)],
        });
    }

    #[test]
    fn reads_and_writes_empty_call() {
        ser_and_de_call_value(Call {
            name: String::new(),
            params: Vec::new(),
        })
    }

    #[test]
    fn reads_and_writes_empty_response() {
        ser_and_de_response_value(Ok(vec![]))
    }
}

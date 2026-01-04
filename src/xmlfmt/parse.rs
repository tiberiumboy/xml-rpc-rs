/* 

use crate::XmlResponse;
use crate::xmlfmt::error::FmtError;
use crate::xmlfmt::error::XmlError;
use crate::xmlfmt::Params;
use crate::xmlfmt::value::StructItem;
use crate::xmlfmt::{Call, Fault, Value, XmlResult};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use regex::Regex;
use serde::Deserialize; // , Deserializer
use std;
use std::collections::HashMap;
use std::io::Read as IoRead;

*/

// TODO: Perform unit test on this? I'm not quite confident to understand what this function does with three regular expressions?
// I get what it replaces, but I think there's a better way to get around this without using regex. Consider stream?
// fn wrap_in_string(content: String) -> String {
//     lazy_static! {
//         static ref RE1: Regex = Regex::new(r"<value\s*/>").unwrap();
//         static ref RE2: Regex = Regex::new(r"<value\s*>\s*<string\s*/>\s*</value\s*>").unwrap();
//         static ref RE3: Regex = Regex::new(r"<value\s*>(?P<rest>[^<>]*)</value\s*>").unwrap();
//     }
//     // I'm legitimately confused about this`` operation here?
//     RE3.replace_all(
//         &RE2.replace_all(
//             &RE1.replace_all(&content, "<value><string></string></value>"),
//             "<value><string></string></value>",
//         ),
//         "<value><string>$rest</string></value>",
//     )
//     .into()
// }

/*

// FIXME: Unused code but is being used for unit test...
#[allow(dead_code)]
pub fn xml<T: IoRead>(mut r: T) -> XmlResult<Value> {
    // TODO: Stream parse this instead of using string?
    let mut content = String::new();
    r.read_to_string(&mut content).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to read data source.{:?}",
            e
        )))
    })?;
    
    // let format_content = &wrap_in_string(content);
    let format_content = content;
    println!("XML Content: {}", &format_content);
    
    // Ok.. wtf is this? 
    serde_xml_rs::from_str::<Value>(&format_content).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to parse XML-RPC data.{:?}",
            e
        )))
    })
}

pub fn call<T: IoRead>(mut r: T) -> XmlResult<Call> {
    let mut content = String::new();
    r.read_to_string(&mut content).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to read data source.{:?}",
            e
        )))
    })?;
    let data: XmlCall = serde_xml_rs::from_str(&wrap_in_string(content)).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to parse XML-RPC call.{:?}",
            e
        )))
    })?;
    data.into()
}


pub fn response<T: IoRead>(mut r: T) -> XmlResult<XmlResponse> {
    let mut content = String::new();
    r.read_to_string(&mut content).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to read data source.{:?}",
            e
        )))
    })?;
    let data: XmlResponse = serde_xml_rs::from_str(&wrap_in_string(content)).map_err(|e| {
        XmlError::Format(FmtError::Decoding(format!(
            "Failed to parse XML-RPC response.{:?}",
            e
        )))
    })?;
    Ok(data)
}
*/

/* 

    Please read:
    This commented section contains code that was used to parse xml string from some formatted data back into true "value" (minus the xml prefix).
    Instead of creating duplicated copy of the class, I would like to know why this codebase branch off to handle both conditions separately.
    If they cannot function the same from the original class, then there needs to be a restructuring in the code architecture to make sure we avoid code duplication.

    Please implement in the "true" object class (Minus the prefix "xml") necessary function instead.
    
#[derive(Debug, PartialEq, Deserialize)]
enum XmlValue {
    #[serde(rename = "i4")]
    I4(i32),
    #[serde(rename = "int")]
    Int(i32),
    #[serde(rename = "boolean")]
    Bool(i32),
    #[serde(rename = "string")]
    Str(String),
    #[serde(rename = "double")]
    Double(String),
    #[serde(rename = "dateTime.iso8601")]
    DateTime(String),
    #[serde(rename = "base64")]
    Base64(String),
    #[serde(rename = "array")]
    Array(XmlArray),
    #[serde(rename = "struct")]
    Struct(XmlStruct),
}

impl From<XmlValue> for XmlResult<Value> {
    fn from(val: XmlValue) -> Self {
        Ok(match val {
            XmlValue::I4(v) | XmlValue::Int(v) => Value::Int(v),
            XmlValue::Bool(v) => Value::Bool(v != 0),
            XmlValue::Str(v) => Value::String(v),
            XmlValue::Double(v) => Value::Double(v.parse().map_err(|e| {
                XmlError::Format(FmtError::Decoding(format!(
                    "Failed to parse double: {:?}",
                    e
                )))
            })?),
            XmlValue::DateTime(v) => Value::DateTime(v),
            XmlValue::Base64(v) => Value::Base64(STANDARD.decode(v.as_bytes()).map_err(|e| {
                XmlError::Format(FmtError::Decoding(format!(
                    "Failed to parse base64: {:?}",
                    e
                )))
            })?),
            XmlValue::Array(v) => {
                let items: XmlResult<Params> = v.into();
                Value::Array(items?)
            }
            XmlValue::Struct(v) => {
                Value::Struct(v.into)
            }
        })
    }
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename = "methodCall")]
struct XmlCall {
    #[serde(rename = "methodName")]
    pub name: String,
    pub params: XmlParams,
}

impl From<XmlCall> for XmlResult<Call> {
    fn from(val: XmlCall) -> Self {
        let params: XmlResult<Params> = val.params.into();
        Ok(Call {
            name: val.name,
            params: params?,
        })
    }
}

/*
#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponseResult {
    #[serde(rename = "params")]
    Success(XmlParams),
    #[serde(rename = "fault")]
    Failure { value: XmlValue },
}

impl From<XmlResponseResult> for XmlResult<XmlResponse> {
    fn from(val: XmlResponseResult) -> Self {
        match val {
            XmlResponseResult::Success(params) => {
                let params: XmlResult<Vec<Value>> = params.into();
                Ok(params?)
            }
            XmlResponseResult::Failure { value: v } => {
                use serde::Deserialize;

                let val: XmlResult<Value> = v.into();

                Ok(Fault::deserialize(val?).map_err(|e| {
                    XmlError::Format(FmtError::Decoding(format!(
                        "Failed to decode fault structure: {}",
                        e
                    )))
                })?)
            }
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
enum XmlResponse {
    #[serde(rename = "methodResponse")]
    Response(XmlResponseResult),
}

impl From<XmlResponse> for XmlResult<XmlResponse> {
    fn from(val: XmlResponse) -> Self {
        match val {
            XmlResponse::Response(v) => v.into(),
        }
    }
}
 */

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParams {
    #[serde(rename = "param", default)]
    pub params: Vec<XmlParamData>,
}

impl From<XmlParams> for XmlResult<Vec<Value>> {
    fn from(val: XmlParams) -> Self {
        val.params
            .into_iter()
            .map(Into::<Result<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlParamData {
    pub value: XmlValue,
}

impl From<XmlParamData> for XmlResult<Value> {
    fn from(val: XmlParamData) -> Self {
        val.value.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArray {
    #[serde(rename = "data")]
    pub data: XmlArrayData,
}

impl From<XmlArray> for XmlResult<Vec<Value>> {
    fn from(val: XmlArray) -> Self {
        val.data.into()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlArrayData {
    #[serde(default)]
    pub value: Vec<XmlValue>,
}

impl From<XmlArrayData> for XmlResult<Vec<Value>> {
    fn from(val: XmlArrayData) -> Self {
        val.value
            .into_iter()
            .map(Into::<XmlResult<Value>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStruct {
    #[serde(rename = "member", default)]
    pub members: Vec<XmlStructItem>,
}

impl From<XmlStruct> for XmlResult<XmlStructItem> {
    fn from(val: XmlStruct) -> Self {
        val.members
            .into_iter()
            .map(Into::<Result<(String, Value)>>::into)
            .collect()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct XmlStructItem {
    pub name: String,
    pub value: XmlValue,
}

impl From<XmlStructItem> for XmlResult<(String, Value)> {
    fn from(val: XmlStructItem) -> Self {
        let value: Result<Value> = val.value.into();
        Ok((val.name, value?))
    }
}

#[cfg(test)]
mod tests {
    use crate::xmlfmt::tests::*;
    use crate::xmlfmt::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[test]
    fn reads_pod_xml_value() {
        let data = r#"<?xml version="1.0"?><string>South Dakota</string>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::String("South Dakota".into()));
        let data = r#"<?xml version="1.0"?><string />"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::String("".into()));
        let data = r#"<?xml version="1.0"?><string></string>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::String("".into()));

        let data = r#"<?xml version="1.0"?><int>-33</int>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Int(-33));
        let data = r#"<?xml version="1.0"?><i4>-33</i4>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Int(-33));

        let data = r#"<?xml version="1.0"?><boolean>1</boolean>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Bool(true));
        let data = r#"<?xml version="1.0"?><boolean>0</boolean>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Bool(false));

        let data = r#"<?xml version="1.0"?><double>-44.2</double>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Double(-44.2));

        let data = r#"<?xml version="1.0"?><dateTime.iso8601>33</dateTime.iso8601>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::DateTime("33".into()));

        let data = r#"<?xml version="1.0"?><base64>Zm9vYmFy</base64>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Base64("foobar".into()));
    }

    #[test]
    fn reads_empty_array_xml_value() {
        let data = r#"<?xml version="1.0"?>
<array>
    <data>
    </data>
</array>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Array(vec![]));
    }

    #[test]
    fn reads_array_xml_value() {
        // <?xml version="1.0"?>
        let data = r#"
<array>
    <data>
        <value><i4>33</i4></value>
        <value><i4>-12</i4></value>
        <value><i4>44</i4></value>
    </data>
</array>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            Value::Array(vec![Value::Int(33), Value::Int(-12), Value::Int(44)])
        );
    }

    #[test]
    fn reads_empty_struct_xml_value() {
        let data = r#"<?xml version="1.0"?><struct></struct>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Struct(HashMap::<String, Value>::new()));
    }

    #[test]
    fn reads_tagged_and_untagged_strings() {
        let data = r#"<?xml version="1.0"?>
<array>
    <data>
        <value><string>foo</string></value>
        <value><string></string></value>
        <value><string /></value>
        <value>bar</value>
        <value></value>
        <value />
    </data>
</array>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            Value::Array(vec![
                Value::String("foo".into()),
                Value::String(String::new()),
                Value::String(String::new()),
                Value::String("bar".into()),
                Value::String(String::new()),
                Value::String(String::new()),
            ])
        );
    }

    #[test]
    fn reads_struct_xml_value() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        let data = r#"<?xml version="1.0"?>
<struct>
    <member>
        <name>foo</name>
        <value><i4>42</i4></value>
    </member>
    <member>
        <name>bar</name>
        <value><string>baz</string></value>
    </member>
</struct>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Value::Struct(fields));
    }

    #[test]
    fn reads_response() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        fields.insert("bar2".into(), Value::String("baz2".into()));
        let params = vec![Value::String("South Dakota".into()), Value::Struct(fields)];
        let data = r#"<?xml version="1.0"?>
<methodResponse>
    <params>
        <param>
            <value><string>South Dakota</string></value>
        </param>
        <param>
            <value>
                <struct>
                    <member>
                        <name>foo</name>
                        <value><i4>42</i4></value>
                    </member>
                    <member>
                        <name>bar</name>
                        <value><string>baz</string></value>
                    </member>
                    <member>
                        <name>bar2</name>
                        <value>baz2</value>
                    </member>
                </struct>
            </value>
        </param>
    </params>
</methodResponse>"#;
        let data = parse::response(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data, Ok(params));
    }

    #[test]
    fn reads_fault() {
        let data = r#"<?xml version="1.0"?>
<methodResponse>
    <fault>
        <value>
            <struct>
                <member>
                    <name>faultCode</name>
                    <value><int>4</int></value>
                </member>
                <member>
                    <name>faultString</name>
                    <value><string>Too many parameters.</string></value>
                </member>
            </struct>
        </value>
    </fault>
</methodResponse>"#;
        let data = parse::response(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(
            data,
            Err(Fault {
                code: 4,
                message: "Too many parameters.".into(),
            })
        );
    }

    #[test]
    fn reads_call() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        let data = r#"<?xml version="1.0"?>
<methodCall>
    <methodName>foobar</methodName>
    <params>
        <param>
            <value><string>South Dakota</string></value>
        </param>
        <param>
            <value>
                <struct>
                    <member>
                        <name>foo</name>
                        <value><i4>42</i4></value>
                    </member>
                    <member>
                        <name>bar</name>
                        <value><string>baz</string></value>
                    </member>
                </struct>
            </value>
        </param>
    </params>
</methodCall>"#;
        let data = parse::call(data.as_bytes()).expect(BAD_DATA);
        assert_eq!(data.name, String::from("foobar"));
        assert_eq!(
            data.params,
            vec![Value::String("South Dakota".into()), Value::Struct(fields)]
        );
    }

    #[test]
    fn reads_array_structure_xml_value() {
        let data = r#"<?xml version="1.0"?>
<array>
    <data>
        <value><i4>33</i4></value>
        <value><i4>-12</i4></value>
        <value><i4>44</i4></value>
    </data>
</array>"#;
        let data = parse::xml(data.as_bytes()).expect(BAD_DATA);
        let data = Vec::<i32>::deserialize(data).expect(BAD_DATA);
        assert_eq!(data, vec![33, -12, 44]);
    }

    #[test]
    fn reads_and_writes_empty_response() {
        tests::ser_and_de_response_value(Ok(vec![]))
    }
}

*/
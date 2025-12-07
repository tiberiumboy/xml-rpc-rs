use crate::xmlfmt::alias::XmlResult;
use crate::xmlfmt::call::Call;
use crate::xmlfmt::fault::Fault;
use crate::xmlfmt::value::Value;
use base64::{Engine as _, engine::general_purpose::STANDARD};

use xml::escape::escape_str_pcdata;

pub trait ToXml {
    fn to_xml(&self) -> String;
}

impl ToXml for Call {
    fn to_xml(&self) -> String {
        use std::fmt::Write;
        format!(
            include_str!("templates/call.xml"),
            name = self.name,
            params = self.params.iter().fold(String::new(), |mut output, param| {
                let _ = write!(output, "<param>{}</param>", param.to_xml());
                output
            }),
        )
    }
}

impl ToXml for XmlResult {
    fn to_xml(&self) -> String {
        use std::fmt::Write;
        match *self {
            Ok(ref params) => format!(
                include_str!("templates/response_success.xml"),
                params = params.iter().fold(String::new(), |mut output, param| {
                    let _ = write!(output, "<param>{}</param>", param.to_xml());
                    output
                })
            ),
            Err(Fault { code, ref message }) => format!(
                include_str!("templates/response_fault.xml"),
                code = code,
                message = message
            ),
        }
    }
}

impl ToXml for Vec<Value> {
    fn to_xml(&self) -> String {
        let mut result = String::new();
        for value in self.into_iter() {
            result.push_str(&value.to_xml());
        }
        result
    }
}

// TODO: is there a way to run a to_xml expression on Vec<Value> types?
impl ToXml for Value {
    fn to_xml(&self) -> String {
        match *self {
            Value::Int(v) => format!("<value><i4>{}</i4></value>", v),
            Value::Bool(v) => format!(
                "<value><boolean>{}</boolean></value>",
                if v { 1 } else { 0 }
            ),
            Value::String(ref v) => {
                format!("<value><string>{}</string></value>", escape_str_pcdata(v))
            }
            Value::Double(v) => format!("<value><double>{}</double></value>", v),
            Value::DateTime(ref v) => {
                format!("<value><dateTime.iso8601>{}</dateTime.iso8601></value>", v)
            }
            Value::Base64(ref v) => {
                format!("<value><base64>{}</base64></value>", STANDARD.encode(v))
            }
            Value::Array(ref v) => format!(
                "<value><array><data>{}</data></array></value>",
                v.iter().map(Value::to_xml).collect::<String>()
            ),
            Value::Struct(ref v) => format!(
                "<value><struct>{}</struct></value>",
                v.iter().fold(String::new(), |mut output, (key, value)| {
                    use std::fmt::Write;
                    let _ = write!(
                        output,
                        "<member><name>{}</name>{}</member>",
                        key,
                        value.to_xml()
                    );
                    output
                })
            ),
        }
    }
}

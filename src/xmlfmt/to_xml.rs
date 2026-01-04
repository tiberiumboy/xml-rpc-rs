use crate::xmlfmt::XmlResult;

pub trait ToXml {
    fn to_xml(&self) -> XmlResult<String>;
}

/*
impl ToXml for Value {
    fn to_xml(&self) -> XmlResult<String> {
        to_string(&self)
            .map_err(|e|
                XmlError::Format(FmtError::Encoding(e.to_string()))
        )
        
        // match *self {
        //     Value::I4(v) => format!("<value><i4>{}</i4></value>", v),
        //     Value::Int(v) => format!("<value><int>{}</int></value>", v),
        //     Value::Bool(v) => format!( "<value><boolean>{}</boolean></value>", i32::from(v)),
        //     Value::String(ref v) => {
        //         format!("<value><string>{}</string></value>", Value::escape_str_pcdata(v)) // replace invalid xml characters
        //     }
        //     Value::Double(v) => format!("<value><double>{}</double></value>", v),
        //     Value::DateTime(ref v) => {
        //         format!("<value><dateTime.iso8601>{}</dateTime.iso8601></value>", v)
        //     }
        //     Value::Base64(ref v) => {
        //         format!("<value><base64>{}</base64></value>", STANDARD.encode(v))
        //     }
        //     Value::Array(ref v) => format!(
        //         "<value><array><data>{}</data></array></value>",
        //         v.iter().map(Value::to_xml).collect::<String>()
        //     ),
        //     Value::Struct(ref v) => format!(
        //         "<value><struct>{}</struct></value>",
        //         v.iter().fold(String::new(), |mut output, (key, value)| {
        //             use std::fmt::Write;
        //             let _ = write!(
        //                 &mut output,
        //                 "<member><name>{}</name>{}</member>",
        //                 key,
        //                 value.to_xml()
        //             );
        //             output
        //         })
        //     ),
        // }
    }
}
 */
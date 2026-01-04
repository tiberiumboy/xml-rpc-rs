use serde::{Deserialize, Serialize};
use serde_xml_rs::to_string;
use crate::xmlfmt::{FmtError, XmlError};
use crate::xmlfmt::to_xml::ToXml;
use crate::xmlfmt::Params;
use crate::xmlfmt::XmlResult;

// Call is the method to invoke methods on python side. Keep it.
/*
    Call schematics is defined as below:
    <?xml version="1.0"?>
    <methodCall>
        <methodName>{name}</methodName>
        <params>{params}</params>
    </methodCall>
*/
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename="methodCall")]
pub struct Call {
    #[serde(rename="methodName")]
    pub name: String,   // method name
    pub params: Params, // parameters/arguments
}

impl Call {
    pub fn new<T>(name: T, params: Params) -> Self
    where
        T: Into<String>,
    {
        Self {
            name: name.into(),
            params,
        }
    }

    pub fn from_xml(str: &str) -> XmlResult<Call> {
        // strip away <?xml keyword, or prefix/suffix content
        let result = serde_xml_rs::from_str::<Call>(&str)
            .map_err(|e| 
                // FEATURE: Fault code is application specific, consider making new enum for our own fault codes
                XmlError::Format(FmtError::Decoding(e.to_string())));
        result
    }
}

impl ToXml for Call {
    fn to_xml(&self) -> XmlResult<String> {
        to_string(&self)
            .map_err(|e| 
                XmlError::Format(
                    FmtError::Encoding(e.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use crate::xmlfmt::*;

    pub fn ser_and_de_call_value(value: Call) {
        let data = value.to_xml();
        assert!(data.is_ok(), "Fail to serialize Call to XML: {}", data.unwrap_err());
        let result = Call::from_xml(&data.unwrap());
        assert!(result.is_ok(), "Failed to deserialize Call from XML: {}", result.unwrap_err());
        assert_eq!(result.unwrap(), value);
    }

    #[test]
    fn writes_call() {
        // this would be valid for response?
        let param: Param = vec![
            Value::Int(42),
            Value::String("baz".into())
        ];

        let params = Params::new(param);
        let name = String::from("foobar");
        let call = Call::new(name, params);

        ser_and_de_call_value(call);
    }

    #[test]
    fn reads_and_writes_empty_call() {
        ser_and_de_call_value(Call {
            name: Default::default(),
            params: Default::default(),
        })
    }
}

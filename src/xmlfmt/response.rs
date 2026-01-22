use crate::xmlfmt::{FmtError, XmlError, XmlResult};
use crate::{Params, Value, xmlfmt::ToXml};
use serde::{Deserialize, Serialize};
use serde_xml_rs::to_string;

// used everywhere for type cast declaration
// TODO: Read the documentation to see if this is supported? Otherwise, I need to handle receiving server response and parse them myself.
pub type XmlResponse = Result<Params, Value>; // we need to make sure that value for error must be fault!

/*
    The design schema for this is this is treated as the root object, so it will alwayas have <?xml ... /> tag.

    <?xml version="1.0"?>
    <methodResponse>
        <params || fault />
    </methodResponse>
*/

// TODO: For this case here, we need to handle deserializing Tuples as serde_xml_rs does not support it.
// Only used within this crate, as it's meant to be used to receive response from server
#[derive(PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum MethodResponse {
    Params(Params),
    Fault(Value),
}

impl MethodResponse {
    #[allow(dead_code)]
    pub fn to_fault<T>(code: i32, message: T) -> MethodResponse
    where
        T: Into<String>,
    {
        MethodResponse::Fault(Value::fault(code, message))
    }

    pub fn new(input: XmlResponse) -> MethodResponse {
        match input {
            Ok(param) => MethodResponse::Params(param),
            Err(value) => MethodResponse::Fault(value),
        }
    }
}

impl ToXml for MethodResponse {
    fn to_xml(&self) -> XmlResult<String> {
        to_string(self).map_err(|e| XmlError::Format(FmtError::Encoding(e.to_string())))
    }
}

impl Into<MethodResponse> for XmlResponse {
    fn into(self) -> MethodResponse {
        match self {
            Ok(v) => MethodResponse::Params(v),
            Err(e) => MethodResponse::Fault(e),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::xmlfmt::{Member, ToXml};

    // why are we asking for the XmlResult when we expects the params instead?
    pub fn ser_and_de_response_value(value: MethodResponse) {
        let data = value.to_xml();
        assert!(
            data.is_ok(),
            "Unable to serialize Response value! {}",
            data.unwrap_err()
        );
        let result = serde_xml_rs::from_str::<MethodResponse>(&data.unwrap());
        assert!(result.is_ok_and(|v| v == value));
    }

    #[test]
    fn writes_response() {
        let members: Vec<Member> = vec![
            Member::new("foo".to_owned(), Value::Int(42)),
            Member::new("bar".to_owned(), Value::String("baz".into())),
        ];
        let data = vec![
            Value::String("South Dakota".into()),
            Value::Struct {
                member: Box::new(members),
            },
        ];
        let params = Params::new(data);
        let response = MethodResponse::Params(params);
        ser_and_de_response_value(response)
    }

    #[test]
    fn writes_fault() {
        let fault = Value::fault(4, "Too many parameters".to_owned());
        let response = MethodResponse::Fault(fault);
        ser_and_de_response_value(response);
    }
}

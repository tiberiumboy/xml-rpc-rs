use crate::{Value, xmlfmt::MethodResponse};

pub type Param = Vec<Value>;

// Params is a list of param, containing value - only ever used in methodResponse and methodCall
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Params {
    param: Param,
}

impl Params {
    
    pub fn new( param: Param ) -> Self {
        Self { param }
    }
    // TODO: Rename this function- "xmlfmt::from_params(params)" sounds confusing?
/* 
pub fn from_params<'a, T: Deserialize<'a>>(mut params: Params) -> XmlResult<T> {
    // so we can create an empty array if the params is empty?
    let data = match params.len() {
        0 => {
            Value::Array(Vec::new())
        },
        1 => params.pop().unwrap(),
        _ => Value::Array(params),
    };

    T::deserialize(data).map_err(|e| {
        errors::XmlError::Format(errors::FmtError::Decoding(format!(
            "Failed to convert XML-RPC to structure. {}",
            e
        )))
    })
}
*/
}

impl Into<Param> for Params {
    fn into(self) -> Param {
        self.param
    }
}

impl Into<MethodResponse> for Params {
    fn into(self) -> MethodResponse {
        MethodResponse::Params(self)
    }
}
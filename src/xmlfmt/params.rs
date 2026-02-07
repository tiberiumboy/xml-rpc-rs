use crate::xmlfmt::{Data, MethodResponse, Value};
use serde::Deserialize;

pub type Param = Vec<Value>;

// Params is a list of param, containing value - only ever used in methodResponse and methodCall
#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Params {
    param: Option<Param>,
}

impl Params {
    pub fn new(param: Param) -> Self {
        Self { param: Some(param) }
    }
}

impl Into<Value> for Params {
    fn into(self) -> Value {
        match self.param {
            Some(mut param) => match param.len(){
                0 => Value::Nil,
                1 => param.pop().unwrap(),
                _ => Value::Array(Box::new(Data::new(param))),
            },
            None => Value::Nil
        }
    }
}

impl Into<Param> for Params {
    fn into(self) -> Param {
        self.param.unwrap_or_default()
    }
}

impl Into<MethodResponse> for Params {
    fn into(self) -> MethodResponse {
        MethodResponse::Params(self)
    }
}

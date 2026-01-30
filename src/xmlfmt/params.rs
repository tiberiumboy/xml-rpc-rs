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

    //     pub fn from_params<'a, T: Deserialize<'a>>(self) -> XmlResult<T> {
    //         // let mut list: Vec<Value> = self.into();
    //         // let data = match list.len() {
    //         //     0 => Value::String("".to_owned()),
    //         //     1 => list.pop().unwrap(),   // TODO we really should handle this gracefully?
    //         //     _ => Value::Array(Box::new(Data::new(list)))
    //         // };
    //         let data: Value = self.into();
    //         T::deserialize(data)
    //     }
}

// params are used as a
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

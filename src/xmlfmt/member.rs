use crate::Value;
use serde::{Deserialize, Serialize};

/*
    The schematic for this is as listed below:
    - commonly used for Value::Struct
    ...
    <struct>
        <member>
            <name/>
            <value/>
        </member>
        <member>
            ...
        </member>
    </struct>
*/
#[derive(PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
// #[cfg(Debug)]
#[derive(Debug)]
pub(crate) struct Member{
    name: String, 
    value: Value
}

impl Member {
    pub fn new<T>(name: T, value: Value ) -> Self
    where T: Into<String> {
        Self {
            name: name.into(),
            value
        }
    }
}

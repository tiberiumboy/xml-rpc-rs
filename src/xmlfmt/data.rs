use crate::xmlfmt::Param;
use serde::{Deserialize, Serialize};

/*
    Used for Value::Array - Schema is detail as below:
    where data could contain other elements
    <array>
        <data>
            <value><i4>1404</i4></value>
            <value><string>Something here</string></value>
            <value><i4>1</i4></value>
        </data>
    </array>
*/
#[derive(Serialize, Deserialize, PartialEq, Clone)]
// #[cfg!(Debug)]
#[derive(Debug)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    value: Param,
}

impl Data {
    pub fn new(value: Param) -> Self {
        Self { value }
    }
}

impl Into<Param> for Data {
    fn into(self) -> Param {
        self.value
    }
}

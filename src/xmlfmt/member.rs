use crate::xmlfmt::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/*
    The schematic for this is as listed below:
    - only used for Value::Struct
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
#[serde(rename_all = "camelCase")]
// #[cfg(Debug)]
#[derive(Debug)]
pub struct Member {
    name: String,
    value: Value,
}

impl Member {
    pub fn new<T>(name: T, value: Value) -> Self
    where
        T: Into<String>,
    {
        Self {
            name: name.into(),
            value,
        }
    }

    pub fn from_hashmap(hashmap: HashMap<String, Value>) -> Vec<Member> {
        hashmap.iter().fold(
            Vec::with_capacity(hashmap.capacity()),
            |mut list, (k, v)| {
                list.push(Member::new(k, v.clone()));
                list
            },
        )
    }
}

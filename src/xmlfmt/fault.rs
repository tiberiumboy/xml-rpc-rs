#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Fault {
    #[serde(rename = "faultCode")]
    pub code: i32,
    #[serde(rename = "faultString")]
    pub message: String,
}

impl Fault {
    pub fn empty() -> Self {
        Self {
            code: 400,
            message: "".into(),
        }
    }
}

impl Fault {
    pub fn new<T>(code: i32, message: T) -> Fault
    where
        T: Into<String>,
    {
        Fault {
            code,
            message: message.into(),
        }
    }
}

use crate::xmlfmt::alias::Params;

// Call is the method to invoke methods on python side. Keep it.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
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
}

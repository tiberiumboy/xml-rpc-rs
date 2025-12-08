use crate::xmlfmt::value::Params;

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

#[cfg(test)]
mod tests {
    use crate::xmlfmt::tests::*;
    use crate::xmlfmt::*;
    use std::collections::HashMap;

    #[test]
    fn writes_call() {
        let mut fields = HashMap::<String, Value>::new();
        fields.insert("foo".into(), Value::Int(42));
        fields.insert("bar".into(), Value::String("baz".into()));
        ser_and_de_call_value(Call {
            name: String::from("foobar"),
            params: vec![Value::String("South Dakota".into()), Value::Struct(fields)],
        });
    }

    #[test]
    fn reads_and_writes_empty_call() {
        ser_and_de_call_value(Call {
            name: String::new(),
            params: Vec::new(),
        })
    }
}

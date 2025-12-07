use crate::xmlfmt::alias::XmlResult;
use crate::xmlfmt::value::Value;

// Is there a trait I'm missing to implement?
pub struct XML;

// TODO: Where is this being used in the codebase?
impl XML {
    pub fn from_data<T: Into<Value>>(key: T, data: T) -> XmlResult {
        let mut values = Vec::with_capacity(2);
        values[0] = key.into();
        values[1] = data.into();
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::{Value, XML};

    #[test]
    fn should_succeed() {
        let key = "key".to_owned();
        let data = "data".to_owned();
        let key_value = Value::String(key);
        let data_value = Value::String(data);
        let result = XML::from_data(key_value.clone(), data_value.clone());
        assert!(result.is_ok_and(|xml| -> bool { xml[0] == key_value && xml[1] == data_value }))
    }
}


use crate::xmlfmt::{XmlError, Value};
use serde::de::{DeserializeSeed, EnumAccess};

pub(crate) struct EnumDeserializer {
    // FIXME: Consider making them not public
    pub(crate) variant: String,
    pub(crate) value: Value,
}


impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = XmlError;
    type Variant = Value;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Value), XmlError>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self.value;
        let variant = Value::String(self.variant);
        seed.deserialize(variant).map(|v| (v, value))
    }
}

use crate::xmlfmt::XmlResult;

pub trait ToXml {
    fn to_xml(&self) -> XmlResult<String>;
}

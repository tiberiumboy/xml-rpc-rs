use crate::xmlfmt::fault::Fault;
use crate::xmlfmt::value::Value;
use std::result::Result;

pub type Params = Vec<Value>;

pub type XmlResult = Result<Params, Fault>;

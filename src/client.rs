// use crate::error::{Result, ResultExt};
use crate::xmlfmt::alias::{Params, XmlResult};
use crate::xmlfmt::call::Call;
use crate::xmlfmt::error::XmlError;
use crate::xmlfmt::fault::Fault;
use crate::xmlfmt::{from_params, into_params, parse};
use serde::{Deserialize, Serialize};
use std;

pub fn call_value<URL, Tkey>(uri: &URL, name: Tkey, params: Params) -> Result<XmlResult, XmlError>
where
    URL: Clone,
    ureq::http::Uri: TryFrom<URL>,
    <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
    Tkey: Into<String>,
{
    Client::new()?.call_value::<URL, Tkey>(uri, name, params)
}

pub fn call<'a, URL, Tkey, Treq, Tres>(
    uri: &URL,
    name: Tkey,
    req: Treq,
) -> Result<std::result::Result<Tres, Fault>, XmlError>
where
    URL: Clone,
    ureq::http::Uri: TryFrom<URL>,
    <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
    Tkey: Into<String>,
    Treq: Serialize,
    Tres: Deserialize<'a>,
{
    Client::new()?.call(uri, name, req)
}

pub struct Client;

impl Client {
    pub fn new() -> Result<Client, XmlError> {
        Ok(Client {})
    }

    pub fn call_value<URL, Tkey>(
        &mut self,
        uri: &URL,
        name: Tkey,
        params: Params,
    ) -> Result<XmlResult, XmlError>
    where
        URL: Clone,
        ureq::http::Uri: TryFrom<URL>,
        <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
    {
        use crate::xmlfmt::to_xml::ToXml;
        let body = Call::new(name.into(), params).to_xml();
        let mut response = ureq::post(uri.clone())
            .header("Content-Type", "text/xml")
            .send(body)
            .map_err(|e| XmlError::Http(e.to_string()))?;

        let content = response.body_mut().as_reader();
        parse::response(content)
    }

    pub fn call<'a, URL, Tkey, Treq, Tres>(
        &mut self,
        uri: &URL,
        name: Tkey,
        req: Treq,
    ) -> Result<std::result::Result<Tres, Fault>, XmlError>
    where
        URL: Clone,
        ureq::http::Uri: TryFrom<URL>,
        <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
        Treq: Serialize,
        Tres: Deserialize<'a>,
    {
        match self.call_value(uri, name, into_params(&req)?) {
            Ok(Ok(v)) => from_params(v).map(Ok).map_err(Into::into),
            Ok(Err(v)) => Ok(Err(v)),
            Err(v) => Err(v),
        }
    }
}

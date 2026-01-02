use crate::{xmlfmt::{Call, Params, XmlError, XmlResponse, XmlResult}};
// use serde::{Deserialize, Serialize};
// use std;

// This should move inside client code?
pub fn call_value<URL, Tkey>(uri: &URL, name: Tkey, params: Params) -> XmlResult<XmlResponse>
where
    URL: Clone,
    ureq::http::Uri: TryFrom<URL>,
    <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
    Tkey: Into<String>,
{
    Client::new()?.call_value::<URL, Tkey>(uri, name, params)
}

pub fn call<'a, URL, Tkey, /*Treq, Tres*/>(
    uri: &URL,
    name: Tkey,
    req: Params, //Treq,
) -> XmlResult<XmlResponse>
where
    URL: Clone,
    ureq::http::Uri: TryFrom<URL>,
    <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
    Tkey: Into<String>,
    // Treq: Serialize,
    // Tres: Deserialize<'a>,
{
    Client::new()?.call(uri, name, req)
}

pub struct Client;

impl Client {
    pub fn new() -> XmlResult<Client> {
        Ok(Client {})
    }

    pub fn call_value<URL, Tkey>(
        &mut self,
        uri: &URL,
        name: Tkey,
        params: Params,
    ) -> XmlResult<XmlResponse>
    where
        URL: Clone,
        ureq::http::Uri: TryFrom<URL>,
        <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
    {
        use crate::xmlfmt::ToXml;
        let body = Call::new(name.into(), params).to_xml()?;
        let mut response = ureq::post(uri.clone())
            .header("Content-Type", "text/xml")
            .send(body)
            .map_err(|e| XmlError::Http(e.to_string()))?;

        let content = response.body_mut().as_reader();
        serde_xml_rs::from_reader(content).map_err(|e| {
            XmlError::Format(crate::xmlfmt::FmtError::Decoding(e.to_string()))
        })?
    }

    pub fn call<'a, URL, Tkey, /* Treq*/>(
        &mut self,
        uri: &URL,
        name: Tkey,
        req: Params, // Treq,
    ) -> XmlResult<XmlResponse>
    where
        URL: Clone,
        ureq::http::Uri: TryFrom<URL>,
        <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
        // Treq: Serialize,
    {
        self.call_value(uri, name, req)
    }
}

use super::error::{Result, ResultExt};
use super::xmlfmt::{from_params, into_params, parse, Call, Fault, Params, Response};
use serde::{Deserialize, Serialize};
use std;

pub fn call_value<URL, Tkey>(uri: &URL, name: Tkey, params: Params) -> Result<Response>
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
) -> Result<std::result::Result<Tres, Fault>>
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

    pub fn new() -> Result<Client> {
        Ok(Client {})
    }

    pub fn call_value<URL, Tkey>(&mut self, uri: &URL, name: Tkey, params: Params) -> Result<Response>
    where
        URL: Clone,
        ureq::http::Uri: TryFrom<URL>,
        <ureq::http::Uri as TryFrom<URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
    {
        use super::xmlfmt::value::ToXml;
        let body = Call::new(name.into(), params).to_xml();
        let mut response = ureq::post(uri.clone())
            .header("Content-Type", "text/xml")
            .send(body)
            .chain_err(|| "Failed to run the HTTP request within ureq.")?;
        parse::response(response.body_mut().as_reader()).map_err(Into::into)
    }

    pub fn call<'a, URL, Tkey, Treq, Tres>(
        &mut self,
        uri: &URL,
        name: Tkey,
        req: Treq,
    ) -> Result<std::result::Result<Tres, Fault>>
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

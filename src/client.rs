use super::error::Result;
use super::xmlfmt::{from_params, into_params, parse, Call, Fault, Params, Response};
use serde::{Deserialize, Serialize};
use std;
use ureq::http::{Uri, Error};

pub fn call_value<URL, Tkey>(uri: &URL, name: Tkey, params: Params) -> Result<Response>
where
    URL: Clone,
    Uri: TryFrom<URL>,
    Tkey: Into<String>,
{
    Client::new()?.call_value(uri, name, params)
}

pub fn call<'a, URL, Tkey, Treq, Tres>(
    uri: &URL,
    name: Tkey,
    req: Treq,
) -> Result<std::result::Result<Tres, Fault>>
where
    URL: Clone,    
    Uri: TryFrom<URL>,
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
        Uri: TryFrom<URL>,
        <Uri as TryFrom<ureq::http::URL>>::Error: Into<ureq::http::Error>,
        Tkey: Into<String>,
    {
        use super::xmlfmt::value::ToXml;
        
        // create a new request and send it to the url path.
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
        Uri: TryFrom<URL>,
        <Uri as TryFrom<URL>>::Error: Into<Error>,
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

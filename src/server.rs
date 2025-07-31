use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::slice::Iter as SliceIter;
use std::fmt;
use std::io::Read;
use std::io::Result as IoResult;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};
use crate::xmlfmt::value::XML;  

use super::error::Result;
use super::xmlfmt::{error, from_params, into_params, parse, Call, Fault, Response, Value};

type Handler = Box<dyn Fn(Vec<Value>) -> Response + Send + Sync>;
type HandlerMap = HashMap<String, Handler>;

pub fn on_decode_fail(err: &error::Error) -> Response {
    Err(Fault::new(
        400,
        format!("Failed to decode request: {}", err),
    ))
}

pub fn on_encode_fail(err: &error::Error) -> Response {
    Err(Fault::new(
        500,
        format!("Failed to encode response: {}", err),
    ))
}

fn on_missing_method(_: Vec<Value>) -> Response {
    Err(Fault::new(404, "Requested method does not exist"))
}

/// Iterator to the list of headers in a request.
#[derive(Debug, Clone)]
pub struct HeadersIter<'a> {
    iter: SliceIter<'a, (String, String)>,
}

impl<'a> Iterator for HeadersIter<'a> {
    type Item = (&'a str, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (&k[..], &v[..]))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}


/// Gives access to the body of a request.
///
/// In order to obtain this object, call `request.data()`.
pub struct RequestBody<'a> {
    body: Box<dyn Read + Send>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Read for RequestBody<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.body.read(buf)
    }
}

/// Represents a request that your handler must answer to.
///
/// This can be either a real request (received by the HTTP server) or a mock object created with
/// one of the `fake_*` constructors.
pub struct Request {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    https: bool,
    data: Arc<Mutex<Option<Box<dyn Read + Send>>>>,
    remote_addr: Option<SocketAddr>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("https", &self.https)
            .field("remote_addr", &self.remote_addr)
            .finish()
    }
}

impl Request {
    /// If the decoded URL of the request starts with `prefix`, builds a new `Request` that is
    /// the same as the original but without that prefix.
    pub fn remove_prefix(&self, prefix: &str) -> Option<Request> {
        if !self.url().starts_with(prefix) {
            return None;
        }

        // TODO: url-encoded characters in the prefix are not implemented
        assert!(self.url.starts_with(prefix));
        Some(Request {
            method: self.method.clone(),
            url: self.url[prefix.len()..].to_owned(),
            headers: self.headers.clone(), // TODO: expensive
            https: self.https,
            data: self.data.clone(),
            remote_addr: self.remote_addr,
        })
    }

    /// Returns `true` if the request uses HTTPS, and `false` if it uses HTTP.
    #[inline]
    pub fn is_secure(&self) -> bool {
        self.https
    }

    /// Returns the method of the request (`GET`, `POST`, etc.).
    #[inline]
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Returns the raw URL requested by the client. It is not decoded and thus can contain strings
    /// such as `%20`, and the query parameters such as `?p=hello`.
    ///
    /// See also `url()`.
    #[inline]
    pub fn raw_url(&self) -> &str {
        &self.url
    }

    /// Returns the raw query string requested by the client. In other words, everything after the
    /// first `?` in the raw url.
    ///
    /// Returns the empty string if no query string.
    #[inline]
    pub fn raw_query_string(&self) -> &str {
        if let Some(pos) = self.url.bytes().position(|c| c == b'?') {
            self.url.split_at(pos + 1).1
        } else {
            ""
        }
    }

    /// Returns the URL requested by the client.
    ///
    /// Contrary to `raw_url`, special characters have been decoded and the query string
    /// (eg `?p=hello`) has been removed.
    ///
    /// If there is any non-unicode character in the URL, it will be replaced with `U+FFFD`.
    ///
    /// > **Note**: This function will decode the token `%2F` will be decoded as `/`. However the
    /// > official specifications say that such a token must not count as a delimiter for URL paths.
    /// > In other words, `/hello/world` is not the same as `/hello%2Fworld`.
    ///
    /// # Example
    ///
    /// ```
    /// use rouille::Request;
    ///
    /// let request = Request::fake_http("GET", "/hello%20world?foo=bar", vec![], vec![]);
    /// assert_eq!(request.url(), "/hello world");
    /// ```
    pub fn url(&self) -> String {
        let url = self.url.as_bytes();
        let url = if let Some(pos) = url.iter().position(|&c| c == b'?') {
            &url[..pos]
        } else {
            url
        };

        percent_encoding::percent_decode(url)
            .decode_utf8_lossy()
            .into_owned()
    }

    /// Returns the value of a GET parameter or None if it doesn't exist.
    pub fn get_param(&self, param_name: &str) -> Option<String> {
        let name_pattern = &format!("{}=", param_name);
        let param_pairs = self.raw_query_string().split('&');
        param_pairs
            .filter(|pair| pair.starts_with(name_pattern) || pair == &param_name)
            .map(|pair| pair.split('=').nth(1).unwrap_or(""))
            .next()
            .map(|value| {
                percent_encoding::percent_decode(value.replace('+', " ").as_bytes())
                    .decode_utf8_lossy()
                    .into_owned()
            })
    }

    /// Returns the value of a header of the request.
    ///
    /// Returns `None` if no such header could be found.
    #[inline]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|&(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| &v[..])
    }

    /// Returns a list of all the headers of the request.
    #[inline]
    pub fn headers(&self) -> HeadersIter {
        HeadersIter {
            iter: self.headers.iter(),
        }
    }

    /// Returns the state of the `DNT` (Do Not Track) header.
    ///
    /// If the header is missing or is malformed, `None` is returned. If the header exists,
    /// `Some(true)` is returned if `DNT` is `1` and `Some(false)` is returned if `DNT` is `0`.
    ///
    /// # Example
    ///
    /// ```
    /// use rouille::{Request, Response};
    ///
    /// # fn track_user(request: &Request) {}
    /// fn handle(request: &Request) -> Response {
    ///     if !request.do_not_track().unwrap_or(false) {
    ///         track_user(&request);
    ///     }
    ///
    ///     // ...
    /// # panic!()
    /// }
    /// ```
    pub fn do_not_track(&self) -> Option<bool> {
        match self.header("DNT") {
            Some("1") => Some(true),
            Some("0") => Some(false),
            _ => None,
        }
    }

    /// Returns the body of the request.
    ///
    /// The body can only be retrieved once. Returns `None` is the body has already been retrieved
    /// before.
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Read;
    /// use rouille::{Request, Response, ResponseBody};
    ///
    /// fn echo(request: &Request) -> Response {
    ///     let mut data = request.data().expect("Oops, body already retrieved, problem \
    ///                                           in the server");
    ///
    ///     let mut buf = Vec::new();
    ///     match data.read_to_end(&mut buf) {
    ///         Ok(_) => (),
    ///         Err(_) => return Response::text("Failed to read body")
    ///     };
    ///
    ///     Response {
    ///         data: ResponseBody::from_data(buf),
    ///         .. Response::text("")
    ///     }
    /// }
    /// ```
    pub fn data(&self) -> Option<RequestBody> {
        let reader = self.data.lock().unwrap().take();
        reader.map(|r| RequestBody {
            body: r,
            marker: PhantomData,
        })
    }

    /// Returns the address of the client that made this request.
    ///
    /// # Example
    ///
    /// ```
    /// use rouille::{Request, Response};
    ///
    /// fn handle(request: &Request) -> Response {
    ///     Response::text(format!("Your IP is: {:?}", request.remote_addr()))
    /// }
    /// ```
    #[inline]
    pub fn remote_addr(&self) -> &SocketAddr {
        self.remote_addr
            .as_ref()
            .expect("Unexpected Unix socket for request")
    }
}


pub struct Server {
    handlers: HandlerMap,
    on_missing_method: Handler,
    listener: TcpListener,
    address: SocketAddr,
}

impl Server {
    pub fn new(port: u16) -> Server {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let listener = TcpListener::bind(addr).expect("Unable to start listener!");
        Self {
            handlers: HashMap::new(),
            on_missing_method: Box::new(on_missing_method),
            listener,
            address: addr
        }
    }

    pub fn run(&self) {
        todo!("Somehow this was missing?");
    }

    pub fn poll(&self) {
        todo!("somehow this was missing?");
    }

    pub fn server_addr(&self) -> &SocketAddr {
        &self.address
    }

    pub fn register_value<K, T>(&mut self, name: K, handler: T)
    where
        K: Into<String>,
        T: Fn(Vec<Value>) -> Response + Send + Sync + 'static,
    {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    pub fn register<'a, K, Treq, Tres, Thandler, Tef, Tdf>(
        &mut self,
        name: K,
        handler: Thandler,
        encode_fail: Tef,
        decode_fail: Tdf,
    ) where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Tres: Serialize,
        Thandler: Fn(Treq) -> std::result::Result<Tres, Fault> + Send + Sync + 'static,
        Tef: Fn(&error::Error) -> Response + Send + Sync + 'static,
        Tdf: Fn(&error::Error) -> Response + Send + Sync + 'static,
    {
        self.register_value(name, move |req| {
            let params = match from_params(req) {
                Ok(v) => v,
                Err(err) => return decode_fail(&err),
            };
            let response = handler(params)?;
            into_params(&response).or_else(|v| encode_fail(&v))
        });
    }

    pub fn register_simple<'a, K, Treq, Tres, Thandler>(&mut self, name: K, handler: Thandler)
    where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Tres: Serialize,
        Thandler: Fn(Treq) -> std::result::Result<Tres, Fault> + Send + Sync + 'static,
    {
        self.register(name, handler, on_encode_fail, on_decode_fail);
    }

    pub fn set_on_missing<T>(&mut self, handler: T)
    where
        T: Fn(Vec<Value>) -> Response + Send + Sync + 'static,
    {
        self.on_missing_method = Box::new(handler);
    }

    // todo - what is this suppose to do?
    pub fn bind<T: Into<SocketAddr>>(
        self,
        _uri: T,
    ) -> Result<Server>
    {
        // Trying to fix this plugin so blender doesn't have any compile time issue in the future updates.
        // three crates were marked depreciated and may stop build unless author of xml-rpc-rs can maintain their crate again.
        // I re-did some code here to use different library while maintaining similar API calls.
        // self.listener.incoming()
        // let sock: SocketAddr = uri.into();
        // let port = sock.port();
        // Server::new(port)

            // .map_err(|err| ErrorKind::BindFail(err.to_string()).into())
            // .map(BoundServer::new)
            // this function expects Ok(BoundServer)
        Ok(self)
    }

    // the request came from rouille, but rouille have dependency issues that needed to be resolved.
    // find a substitution replacement and used it instead.
    fn handle_outer(&self, request: &Request) -> Response {
        use super::xmlfmt::value::ToXml;

        // get the content of the body
        let body = match request.data {
            Some(data) => data.as_bytes().ok_or(Fault::empty()),
            // TODO: Check and see if reqwest does have a basic 400 default response types
            None => return Err(Fault::empty()),
        }?;

        // parse the body into xml call
        let call: Call = match parse::call(body) {
            Ok(data) => data,
            // TODO: Check and see if reqwest does have a basic 400 default response types
            Err(_err) => return Err(Fault::empty()),
        };

        // handle the xml callback
        let res = self.handle(call);
        let body = res.to_xml();

        // TODO: replace with reqwest appropriate response
        XML::from_data(Value::String("text/xml".into()), Value::String(body))
    }

    fn handle(&self, req: Call) -> Response {
        self.handlers
            .get(&req.name)
            .unwrap_or(&self.on_missing_method)(req.params)
    }
}

// I'm a bit confused why we need this generic to be async safe (send + sync + 'static) 
// but with a fn call that takes in request and returns Response
pub struct BoundServer
{
    server: Server,
}

impl BoundServer
{
    fn new(server: Server) -> Self {
        Self { server }
    }

    pub fn local_addr(&self) -> &SocketAddr {
        self.server.server_addr()
    }

    pub fn run(self) {
        self.server.run()
    }

    pub fn poll(&self) {
        self.server.poll()
    }
}

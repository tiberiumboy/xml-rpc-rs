use crate::xmlfmt::ToXml;
use crate::xmlfmt::{Param, ser, de, Params, Call, XmlError, Value, XmlResponse, MethodResponse, XmlResult, FmtError, on_decode_fail, on_encode_fail, from_params, into_params};
use serde::{Deserialize, Serialize};
// use ureq::http::response;
use std::collections::HashMap;
use std::io::Read;
use std::io::Result as IoResult;
use std::marker::PhantomData;
use std::net::SocketAddrV4;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::slice::Iter as SliceIter;
use tiny_http::{Request, Response, Server as TinyHttpServer};

// I need to provide a response back. - See if we can do this without async/mutex
// TODO: Do we need send + Sync? Is async ideal? Thread safe? Mutex?
type Handler = Box<dyn Fn(Params) -> XmlResponse + Send + Sync + 'static>;
type HandlerMap = HashMap<String, Handler>;

fn on_missing_method(e: Param) -> XmlResponse {
    Err(
        Value::fault(404, 
            format!("Requested method does not exist: {:?}", e)))
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

/*
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

// may not be in use? We'll see
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
            headers: self.headers.clone(), // TODO: expensive, can we consume?
            https: self.https,
            data: self.data.clone(), // TODO: Expensive? Can we consume?
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
    /// use tiny_http::Response;
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
    /// use tiny_http::Response;
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
    pub fn data(&'_ self) -> Option<RequestBody<'_>> {
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
    /// use tiny_http::Response;
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

enum Executor {
    Threaded { count: Arc<AtomicUsize> },
    Pooled { pool: threadpool::ThreadPool },
}

*/

// FIXME: Got lint warning complaining server and on_missing_method not in used.
pub struct Server {
    server: TinyHttpServer,
    // executor: Executor,
    handlers: HandlerMap,
    #[allow(dead_code)] // todo: find a way to use this?
    on_missing_method: Handler,
}

impl Default for Server {
    fn default() -> Self {
        let localhost = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8000));
        let server = TinyHttpServer::http(localhost).unwrap();
        Self {
            server,
            handlers: HashMap::new(),
            on_missing_method: Box::new(|e| on_missing_method(e.into())),
        }
    }
}

impl Server {
    pub fn new(port: u16) -> XmlResult<Server> {
        let localhost = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
        let server =
            TinyHttpServer::http(localhost).map_err(|e| XmlError::Server(e.to_string()))?;

        Ok(Self {
            server,
            handlers: HashMap::new(),
            on_missing_method: Box::new(|e| on_missing_method(e.into())),
        })
    }

    pub(crate) fn register_value<K, T>(&mut self, name: K, handler: T)
    where
        K: Into<String>,
        T: Fn(Params) -> XmlResponse + Send + Sync + 'static,
    {
        self.handlers.insert(name.into(), Box::new(handler));
    }

    pub fn register<'a, K, Treq, Thandler, Tef, Tdf>(
        &mut self,
        name: K,
        handler: Thandler,
        encode_fail: Tef,
        decode_fail: Tdf,
    ) where
        K: Into<String>,
        Treq: Deserialize<'a>,
        Thandler: Fn(Treq) -> XmlResponse + Send + Sync + 'static,
        Tef: Fn(&XmlError) -> XmlResponse + Send + Sync + 'static,
        Tdf: Fn(&XmlError) -> XmlResponse + Send + Sync + 'static,
    {
        self.register_value(name.into(), move |req| {
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
        Tres: serde::ser::Serialize,
        Thandler: Fn(Treq) -> XmlResponse + Send + Sync + 'static,
    {
        self.register(name, handler, on_encode_fail, on_decode_fail);
    }

    pub fn set_on_missing<T>(&mut self, handler: T)
    where
        T: Fn(Params) -> XmlResponse + Send + Sync + 'static,
    {
        self.on_missing_method = Box::new(handler);
    }

    // convert request into Call struct and invoke the method
    fn handle_outer(&self, request: &mut Request) -> XmlResponse {
        let call = match serde_xml_rs::from_reader(request.as_reader()) {
            Ok(data) => data,
            Err(e) => Err(Value::fault(-1, e.to_string())),
        }?;

        self.handle(call)
    }

    #[allow(dead_code)]
    fn poll(&self) {
        // think there's already a transport we could use?
        for mut request in self.server.incoming_requests() {
            let result: XmlResponse = self.handle_outer(&mut request);            
            
            let reply = MethodResponse::new(result);
            let content = match reply.to_xml() {
                Ok(str) => str,
                Err(val) => val.to_string(),
            };

            let response = Response::from_string(content);
            match request.respond(response) {
                Ok(data) => println!("Successfully responded! {data:?}"),
                Err(e) => println!("Fail to respond with this error message: {e:?}"),
            };
        }
    }

    /// Invoke subscribers matching handler names.
    /// Use this as a way to invoke methods or function on the python side of environment.
    fn handle(&self, req: Call) -> XmlResponse {
        match self.handlers.get(&req.name) {
            Some(v) => v(req.params.into()),
            None => Err(Value::fault(
                -1,
                format!("No handlers found for {}! Please register first!", &req.name),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_simple_server_should_succeed() {
        let server = Server::new(8001);
        assert!(server.is_ok());
        // I would assume that leaving this scope would free the server from being used?
    }

    #[test]
    fn server_should_fail_for_port_already_in_used() {
        let main_server = Server::new(8000);
        assert!(main_server.is_ok());

        let problem_server = Server::new(8000);
        assert!(problem_server.is_err());
    }
}

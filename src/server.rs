use serde::{Deserialize, Serialize};
use std;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use crate::xmlfmt::value::XML;  

use super::error::{ErrorKind, Result};
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

    fn handle_outer(&self, request: &Request) -> Response {
        use super::xmlfmt::value::ToXml;

        // get the content of the body
        let body = match request.body() {
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

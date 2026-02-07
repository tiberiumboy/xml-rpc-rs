#[macro_use]
extern crate serde;
extern crate serde_xml_rs;
pub extern crate ureq;

pub mod client;
pub mod server;
pub mod xmlfmt;

pub use client::{Client, call, call_value};
// pub use server::Server;
pub use xmlfmt::{MethodResponse, Params, Value, XmlError};

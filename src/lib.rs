#![recursion_limit = "1024"] // TODO: research this derive
extern crate base64;
extern crate url;// Do I need this, or is this being used by the server?
#[macro_use]
extern crate serde;
#[cfg(test)]
extern crate serde_bytes;
extern crate serde_xml_rs;
extern crate xml;
pub extern crate ureq;  // Is it safe to re-export extern crates?

pub mod client;
pub mod server;
mod xmlfmt;

pub use client::{call, call_value, Client};
pub use server::Server;
pub use xmlfmt::{XmlError, XmlResponse, Params, Value};

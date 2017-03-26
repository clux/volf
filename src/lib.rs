//! This is the rust doc for the `volf` *library* the github + jenkins interface
//! that the `volf` binary relies on to maintain state.

extern crate url;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate hyper;

// re-exports
pub use errors::{VolfError, VolfResult};
pub use pullrequest::{Pull, parse_commands};

pub mod client;
pub mod config;
pub mod server;

mod errors;
mod webhook;
mod pullrequest;

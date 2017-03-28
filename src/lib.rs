//! This is the rust doc for the `volf` *library* the github + jenkins interface
//! that the `volf` binary relies on to maintain state.

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate hyper;

extern crate hubcaps;

// re-exports
pub use errors::{VolfError, VolfResult};
pub use pullrequest::{Pull, parse_commands};

pub mod config;
pub mod server;

pub mod ci;

mod errors;
mod webhook;
mod pullrequest;

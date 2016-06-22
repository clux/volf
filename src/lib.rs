//! This is the rust doc for the `volf` *library* the github + jenkins interface
//! that the `volf` binary relies on to maintain state.

extern crate rustc_serialize;
#[macro_use]
extern crate log;

// re-exports
pub use config::Config;
pub use errors::{VolfError, VolfResult};

mod config;
mod errors;

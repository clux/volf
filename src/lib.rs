//! This is the rust doc for the `volf` *library* the github + jenkins interface
//! that the `volf` binary relies on to maintain state.

extern crate rustc_serialize;
extern crate url;

#[macro_use]
extern crate log;

#[macro_use]
extern crate hyper;

// re-exports
pub use config::Config;
pub use errors::{VolfError, VolfResult};
pub use pullrequest::{Pull, PullRequestState};

mod client;
mod config;
mod errors;
pub mod github;
mod pullrequest;

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
pub use github::webhook_handler;
pub use api::client::{Github, Credentials};

mod api;
mod config;
mod errors;
mod github;
mod pullrequest;

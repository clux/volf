use std::fmt;
use std::io;
use serde_json;
use json;
use hyper::Error as HttpError;
use hyper::status::StatusCode;

/// The one and only error type for the volf library
#[derive(Debug)]
pub enum VolfError {
    /// Miscellaneous errors propagated from `fs` and `process`
    Io(io::Error),
    /// Errors propagated from sedre
    Parse(serde_json::error::Error),
    /// Errors from other rcjson library
    Parse2(json::Error),
    /// Errors propagated from hyper
    Http(HttpError),
    /// Github API errors from client
    Client {
        code: StatusCode,
        error: json::JsonValue,
    },

    /// Config (volf.json) not found in current working directory
    MissingConfig,
    /// Config exists when expected not to
    ConfigExists,
    /// Misconfigured github webhooks - sends events we don't need
    SpammyGithub(String),
}

// Format implementation used when printing an error
impl fmt::Display for VolfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VolfError::Io(ref err) => err.fmt(f),
            VolfError::Parse(ref err) => err.fmt(f),
            VolfError::Parse2(ref err) => err.fmt(f),
            VolfError::Http(ref err) => err.fmt(f),
            VolfError::MissingConfig => write!(f, "Local config volf.json not found"),
            VolfError::ConfigExists => write!(f, "Local config volf.json exists"),
            VolfError::SpammyGithub(ref s) => write!(f, "{} events should not be sent to volf", s),
            VolfError::Client { ref error, ref code } => {
                write!(f, "{} - {}", code, json::stringify(error.clone()))
            }
        }
    }
}

impl From<io::Error> for VolfError {
    fn from(err: io::Error) -> VolfError { VolfError::Io(err) }
}

impl From<serde_json::error::Error> for VolfError {
    fn from(err: serde_json::error::Error) -> VolfError { VolfError::Parse(err) }
}

impl From<json::Error> for VolfError {
    fn from(err: json::Error) -> VolfError { VolfError::Parse2(err) }
}

impl From<HttpError> for VolfError {
    fn from(error: HttpError) -> VolfError { VolfError::Http(error) }
}

/// Type alias to stop having to type out VolfError everywhere.
pub type VolfResult<T> = Result<T, VolfError>;

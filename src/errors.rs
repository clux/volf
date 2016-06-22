use std::fmt;
use std::io;
use rustc_serialize::json;

/// The one and only error type for the volf library
#[derive(Debug)]
pub enum VolfError {
    /// Errors propagated from the `fs` module
    Io(io::Error),
    /// Errors propagated from rustc_serialize
    Parse(json::DecoderError),

    /// Config (volf.json) not found in current working directory
    MissingConfig,
}

// Format implementation used when printing an error
impl fmt::Display for VolfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            VolfError::Io(ref err) => err.fmt(f),
            VolfError::Parse(ref err) => err.fmt(f),
            VolfError::MissingConfig => write!(f, "Local config volf.json not found"),
        }
    }
}

// Allow io and json errors to be converted to VolfError in a try! without map_err
impl From<io::Error> for VolfError {
    fn from(err: io::Error) -> VolfError { VolfError::Io(err) }
}

impl From<json::DecoderError> for VolfError {
    fn from(err: json::DecoderError) -> VolfError { VolfError::Parse(err) }
}

/// Type alias to stop having to type out VolfError everywhere.
pub type VolfResult<T> = Result<T, VolfError>;

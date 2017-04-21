use errors::VolfResult;

// NB: Results expected to be notified back to the server.rs

/// Trait to map a required status to a build
pub trait Buildable {
    /// Start a build if necessary
    ///
    /// Called after auto branch is moved to where it needs to be.
    /// If CI is set to build on branch change on auto then this can be a noop.
    fn trigger(&self, build: &str) -> VolfResult<()>;

    /// Abort a build if possible
    ///
    /// Called if a user gives an abort command or someone rejects a PR in review.
    /// This is called for each build triggered.
    fn abort(&self, build: &str) -> VolfResult<()>;
}

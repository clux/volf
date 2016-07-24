use hyper::server::{Request, Response, Handler};
use hyper::status::StatusCode;
use hyper::method::Method;
use std::sync::{Arc, Mutex};

use super::Pull;
use super::config::Config;
use super::client::Github;

/// Convenience alias for main application state
pub type PullRequestState = Arc<Mutex<Vec<Pull>>>;

#[derive(Clone)]
pub struct ServerHandle {
    /// Shared state
    pub prs: PullRequestState,
    /// Shared github client instance
    pub gh: Arc<Github>,
    /// Shared Volf configuration data
    pub cfg: Arc<Config>,
}
impl ServerHandle {
    pub fn new(prs: PullRequestState, gh: Arc<Github>, cfg: Arc<Config>) -> ServerHandle {
        ServerHandle { prs: prs, gh: gh, cfg: cfg }
    }
}

// hyper interface
impl Handler for ServerHandle {
    fn handle(&self, req: Request, mut res: Response) {
        let uri = format!("{}", req.uri);
        if uri == "/github" && req.method == Method::Post {
            self.handle_webhook(req, res)
        } else {
            *res.status_mut() = StatusCode::MethodNotAllowed
        };
    }
}

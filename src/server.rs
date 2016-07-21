use hyper::server::{Request, Response, Handler};
use hyper::status::StatusCode;
use hyper::method::Method;
use std::sync::{Arc, Mutex};
use super::Pull;
use super::client::Github;

/// Convenience alias for main application state
pub type PullRequestState = Arc<Mutex<Vec<Pull>>>;

pub struct ServerHandle {
    /// Shared state
    pub prs: PullRequestState,
    /// Shared github client instance
    pub gh: Arc<Github>,
}
impl ServerHandle {
    pub fn new(prs: PullRequestState, gh: Arc<Github>) -> ServerHandle {
        ServerHandle {
            prs: prs,
            gh: gh,
        }
    }
}

impl Handler for ServerHandle {
    fn handle(&self, mut req: Request, mut res: Response) {
        let uri = format!("{}", req.uri);
        if uri == "/github" && req.method == Method::Post {
            self.handle_webhook(req, res)
        }
        else {
            *res.status_mut() = StatusCode::MethodNotAllowed
        };
    }
}

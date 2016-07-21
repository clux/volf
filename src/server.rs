use hyper::server::{Request, Response, Handler};
use hyper::status::StatusCode;
use hyper::method::Method;
use std::sync::{Arc, Mutex};
use super::Pull;

/// Convenience alias for main application state
pub type PullRequestState = Arc<Mutex<Vec<Pull>>>;

pub struct ServerHandle {
    pub prs: PullRequestState
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

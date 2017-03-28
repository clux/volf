use hyper::server::{Request, Response, Handler};
use hyper::status::StatusCode;
use hyper::method::Method;
use std::sync::{Arc, Mutex};
use std::io::Read;

use super::Pull;
use super::config::Config;
use super::{VolfResult};

use serde_json;
use hubcaps::Github;

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
        ServerHandle {
            prs: prs,
            gh: gh,
            cfg: cfg,
        }
    }
}

// hyper interface
impl Handler for ServerHandle {
    fn handle(&self, req: Request, mut res: Response) {
        let uri = format!("{}", req.uri);
        if uri == "/github" && req.method == Method::Post {
            self.handle_webhook(req, res)
        } else if uri == "/ci" && req.method == Method::Post {
            self.handle_ci(req, res)
        } else {
            *res.status_mut() = StatusCode::MethodNotAllowed
        };
    }
}


// -----------------------------------------------------------------------------

// homu expects less info, but ends up iterating through all kinds of global state
// to figure out what a build corresponds to, better to just codify this here:

/// Result data expected to be POST'd back to volf_url/ci at the end of a build
#[derive(Deserialize)]
pub struct BuildResult {
    /// Full owner/repo name
    pub repo: String,
    /// PR number: TODO: require? could just iterate through..
    pub number: u64,
    /// Changeset id of build (for sanity)
    pub sha: String,
    /// Whether the build succeeded
    pub success: bool,
}


/// Extra routes for CI
impl ServerHandle {
    fn handle_build_result(&self, payload: &str) -> VolfResult<()> {
        // 1. deserialize payload into BuildResult
        let res : BuildResult = serde_json::from_str(&payload)?;
        // 2. match up build name to a PR
        let mut prs = self.prs.lock().unwrap();
        if let Some(pr) = prs.iter_mut().find(|ref pr| pr.num == res.number && pr.repo == res.repo) {
            debug!("found corresponding pr {}", pr.num);
            // TODO: call status API to set required status on head_sha
            if res.success {
                //  - check status API to see what's left
                //  - if nothing left:
                //      - call pr.success() (merges and closes)
            } else {
                //pr.failure(); // move queue to next pr
            }
        } else {
            warn!("ignoring comment on untracked pr {}", res.number);
        }
        Ok(())
    }

    pub fn handle_ci(&self, mut req: Request, res: Response) {
        let mut payload = String::new();
        if let Ok(_) = req.read_to_string(&mut payload) {
            debug!("ci result: {}", payload);
            let _ = self.handle_build_result(&payload)
                    .map_err(|err| warn!("Failed to handle ci res {}", err));
        }
        res.send(b"ok").ok();
    }
}

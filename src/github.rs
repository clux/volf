use rustc_serialize::json;
use hyper::server::{Request, Response};
use std::io::Read;
use std::sync::{Arc, Mutex};
use super::{PullRequestState, Pull};

// -----------------------------------------------------------------------------
// Minor structs parts of various event types

#[derive(RustcDecodable, Debug)]
pub struct User {
    /// Unique github user name
    pub login: String,
}
#[derive(RustcDecodable, Debug)]
pub struct Repository {
    /// Owner and repo name joined by a slash
    pub full_name: String,
}

#[derive(RustcDecodable, Debug)]
pub struct Comment {
    /// User creating the comment
    pub user: User,
    /// Body of the comment
    pub body: String,
}

#[derive(RustcDecodable, Debug)]
pub struct PullRequestIssue {
    /// Unique PR number typically refernced by #n
    pub number: u64,
}

#[derive(RustcDecodable, Debug)]
pub struct Issue {
    /// Unique PR number typically refernced by #n
    pub number: u64,
    /// Body of the original issue
    pub body: String,
    /// Struct that is set if the Issue is a PR
    pub pull_request: Option<PullRequestIssue>
}

#[derive(RustcDecodable, Debug)]
pub struct PullRequestRef {
    /// Ref name (only works with serde atm due to reserved keyword..)
    // _ref: String,
    /// Changeset id
    pub sha: String,
    /// Owning user
    pub user: User,
    /// Respository containing the ref
    pub repo: Repository,
}

#[derive(RustcDecodable, Debug)]
pub struct PullRequestInner {
    /// Title text
    pub title: String,
    /// State open/closed
    pub state: String,
    /// User opening PR
    pub user: User,
    /// State of head (branch/fork)
    pub head: PullRequestRef,
    /// State of destination (master typically)
    pub base: PullRequestRef,
}

// -----------------------------------------------------------------------------
// Main Event types handled

/// Subset of github events that we need
#[derive(RustcDecodable, Debug)]
pub struct PullRequest {
    /// Action taken (opened/reopened/closed/assigned/unassigned)
    pub action: String,
    /// Unique PR number typically refernced by #n
    pub number: u64,
    /// All PR related data
    pub pull_request: PullRequestInner,
    /// Location of repository that contain the PR
    pub repository: Repository,
    /// Poster of PR
    pub sender: User,
    /// Body of PR (not sent as a normal Comment struct)
    pub body: String,
}
// review comments (think these are only comments on specific lines)
// ignore these for now
// PullRequestReviewComment {
//    /// Action taken (created is the only event we expect)
//    pub action: String,
//    /// Comment info
//    pub comment: Comment,
//    /// Repository of review comment
//    pub repository: Repository,
//    /// Sender of review comment
//    pub sender: User,
// }
#[derive(RustcDecodable, Debug)]
pub struct Push {
    /// Ref name  (only works with serde atm due to reserved keyword..)
    // _ref: String,
    /// Changeset id of last change pushed
    pub after: String,
    /// The sha before the push
    pub before: String,
    /// Repository pushed to
    pub repository: Repository,
    /// User sending the change
    pub sender: User, // we only use login anyway
}
#[derive(RustcDecodable, Debug)]
pub struct IssueComment {
    /// Action taken (created is the only action we expect)
    pub action: String,
    /// Comment data we actually care about
    pub comment: Comment,
    /// Related issue (contains the number, crucially)
    pub issue: Issue,
    // Repository the relavant issue was in
    pub repository: Repository,
    /// Sender of the comment
    pub sender: User,
}
#[derive(RustcDecodable, Debug)]
pub struct Ping {
    /// Github Zen
    pub zen: String,
}
// TODO: Status ? probably only needed if hooks talk to github directly

// -----------------------------------------------------------------------------
// event handler traits

pub trait PushHook: Send + Sync {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &Push);
}
pub trait PullRequestHook: Send + Sync {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &PullRequest);
}
pub trait IssueCommentHook: Send + Sync {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &IssueComment);
}
pub trait PingHook: Send + Sync {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &Ping);
}
impl<F> PushHook for F where F: Fn(&Mutex<Vec<Pull>>, &Push), F: Sync + Send {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &Push) {
        self(state, data)
    }
}
impl<F> PullRequestHook for F where F: Fn(&Mutex<Vec<Pull>>, &PullRequest), F: Sync + Send {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &PullRequest) {
        self(state, data)
    }
}
impl<F> IssueCommentHook for F where F: Fn(&Mutex<Vec<Pull>>, &IssueComment), F: Sync + Send {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &IssueComment) {
        self(state, data)
    }
}
impl<F> PingHook for F where F: Fn(&Mutex<Vec<Pull>>, &Ping), F: Sync + Send {
    fn handle(&self, state: &Mutex<Vec<Pull>>, data: &Ping) {
        self(state, data)
    }
}

// -----------------------------------------------------------------------------
// main event handler

/// A hub is a registry of hooks
//#[derive(Default)]
pub struct Hub {
    state: PullRequestState,
    push_hook: Option<Box<PushHook>>,
    pull_request_hook: Option<Box<PullRequestHook>>,
    issue_comment_hook: Option<Box<IssueCommentHook>>,
    ping_hook: Option<Box<PingHook>>,
}

impl Hub {
    /// construct a new hub instance
    pub fn new(state : PullRequestState) -> Hub {
        Hub {
            state: state,
            push_hook: None,
            pull_request_hook: None,
            issue_comment_hook: None,
            ping_hook: None,
        }
    }
    // register a hook handlers on an event
    pub fn on_push<H>(&mut self, hook: H) where H: PushHook + 'static {
        self.push_hook = Some(Box::new(hook));
    }
    pub fn on_pull_request<H>(&mut self, hook: H) where H: PullRequestHook + 'static {
        self.pull_request_hook = Some(Box::new(hook));
    }
    pub fn on_issue_comment<H>(&mut self, hook: H) where H: IssueCommentHook + 'static {
        self.issue_comment_hook = Some(Box::new(hook));
    }
    pub fn on_ping<H>(&mut self, hook: H) where H: PingHook + 'static {
        self.ping_hook = Some(Box::new(hook));
    }


    /// deliver an event to the registered hook via Handler
    fn deliver(&self, event: &str, payload: &str) {
        // probably is a nicer way to do this, but can't think of one atm..
        match event {
            "pull_request" => {
                if let Some(ref hook) = self.pull_request_hook {
                    if let Ok(res) = json::decode::<PullRequest>(&payload) {
                        debug!("github pull_request : {:?}", res);
                        hook.handle(&self.state.clone(), &res);
                    }
                }
            },
            "push" => {
                if let Some(ref hook) = self.push_hook {
                    if let Ok(res) = json::decode::<Push>(&payload) {
                        debug!("github push : {:?}", res);
                        hook.handle(&self.state.clone(), &res);
                    }
                }
            },
            "issue_comment" => {
                if let Some(ref hook) = self.issue_comment_hook {
                    if let Ok(res) = json::decode::<IssueComment>(&payload) {
                        debug!("github issue_comment : {:?}", res);
                        hook.handle(&self.state.clone(), &res);
                    }
                }
            },
            "ping" => {
                if let Some(ref hook) = self.ping_hook {
                    if let Ok(res) = json::decode::<Ping>(&payload) {
                        debug!("github ping : {:?}", res);
                        hook.handle(&self.state.clone(), &res);
                    }
                }
            },
            _ => warn!("{} event unhandled - you are sending more than you need", event),
        }
    }
}

// -----------------------------------------------------------------------------
// webhook server handler

/// signature for request
/// see [this document](https://developer.github.com/webhooks/securing/) for more information
header! {(XHubSignature, "X-Hub-Signature") => [String]}

/// name of Github event
/// see [this document](https://developer.github.com/webhooks/#events) for available types
header! {(XGithubEvent, "X-Github-Event") => [String]}

/// unique id for each delivery
header! {(XGithubDelivery, "X-Github-Delivery") => [String]}

/// server handler equivalent to a hyper::Handler
///
/// This is meant to be used by reroute and is thus not implementing Handler itself
impl Hub {
    pub fn handler(&self, mut req: Request, res: Response) {
        let mut payload = String::new();
        let headers = req.headers.clone();
        if let (Some(&XGithubEvent(ref event)),
                Some(&XGithubDelivery(ref id)),
                Some(&XHubSignature(ref signature))) = (headers.get::<XGithubEvent>(),
                                                        headers.get::<XGithubDelivery>(),
                                                        headers.get::<XHubSignature>()) {
            if let Ok(_) = req.read_to_string(&mut payload) {
                debug!("github event: {}", event);
                // TODO: verify signature sha1 value == sha1(github.secret)
                trace!("signature: {}", signature);
                trace!("id {}", id);
                self.deliver(event, &payload);
            }
        }
        res.send(b"ok").ok();
    }
}

use rustc_serialize::json;
use pencil::{Request, Response, PencilResult};
use std::io::Read;
use std::collections::HashMap;

use VolfResult;

// -----------------------------------------------------------------------------
// Minor structs parts of various event types

#[derive(RustcDecodable, Debug)]
struct User {
    /// Unique github user name
    login: String,
}
#[derive(RustcDecodable, Debug)]
struct Repository {
    /// Owner and repo name joined by a slash
    full_name: String,
}

#[derive(RustcDecodable, Debug)]
struct Comment {
    /// User creating the comment
    user: User,
    /// Body of the comment
    body: String,
}

#[derive(RustcDecodable, Debug)]
struct PullRequestIssue {
    /// Unique PR number typically refernced by #n
    number: u64,
}

#[derive(RustcDecodable, Debug)]
struct Issue {
    /// Unique PR number typically refernced by #n
    number: u64,
    /// Body of the original issue
    body: String,
    /// Struct that is set if the Issue is a PR
    pull_request: Option<PullRequestIssue>
}

#[derive(RustcDecodable, Debug)]
struct PullRequestRef {
    /// Ref name (only works with serde atm due to reserved keyword..)
    // _ref: String,
    /// Changeset id
    sha: String,
    /// Owning user
    user: User,
    /// Respository containing the ref
    repo: Repository,
}

#[derive(RustcDecodable, Debug)]
struct PullRequestInner {
    /// Title text
    title: String,
    /// State open/closed
    state: String,
    /// User opening PR
    user: User,
    /// State of head (branch/fork)
    head: PullRequestRef,
    /// State of destination (master typically)
    base: PullRequestRef,
}

// -----------------------------------------------------------------------------
// Main Event types handled

/// Subset of github events that we need
#[derive(RustcDecodable, Debug)]
pub struct PullRequest {
    /// Action taken (opened/reopened/closed/assigned/unassigned)
    action: String,
    /// Unique PR number typically refernced by #n
    number: u64,
    /// All PR related data
    pull_request: PullRequestInner,
    /// Location of repository that contain the PR
    repository: Repository,
    /// Poster of PR
    sender: User,
    /// Body of PR (not sent as a normal Comment struct)
    body: String,
}
// review comments (think these are only comments on specific lines)
// ignore these for now
// PullRequestReviewComment {
//    /// Action taken (created is the only event we expect)
//    action: String,
//    /// Comment info
//    comment: Comment,
//    /// Repository of review comment
//    repository: Repository,
//    /// Sender of review comment
//    sender: User,
// }
#[derive(RustcDecodable, Debug)]
pub struct Push {
    /// Ref name  (only works with serde atm due to reserved keyword..)
    // _ref: String,
    /// Changeset id of last change pushed
    after: String,
    /// The sha before the push
    before: String,
    /// Repository pushed to
    repository: Repository,
    /// User sending the change
    sender: User, // we only use login anyway
}
#[derive(RustcDecodable, Debug)]
pub struct IssueComment {
    /// Action taken (created is the only action we expect)
    action: String,
    /// Comment data we actually care about
    comment: Comment,
    /// Related issue (contains the number, crucially)
    issue: Issue,
    // Repository the relavant issue was in
    repository: Repository,
    /// Sender of the comment
    sender: User,
}
#[derive(RustcDecodable, Debug)]
pub struct Ping {
    /// Github Zen
    zen: String,
}
// TODO: Status ? probably only needed if hooks talk to github directly

// -----------------------------------------------------------------------------
// event handlers

fn handle_issue_comment(ev: &IssueComment) -> VolfResult<()> {
    if ev.action == "created" && ev.issue.pull_request.is_some() {
        info!("Comment on {}#{} by {} - {}",
            ev.repository.full_name,
            ev.issue.number,
            ev.sender.login,
            ev.comment.body);
    }
    Ok(())
}

fn handle_pull_request(ev: &PullRequest) -> VolfResult<()> {
    Ok(())
}

fn handle_push(ev: &Push) -> VolfResult<()> {
    Ok(())
}

fn handle_event(payload: &String, event: &str) -> VolfResult<()> {
    match event {
        "pull_request" => {
            let res: PullRequest = try!(json::decode(&payload));
            debug!("github pull_request : {:?}", res);
            try!(handle_pull_request(&res));
        }
        "push" => {
            let res: Push = try!(json::decode(&payload));
            debug!("github push : {:?}", res);
            try!(handle_push(&res));
        }
        "issue_comment" => {
            let res: IssueComment = try!(json::decode(&payload));
            debug!("github issue_comment : {:?}", res);
            try!(handle_issue_comment(&res));
        }
        "ping" => {
            let res: Ping = try!(json::decode(&payload));
            debug!("github ping event - '{}'", res.zen);
        }
        _ => warn!("{} event unhandled - you are sending more than you need", event),
    }
    Ok(())
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

/// Main webhook handler
pub fn hook(req: &mut Request) -> PencilResult {
    let mut payload = String::new();
    // Expect the three github headers:
    let headers = req.headers().clone();
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
            let _ = handle_event(&payload, event.as_str()).map_err(|e| {
                warn!("Failed to handle '{}' event", event);
                warn!("Caught {}", e);
            });
        }
    }
    Ok(Response::new_empty())
}


// -----------------------------------------------------------------------------
// experiment

/// Wrapper type for all handled events
//pub enum Event {
//    EPullRequest(PullRequest),
//    EPush(Push),
//    EPing(Ping),
//    EIssueComment(IssueComment),
//}

//#[derive(Debug)]
//pub struct Delivery<'a T> {
//    pub id: &'a str,
//    pub event: &'a str,
//    pub unparsed_payload: &'a str,
//}
//
//impl<'a T> Delivery<'a T> {
//    pub fn new(id: &'a str, event: &'a str) -> Option<Delivery<'a>> {
        //let patched = events::patch_payload_json(event, payload);
        //match serde_json::from_str::<Event>(&patched) {
        //    Ok(parsed) => {
        //        Some(Delivery {
        //            id: id,
        //            event: event,
        //            payload: parsed,
        //            unparsed_payload: payload,
        //            signature: signature,
        //        })
        //    }
        //    Err(e) => {
        //        error!(
        //            "failed to parse json {:?}\n{:#?}", e, patched);
        //        None
        //    },
        //}
//        None
//    }
//}


pub trait PushHook: Send + Sync {
    fn handle(&self, delivery: &Push);
}
pub trait PullRequestHook: Send + Sync {
    fn handle(&self, delivery: &PullRequest);
}
pub trait IssueCommentHook: Send + Sync {
    fn handle(&self, delivery: &IssueComment);
}
impl<F> PushHook for F where F: Fn(&Push), F: Sync + Send {
    fn handle(&self, delivery: &Push) {
        self(delivery)
    }
}
impl<F> PullRequestHook for F where F: Fn(&PullRequest), F: Sync + Send {
    fn handle(&self, delivery: &PullRequest) {
        self(delivery)
    }
}
impl<F> IssueCommentHook for F where F: Fn(&IssueComment), F: Sync + Send {
    fn handle(&self, delivery: &IssueComment) {
        self(delivery)
    }
}


/// A hub is a registry of hooks
#[derive(Default)]
pub struct Hub {
    push_hook: Option<Box<PushHook>>,
    pull_request_hook: Option<Box<PullRequestHook>>,
    issue_comment_hook: Option<Box<IssueCommentHook>>,
    //hooks: HashMap<String, Box<Hook>>,
}

impl Hub {
    /// construct a new hub instance
    pub fn new() -> Hub {
        Hub { ..Default::default() }
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


    /// deliver an event to the registered hook
    pub fn deliver(&self, event: &str, payload: &str) {
        match event {
            "pull_request" => {
                if let Some(ref hook) = self.pull_request_hook {
                    if let Ok(res) = json::decode::<PullRequest>(&payload) {
                        debug!("github pull_request : {:?}", res);
                        hook.handle(&res);
                    }
                }
            }
            //"push" => {
            //    let res: Push = try!(json::decode(&payload));
            //    debug!("github push : {:?}", res);
            //    try!(handle_push(&res));
            //}
            //"issue_comment" => {
            //    let res: IssueComment = try!(json::decode(&payload));
            //    debug!("github issue_comment : {:?}", res);
            //    try!(handle_issue_comment(&res));
            //}
            //"ping" => {
            //    let res: Ping = try!(json::decode(&payload));
            //    debug!("github ping event - '{}'", res.zen);
            //}
            _ => warn!("{} event unhandled - you are sending more than you need", event),
        }
        //if let Some(hook) = self.hook(delivery) {
        //    hook.handle(&delivery)
        //}
    }
}


//impl Handler for Hub {
//    fn handle(&self, mut req: Request, res: Response) {
//        let mut payload = String::new();
//        let headers = req.headers().clone();
//        if let (Some(&XGithubEvent(ref event)),
//                Some(&XGithubDelivery(ref id)),
//                Some(&XHubSignature(ref signature))) = (headers.get::<XGithubEvent>(),
//                                                        headers.get::<XGithubDelivery>(),
//                                                        headers.get::<XHubSignature>()) {
//            if let Ok(_) = req.read_to_string(&mut payload) {
//                debug!("github event: {}", event);
//                // TODO: verify signature sha1 value == sha1(github.secret)
//                trace!("signature: {}", signature);
//                trace!("id {}", id);
//
//                match Delivery::new(id, event) {
//                    Some(delivery) => self.deliver(&delivery),
//                    _ => {
//                        error!("failed to parse event {:?} for delivery {:?}",
//                               event,
//                               delivery)
//                    }
//                }
//            }
//        }
//        let _ = res.send(b"ok");
//        ()
//    }
//}

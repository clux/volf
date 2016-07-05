use rustc_serialize::json;
use hyper::server::{Request, Response};
use std::io::Read;
use super::{PullRequestState, Pull, VolfResult};

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
// event handlers

fn handle_push(_: &PullRequestState, _: &Push) -> VolfResult<()> {
    // TODO: need `ref` key here to match up with a pr
    Ok(())
}

fn handle_pull_request(state: &PullRequestState, data: &PullRequest) -> VolfResult<()> {
    info!("got pr {:?}", data);
    let prdata = &data.pull_request;
    if data.action == "opened" || data.action == "reopened" {
        let pr = Pull::new(&data.repository.full_name, &prdata.title, data.number);
        let mut prs = state.lock().unwrap();
        prs.push(pr);
    }
    Ok(())
}

fn handle_issue_comment(state: &PullRequestState, data: &IssueComment) -> VolfResult<()> {
    info!("got issue comment {:?}", data);
    if let Some(ref prdata) = data.issue.pull_request {
        if data.action == "created" {
            info!("Comment on {}#{} by {} - {}",
                data.repository.full_name,
                data.issue.number,
                data.sender.login,
                data.comment.body);
        }
        let mut prs = state.lock().unwrap();
        if let Some(pr) = prs.iter().find(|&pr| pr.num == prdata.number) {
            info!("found corresponding pr {}", pr.num);
        }
        else {
            warn!("ignoring comment on untracked pr {}", prdata.number);
        }
    }
    Ok(())
}

fn handle_ping(_: &PullRequestState, data: &Ping) -> VolfResult<()> {
    info!("Ping - {}", data.zen);
    Ok(())
}

// multiplex events
fn handle_event(state: &PullRequestState, event: &str, payload: &str) -> VolfResult<()> {
    match event {
        "pull_request" => {
            let res: PullRequest = try!(json::decode(&payload));
            trace!("github pull_request : {:?}", res);
            Ok(try!(handle_pull_request(state, &res)))
        }
        "push" => {
            let res: Push = try!(json::decode(&payload));
            trace!("github push : {:?}", res);
            Ok(try!(handle_push(state, &res)))
        }
        "issue_comment" => {
            let res: IssueComment = try!(json::decode(&payload));
            trace!("github issue_comment : {:?}", res);
            Ok(try!(handle_issue_comment(state, &res)))
        }
        "ping" => {
            let res: Ping = try!(json::decode(&payload));
            trace!("github ping event - '{}'", res.zen);
            Ok(try!(handle_ping(state, &res)))
        }
        _ => {
            warn!("{} event unhandled - you are sending more than you need", event);
            Ok(())
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

pub fn webhook_handler(state: &PullRequestState, mut req: Request, res: Response) {
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
            let _ = handle_event(state, event.as_str(), payload.as_str()).map_err(|err| {
                warn!("Failed to handle {} : {}", event, err)
            });
        }
    }
    res.send(b"ok").ok();
}

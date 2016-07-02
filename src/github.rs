use rustc_serialize::json;
use pencil::{Request, Response, PencilResult};
use std::io::Read;
use VolfResult;

// Small helper structs for Event

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
struct Issue {
    /// Unique PR number typically refernced by #n
    number: u64,
    /// Body of the original issue
    body: String,
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

/// Subset of github events that we need
#[derive(RustcDecodable, Debug)]
struct PullRequest {
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
// TODO: Status ? probably only needed if hooks talk to github directly

/// signature for request
/// see [this document](https://developer.github.com/webhooks/securing/) for more information
header! {(XHubSignature, "X-Hub-Signature") => [String]}

/// name of Github event
/// see [this document](https://developer.github.com/webhooks/#events) for available types
header! {(XGithubEvent, "X-Github-Event") => [String]}

/// unique id for each delivery
header! {(XGithubDelivery, "X-Github-Delivery") => [String]}


pub fn handle_event(payload: &String, event: &str) -> VolfResult<()> {
    debug!("handle event for {}", event);
    match event {
        "pull_request" => {
            let res: PullRequest = try!(json::decode(&payload));
            debug!("github pull_request : {:?}", res)
        }
        "push" => {
            let res: Push = try!(json::decode(&payload));
            debug!("github push : {:?}", res);
        }
        "issue_comment" => {
            let res: IssueComment = try!(json::decode(&payload));
            debug!("github issue_comment : {:?}", res)
        }
        _ => trace!("unhandled {} event", event),
    }
    Ok(())
}

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
            let _ = handle_event(&payload, event.as_str());
        }
    }
    Ok(Response::new_empty())
}

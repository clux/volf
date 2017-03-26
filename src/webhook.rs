use serde_json;
use hyper::server::{Request, Response};
use std::io::Read;
use super::{Pull, VolfResult, VolfError, parse_commands};
use super::server::ServerHandle;

// -----------------------------------------------------------------------------
// Minor structs parts of various event types

#[derive(Deserialize, Debug)]
pub struct User {
    /// Unique github user name
    pub login: String,
}
#[derive(Deserialize, Debug)]
pub struct Repository {
    /// Owner and repo name joined by a slash
    pub full_name: String,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    /// User creating the comment
    pub user: User,
    /// Body of the comment
    pub body: String,
}

#[derive(Deserialize, Debug)]
pub struct PullRequestIssue {
    /// Unique PR number typically refernced by #n
    pub number: u64,
}

#[derive(Deserialize, Debug)]
pub struct Issue {
    /// Unique PR number typically refernced by #n
    pub number: u64,
    /// Body of the original issue
    pub body: String,
    /// Struct that is set if the Issue is a PR
    pub pull_request: Option<PullRequestIssue>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
pub struct Ping {
    /// Github Zen
    pub zen: String,
}
// TODO: Status ? probably only needed if hooks talk to github directly

// -----------------------------------------------------------------------------
// event handlers

impl ServerHandle {
    fn handle_push(&self, _: Push) -> VolfResult<()> {
        // TODO: need `ref` key here to match up with a pr
        Ok(())
    }

    fn handle_pull_request(&self, data: PullRequest) -> VolfResult<()> {
        info!("got pr {:?}", data);
        let prdata = &data.pull_request;
        if data.action == "opened" || data.action == "reopened" {
            let pr = Pull::new(&data.repository.full_name, data.number, &prdata.title);
            let mut prs = self.prs.lock().unwrap();
            prs.push(pr);
        }
        Ok(())
    }

    fn handle_issue_comment(&self, data: IssueComment) -> VolfResult<()> {
        info!("got issue comment {:?}", data);
        if let Some(ref prdata) = data.issue.pull_request {
            if data.action == "created" {
                debug!("Comment on {}#{} by {} - {}",
                       data.repository.full_name,
                       data.issue.number,
                       data.sender.login,
                       data.comment.body);
            }
            let mut prs = self.prs.lock().unwrap();
            if let Some(pr) = prs.iter_mut().find(|ref pr| pr.num == prdata.number) {
                debug!("found corresponding pr {}", pr.num);
                parse_commands(pr, data.comment.body, data.sender.login);
            } else {
                warn!("ignoring comment on untracked pr {}", prdata.number);
            }
        }
        Ok(())
    }

    fn handle_ping(&self, data: Ping) -> VolfResult<()> {
        info!("Ping - {}", data.zen);
        Ok(())
    }

    /// Event multiplexer
    pub fn handle_event(&self, event: &str, payload: &str) -> VolfResult<()> {
        match event {
            "issue_comment" => self.handle_issue_comment(serde_json::from_str(&payload)?),
            "pull_request" => self.handle_pull_request(serde_json::from_str(&payload)?),
            "push" => self.handle_push(serde_json::from_str(&payload)?),
            "ping" => self.handle_ping(serde_json::from_str(&payload)?),
            _ => Err(VolfError::SpammyGithub(event.into())),
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


/// A Handler equivalent implementation for our state struct
impl ServerHandle {
    pub fn handle_webhook(&self, mut req: Request, res: Response) {
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
                let _ = self.handle_event(&event, &payload)
                    .map_err(|err| warn!("Failed to handle {} : {}", event, err));
            }
        }
        res.send(b"ok").ok();
    }
}

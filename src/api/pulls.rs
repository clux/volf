use super::client::Github;
use rustc_serialize::json;
use std::default::Default;
use std::fmt;

#[derive(Debug, RustcDecodable)]
pub struct Commit {
    pub label: String,
    //#[serde(rename="ref")]
    //pub commit_ref: String,
    pub sha: String,
}

#[derive(Debug, RustcDecodable)]
pub struct Pull {
    pub id: u64,
    pub url: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,
    pub head: Commit,
    pub base: Commit,
    pub merge_commit_sha: Option<String>,
    pub mergeable: Option<bool>,
    pub comments: Option<u64>,
}


#[derive(Debug, RustcDecodable)]
pub struct PullOptions {
    pub title: String,
    pub head: String,
    pub base: String,
    //#[serde(skip_serializing_if="Option::is_none")]
    pub body: Option<String>,
}

impl PullOptions {
    pub fn new<T, H, BS, B>(title: T, head: H, base: BS, body: Option<B>) -> PullOptions
        where T: Into<String>,
              H: Into<String>,
              BS: Into<String>,
              B: Into<String>
    {
        PullOptions {
            title: title.into(),
            head: head.into(),
            base: base.into(),
            body: body.map(|b| b.into()),
        }
    }
}


/// A structure for accessing interfacing with a specific pull request
pub struct PullRequest<'a> {
    github: &'a Github<'a>,
    owner: String,
    repo: String,
    number: u64,
}

impl<'a> PullRequest<'a> {
    pub fn new<O, R>(github: &'a Github<'a>, owner: O, repo: R, number: u64) -> PullRequest<'a>
        where O: Into<String>,
              R: Into<String>
    {
        PullRequest {
            github: github,
            owner: owner.into(),
            repo: repo.into(),
            number: number,
        }
    }

    fn path(&self, more: &str) -> String {
        format!("/repos/{}/{}/pulls/{}{}",
                self.owner,
                self.repo,
                self.number,
                more)
    }

    ///// Request a pull requests information
    //pub fn get(&self) -> Result<Pull> {
    //    self.github.get::<Pull>(&self.path(""))
    //}

    ///// shorthand for editing state = closed
    //pub fn close(&self) -> Result<Pull> {
    //    self.edit(&PullEditOptions::builder().state("closed").build())
    //}
}

/// A structure for interfacing with a repositories list of pull requests
pub struct PullRequests<'a> {
    github: &'a Github<'a>,
    owner: String,
    repo: String,
}

impl<'a> PullRequests<'a> {
    pub fn new<O, R>(github: &'a Github<'a>, owner: O, repo: R) -> PullRequests<'a>
        where O: Into<String>,
              R: Into<String>
    {
        PullRequests {
            github: github,
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    fn path(&self, more: &str) -> String {
        format!("/repos/{}/{}/pulls{}", self.owner, self.repo, more)
    }

    /// Get a reference to a strucuture for interfacing with a specific pull request
    pub fn get(&self, number: u64) -> PullRequest {
        PullRequest::new(self.github, self.owner.as_ref(), self.repo.as_ref(), number)
    }

    ///// Create a new pull request
    //pub fn create(&self, pr: &PullOptions) -> Result<Pull> {
    //    let data = try!(serde_json::to_string(&pr));
    //    self.github.post::<Pull>(&self.path(""), data.as_bytes())
    //}

    ///// list pull requests
    //pub fn list(&self, options: &PullListOptions) -> Result<Vec<Pull>> {
    //    let mut uri = vec![self.path("")];
    //    if let Some(query) = options.serialize() {
    //        uri.push(query);
    //    }
    //    self.github.get::<Vec<Pull>>(&uri.join("?"))
    //}
}

use std::collections::HashMap;
use url::form_urlencoded;
use rustc_serialize::json;
use super::client::Github;
use super::pulls::{PullRequests, Commit};

// -----------------------------------------------------------------------------
// deserializable structs:
#[derive(Debug, RustcDecodable)]
pub struct Repo {
    pub id: u64,
    pub full_name: String,
    pub default_branch: String,
    pub created_at: String,
    pub updated_at: String,
}

// -----------------------------------------------------------------------------
// api helpers
pub struct Repositories<'a> {
    github: &'a Github<'a>,
}

#[derive(Default)]
pub struct RepoListOptions {
    params: HashMap<&'static str, String>
}
impl RepoListOptions {
    /// serialize options as a string. returns None if no options are defined
    pub fn serialize(&self) -> Option<String> {
        if self.params.is_empty() {
            None
        } else {
            let encoded: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(&self.params)
                .finish();
            Some(encoded)
        }
    }
}


impl<'a> Repositories<'a> {
    pub fn new(github: &'a Github<'a>) -> Repositories<'a> {
        Repositories { github: github }
    }

    fn path(&self, more: &str) -> String {
        format!("/user/repos{}", more)
    }

    ///// list the authenticated users repositories
    ///// https://developer.github.com/v3/repos/#list-your-repositories
    //pub fn list(&self, options: &RepoListOptions) -> Result<Vec<Repo>> {
    //    let mut uri = vec![self.path("")];
    //    if let Some(query) = options.serialize() {
    //        uri.push(query);
    //    }
    //    self.github.get::<Vec<Repo>>(&uri.join("?"))
    //}
}

//#[derive(Default)]
//pub struct UserRepoListOptions {
//    params: HashMap<&'static str, String>
//}
//impl UserRepoListOptions {
//    /// serialize options as a string. returns None if no options are defined
//    pub fn serialize(&self) -> Option<String> {
//        if self.params.is_empty() {
//            None
//        } else {
//            let encoded: String = form_urlencoded::Serializer::new(String::new())
//                .extend_pairs(&self.params)
//                .finish();
//            Some(encoded)
//        }
//    }
//}
//
///// Provides access to the authenticated user's repositories
//pub struct UserRepositories<'a> {
//    github: &'a Github<'a>,
//    owner: String,
//}
//
//impl<'a> UserRepositories<'a> {
//    pub fn new<O>(github: &'a Github<'a>, owner: O) -> UserRepositories<'a>
//        where O: Into<String>
//    {
//        UserRepositories {
//            github: github,
//            owner: owner.into(),
//        }
//    }
//
//    fn path(&self, more: &str) -> String {
//        format!("/users/{}/repos{}", self.owner, more)
//    }
//
//    /// https://developer.github.com/v3/repos/#list-user-repositories
//    pub fn list(&self, options: &UserRepoListOptions) -> Result<Vec<Repo>> {
//        let mut uri = vec![self.path("")];
//        if let Some(query) = options.serialize() {
//            uri.push(query);
//        }
//        self.github.get::<Vec<Repo>>(&uri.join("?"))
//    }
//}

pub struct Repository<'a> {
    github: &'a Github<'a>,
    owner: String,
    repo: String,
}

impl<'a> Repository<'a> {
    pub fn new<O, R>(github: &'a Github<'a>, owner: O, repo: R) -> Repository<'a>
        where O: Into<String>,
              R: Into<String>
    {
        Repository {
            github: github,
            owner: owner.into(),
            repo: repo.into(),
        }
    }

    ///// get a reference to a specific github issue associated with this repoistory ref
    //pub fn issue(&self, number: u64) -> IssueRef {
    //    IssueRef::new(self.github, self.owner.as_ref(), self.repo.as_ref(), number)
    //}

    ///// get a list of labels associated with this repository ref
    //pub fn labels(&self) -> Labels {
    //    Labels::new(self.github, self.owner.as_ref(), self.repo.as_ref())
    //}

    /// get a list of [pulls](https://developer.github.com/v3/pulls/)
    /// associated with this repository ref
    pub fn pulls(&self) -> PullRequests {
        PullRequests::new(self.github, self.owner.as_ref(), self.repo.as_ref())
    }

    ///// get a references to [statuses](https://developer.github.com/v3/repos/statuses/)
    ///// associated with this reposoitory ref
    //pub fn statuses(&self) -> Statuses {
    //    Statuses::new(self.github, self.owner.as_ref(), self.repo.as_ref())
    //}
}

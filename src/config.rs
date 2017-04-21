use serde_json;

use std::path::Path;
use std::fs;
use std::vec::Vec;
use std::io::prelude::{Read, Write};
use std::process::Command;
use std::env;
use std::sync::Arc;
use errors::{VolfError, VolfResult};
use super::{Pull, parse_commands};

use hubcaps::Github;

/// Repository data
#[derive(Serialize, Deserialize, Clone)]
pub struct Repository {
    /// Repository owner + name
    pub name: String,
    /// Required status builds (with same name)
    pub required_builds: Vec<String>,
    /// Optional status builds (with same name)
    pub optional_builds: Vec<String>,
    /// Github secret
    pub github_secret: String,
}

impl Repository {
    pub fn synchronize(&self, gh: Arc<Github>, repo: &str) -> VolfResult<Vec<Pull>> {
        use hubcaps::issues::State;
        use hubcaps::pulls::{PullRequests, PullListOptionsBuilder};
        use hubcaps::comments::{Comments, CommentListOptions};
        // TODO:  wipe state related to this repo!

        // GET request to repos/{}/issues
        let params = PullListOptionsBuilder::new().state(State::Open).build();
        let repoz = repo.split('/').collect::<Vec<_>>();
        let issues = PullRequests::new(&gh, repoz[0], repoz[1]);
        let issue_list = issues.list(&params)?;

        // state to replace old state with..
        let mut result_list = vec![];

        for issue in issue_list {
            println!("Found PR: {:?}", issue);
            //   - create Pull struct instance
            let mut pr = Pull::new("clux/volf", issue.id, &issue.title);

            //   - parse command on issue body
            let comments = Comments::new(&gh, repoz[0], repoz[1], issue.id);
            let comment_list = comments.list(&CommentListOptions::default())?;

            for comment in comment_list {
                println!(" - {}: {}", comment.user.login, comment.body);
                parse_commands(&mut pr, comment.body, comment.user.login);
            }
            // TODO: parse github reviews: https://developer.github.com/v3/pulls/reviews/
            // need to add this to hubcaps - not supported atm it looks like.

            result_list.push(pr);

        }
        Ok(result_list)
    }
}

/// Github specific tokens and data
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct GithubData {
    /// Personal access token for volf app host
    pub access_token: String,
    /// Client id for volf app
    pub app_client_id: String,
    /// Client secret for volf app
    pub app_client_secret: String,
}

/// Representation of `volf.json`
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// Port to listen on
    pub port: u32,

    /// Github tokens and client
    pub github: GithubData,

    // TOOD: CI usernames, tokens and urls
    /// Repositories to watch
    pub repositories: Vec<Repository>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 54857,
            github: GithubData::default(),
            repositories: vec![],
        }
    }
}

impl Config {
    /// Read and deserialize a Config from volf.json
    pub fn read() -> VolfResult<Config> {
        let cfg_path = Path::new("volf.json");
        if !cfg_path.exists() {
            return Err(VolfError::MissingConfig);
        }
        let mut f = fs::File::open(&cfg_path)?;
        let mut cfg_str = String::new();
        f.read_to_string(&mut cfg_str)?;
        let res: Config = serde_json::from_str(&cfg_str)?;
        Ok(res)
    }

    pub fn generate() -> VolfResult<()> {
        let cfg_path = Path::new("volf.json");
        if cfg_path.exists() {
            return Err(VolfError::ConfigExists);
        }
        let cfg = Config::default();
        let encoded = serde_json::to_string_pretty(&cfg)?;

        let mut f = fs::File::create(&cfg_path)?;
        write!(f, "{}\n", encoded)?;
        info!("Wrote config {}: \n{}", cfg_path.display(), encoded);
        Ok(())
    }

    pub fn edit() -> VolfResult<()> {
        let editor = env::var("EDITOR")
            .map_err(|e| {
                error!("Could not find $EDITOR - {}", e);
            })
            .unwrap();
        Command::new(editor).arg("volf.json").status()?;
        Ok(())
    }
}

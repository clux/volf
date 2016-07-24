use rustc_serialize::json;
use std::path::Path;
use std::fs;
use std::vec::Vec;
use std::io::prelude::{Read, Write};
use std::process::Command;
use std::env;
use std::sync::Arc;
use errors::{VolfError, VolfResult};
use super::client::Github;

/// Repository data
#[derive(RustcDecodable, RustcEncodable, Clone)]
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
    pub fn synchronize(&self, gh: Arc<Github>, repo: &str) -> VolfResult<()> {
        // First wipe state related to this repo!
        // GET request to repos/{}/issues
        let res = try!(gh.issues(repo));
        println!("res {:?}", res);
        // for each of those that are OPEN PRs:
        //   - create Pull struct instance
        //   - parse command on issue body
        //   - parse command on each issue comment
        Ok(())
    }
}

/// Github specific tokens and data
#[derive(RustcDecodable, RustcEncodable, Default, Clone)]
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
#[derive(RustcDecodable, RustcEncodable, Clone)]
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
            repositories: vec![]
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
        let mut f = try!(fs::File::open(&cfg_path));
        let mut cfg_str = String::new();
        try!(f.read_to_string(&mut cfg_str));
        let res = try!(json::decode(&cfg_str));
        Ok(res)
    }

    pub fn generate() -> VolfResult<()> {
        let cfg_path = Path::new("volf.json");
        if cfg_path.exists() {
            return Err(VolfError::ConfigExists);
        }
        let cfg = Config::default();
        let encoded = json::as_pretty_json(&cfg);

        let mut f = try!(fs::File::create(&cfg_path));
        try!(write!(f, "{}\n", encoded));
        info!("Wrote config {}: \n{}", cfg_path.display(), encoded);
        Ok(())
    }

    pub fn edit() -> VolfResult<()> {
        let editor = env::var("EDITOR").map_err(|e| {
            error!("Could not find $EDITOR - {}", e);
        }).unwrap();
        try!(Command::new(editor).arg("volf.json").status());
        Ok(())
    }
}

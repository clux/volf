use rustc_serialize::json;
use std::path::Path;
use std::fs;
use std::vec::Vec;
use std::io::prelude::*;
use errors::{VolfError, VolfResult};

/// Repository data
#[derive(RustcDecodable)]
pub struct Repository {
    /// Repository owner
    pub owner: String,
    /// Repository name
    pub name: String,

    /// Required status builds (with same name)
    pub required_builds: Vec<String>,
    /// Optional status builds (with same name)
    pub optional_builds: Vec<String>,
    /// Github secret
    pub github_secret: String,
}

/// Github specific tokens and data
#[derive(RustcDecodable)]
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
#[derive(RustcDecodable)]
pub struct Config {
    /// Port to listen on
    pub port: u32,

    /// Github tokens and client
    pub github: GithubData,

    // TOOD: CI usernames, tokens and urls
    /// Repositories to watch
    pub repositories: Vec<Repository>,
}

impl Config {
    /// Read and deserialize a Config from volf.json
    pub fn read() -> VolfResult<Config> {
        let cfg_path = Path::new(".").join("volf.json");
        if !cfg_path.exists() {
            return Err(VolfError::MissingConfig);
        }
        let mut f = try!(fs::File::open(&cfg_path));
        let mut cfg_str = String::new();
        try!(f.read_to_string(&mut cfg_str));
        let res = try!(json::decode(&cfg_str));
        Ok(res)
    }
}

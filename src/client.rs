use hyper::Client;
use hyper::client::RequestBuilder;
use hyper::method::Method;
use hyper::header::{Authorization, ContentLength, UserAgent};
use hyper::status::StatusCode;
use std::io::Read;
//use url::Url;
use json::{self, JsonValue};
use super::{VolfResult, VolfError};
//use super::config as cfg;

const DEFAULT_HOST: &'static str = "https://api.github.com";

/// Entry point interface for interacting with Github API
#[derive(Debug)]
pub struct Github {
    host: String,
    agent: String,
    client: Client,
    token: String,
}

impl Github {
    /// Create a new Github instance
    pub fn new<A, T>(agent: A, client: Client, token: T) -> Github
        where A: Into<String>,
              T: Into<String>
    {
        Github::host(DEFAULT_HOST, agent, client, token)
    }

    /// Create a new Github instance hosted at a custom location.
    /// Useful for github enterprise installations ( yourdomain.com/api/v3/ )
    pub fn host<H, A, T>(host: H, agent: A, client: Client, token: T) -> Github
        where H: Into<String>,
              A: Into<String>,
              T: Into<String>
    {
        Github {
            host: host.into(),
            agent: agent.into(),
            client: client,
            token: token.into(),
        }
    }


    fn authenticate(&self, method: Method, uri: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.host, uri);
        info!("req to uri {}", url);
        self.client.request(method, &url).header(Authorization(format!("token {}", self.token)))
    }
    // TODO: accept v3 header

    fn request(&self, method: Method, uri: &str, body: Option<&str>) -> VolfResult<JsonValue> {
        let builder = self.authenticate(method, uri).header(UserAgent(self.agent.to_owned()));
        let mut res = match body {
            Some(ref bod) => builder.body(*bod).send(),
            _ => builder.send(),
        }?;
        let mut body = match res.headers.clone().get::<ContentLength>() {
            Some(&ContentLength(len)) => String::with_capacity(len as usize),
            _ => String::new(),
        };
        res.read_to_string(&mut body)?;
        debug!("rev response {:#?} {:#?} {:#?}",
               res.status,
               res.headers,
               body);
        match res.status {
            StatusCode::Conflict |
            StatusCode::BadRequest |
            StatusCode::UnprocessableEntity |
            StatusCode::Unauthorized |
            StatusCode::NotFound |
            StatusCode::Forbidden => {
                Err(VolfError::Client {
                        code: res.status,
                        error: json::parse(&body)?,
                    })
            }
            _ => {
                if body.len() > 0 {
                    Ok(json::parse(&body)?)
                } else {
                    // allow empty bodies (from test ping)
                    Ok(json::Null)
                }
            }
        }
    }

    fn get(&self, uri: &str) -> VolfResult<JsonValue> {
        self.request(Method::Get, uri, None)
    }

    fn post(&self, uri: &str, message: &str) -> VolfResult<JsonValue> {
        self.request(Method::Post, uri, Some(message))
    }

    fn patch(&self, uri: &str, message: &str) -> VolfResult<JsonValue> {
        self.request(Method::Patch, uri, Some(message))
    }

    fn put(&self, uri: &str, message: &str) -> VolfResult<JsonValue> {
        self.request(Method::Put, uri, Some(message))
    }

    /// Make a comment on an issue
    pub fn comment(&self, repo: &str, issue: u64, message: &str) -> VolfResult<JsonValue> {
        let uri = format!("repos/{}/issues/{}/comments", repo, issue);
        let data = object!{
            "body" => message
        };
        self.post(&uri, &json::stringify(data))
    }

    /// Fetch issues
    pub fn issues(&self, repo: &str) -> VolfResult<JsonValue> {
        let uri = format!("repos/{}/issues", repo);
        // TODO: only get OPEN issues
        self.get(&uri)
    }

    /// Update the hook
    pub fn hook_update(&self, repo: &str, hook: u64) -> VolfResult<()> {
        let uri = format!("repos/{}/hooks/{}", repo, hook);
        let data = object!{
            "content_type" => "json",
            "url" => "http://109.146.235.224:54857/github",
            "insecure_ssl" => "0",
            "secret" => "hunter2"
        };
        self.patch(&uri, &json::stringify(data))?;
        Ok(())
    }
    /// Test the ping hook
    pub fn ping(&self, repo: &str, hook: u64) -> VolfResult<()> {
        let uri = format!("repos/{}/hooks/{}/pings", repo, hook);
        self.post(&uri, "")?;
        Ok(())
    }
}

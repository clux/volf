use hyper::Client;
use hyper::client::RequestBuilder;
use hyper::method::Method;
use hyper::header::{Authorization, ContentLength, UserAgent};
use hyper::status::StatusCode;
use std::io::Read;
use url::Url;
use json::{self, JsonValue};
use super::{VolfResult, VolfError};
use super::config as cfg;

const DEFAULT_HOST: &'static str = "https://api.github.com";


/// Various forms of authentication credentials supported by Github
#[derive(Debug, PartialEq)]
pub enum Credentials {
    /// No authentication
    None,
    /// Oauth token string
    /// https://developer.github.com/v3/#oauth2-token-sent-in-a-header
    Token(String),
    /// Oauth client id and secret
    /// https://developer.github.com/v3/#oauth2-keysecret
    Client(String, String),
}

/// Entry point interface for interacting with Github API
pub struct Github<'a> {
    host: String,
    agent: String,
    client: &'a Client,
    credentials: Credentials,
}

impl<'a> Github<'a> {
    /// Create a new Github instance
    pub fn new<A>(agent: A, client: &'a Client, credentials: Credentials) -> Github<'a>
        where A: Into<String>
    {
        Github::host(DEFAULT_HOST, agent, client, credentials)
    }

    /// Create a new Github instance hosted at a custom location.
    /// Useful for github enterprise installations ( yourdomain.com/api/v3/ )
    pub fn host<H, A>(host: H, agent: A, client: &'a Client, credentials: Credentials) -> Github<'a>
        where H: Into<String>,
              A: Into<String>
    {
        Github {
            host: host.into(),
            agent: agent.into(),
            client: client,
            credentials: credentials,
        }
    }


    fn authenticate(&self, method: Method, uri: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.host, uri);
        info!("req to uri {}", url);
        match self.credentials {
            Credentials::Token(ref token) => {
                self.client.request(method, &url).header(Authorization(format!("token {}", token)))
            }
            Credentials::Client(ref id, ref secret) => {

                let mut parsed = Url::parse(&url).unwrap();
                parsed.query_pairs_mut()
                    .append_pair("client_id", id)
                    .append_pair("client_secret", secret);
                self.client.request(method, parsed)
            }
            Credentials::None => self.client.request(method, &url),
        }
    }
    // TODO: accept v3 header

    fn request(&self, method: Method, uri: &str, body: Option<&'a str>) -> VolfResult<JsonValue> {
        let builder = self.authenticate(method, uri).header(UserAgent(self.agent.to_owned()));
        let mut res = try!(match body {
            Some(ref bod) => builder.body(*bod).send(),
            _ => builder.send(),
        });
        let mut body = match res.headers.clone().get::<ContentLength>() {
            Some(&ContentLength(len)) => String::with_capacity(len as usize),
            _ => String::new(),
        };
        try!(res.read_to_string(&mut body));
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
                    error: try!(json::parse(&body)),
                })
            }
            _ => {
                if body.len() > 0 {
                    Ok(try!(json::parse(&body)))
                } else {
                    // allow empty bodies (from test ping)
                    Ok(json::Null)
                }
            },
        }
    }

    fn get(&self, uri: &str) -> VolfResult<JsonValue> { self.request(Method::Get, uri, None) }

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

    /// Update the hook
    pub fn hook_update(&self, repo: &str, hook: u64) -> VolfResult<()> {
        let uri = format!("repos/{}/hooks/{}", repo, hook);
        let data = object!{
            "content_type" => "json",
            "url" => "http://109.146.235.224:54857/github",
            "insecure_ssl" => "0",
            "secret" => "hunter2"
        };
        try!(self.patch(&uri, &json::stringify(data)));
        Ok(())
    }
    /// Test the ping hook
    pub fn ping(&self, repo: &str, hook: u64) -> VolfResult<()> {
        let uri = format!("repos/{}/hooks/{}/pings", repo, hook);
        try!(self.post(&uri, ""));
        Ok(())
    }
}

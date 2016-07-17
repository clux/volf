use hyper::Client;
use hyper::client::RequestBuilder;
use hyper::method::Method;
use hyper::header::{Authorization, ContentLength, UserAgent};
use hyper::status::StatusCode;
use std::fmt;
use std::io::Read;
use url::Url;
// TODO: intermediate error type?
use {VolfResult, VolfError};

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

    ///// Return a reference to a Github reposistory
    //pub fn repo<O, R>(&self, owner: O, repo: R) -> Repository
    //    where O: Into<String>,
    //          R: Into<String>
    //{
    //    Repository::new(self, owner, repo)
    //}

    ///// Return a reference to the collection of repositories owned by and
    ///// associated with an owner
    //pub fn user_repos<S>(&self, owner: S) -> UserRepositories
    //    where S: Into<String>
    //{
    //    UserRepositories::new(self, owner)
    //}

    ///// Return a reference to the collection of repositories owned by the user
    ///// associated with the current authentication credentials
    //pub fn repos(&self) -> Repositories {
    //    Repositories::new(self)
    //}

    fn authenticate(&self, method: Method, uri: &str) -> RequestBuilder {
        let url = format!("{}{}", self.host, uri);
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

    fn request<D>(&self, method: Method, uri: &str, body: Option<&'a [u8]>) -> VolfResult<D>
    {
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
        //match res.status {
        //    StatusCode::Conflict |
        //    StatusCode::BadRequest |
        //    StatusCode::UnprocessableEntity |
        //    StatusCode::Unauthorized |
        //    StatusCode::NotFound |
        //    StatusCode::Forbidden => {
        //        Err(Error::Fault {
        //            code: res.status,
        //            error: try!(serde_json::from_str::<ClientError>(&body)),
        //        })
        //    }
        //    _ => Ok(try!(serde_json::from_str::<D>(&body))),
        //}
        unimplemented!()
    }

    fn get<D>(&self, uri: &str) -> VolfResult<D>
    {
        self.request(Method::Get, uri, None)
    }

    //fn delete(&self, uri: &str) -> VolfResult<()> {
    //    match self.request::<()>(Method::Delete, uri, None) {
    //        Err(Error::Codec(_)) => Ok(()),
    //        otherwise => otherwise,
    //    }
    //}

    fn post<D>(&self, uri: &str, message: &[u8]) -> VolfResult<D>
    {
        self.request(Method::Post, uri, Some(message))
    }

    fn patch<D>(&self, uri: &str, message: &[u8]) -> VolfResult<D>
    {
        self.request(Method::Patch, uri, Some(message))
    }

    fn put<D>(&self, uri: &str, message: &[u8]) -> VolfResult<D>
    {
        self.request(Method::Put, uri, Some(message))
    }
}

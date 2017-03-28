extern crate volf;
extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;

use volf::config::Config;
use volf::server::{ServerHandle, PullRequestState};
use volf::client::Github;

use hyper::{Server, Client};

use std::env;
use std::sync::{Arc, Mutex};


fn main() {
    env_logger::init().unwrap();

    println!("# volf tests");

    println!("# has_config");
    has_config();
    println!("ok has_config");

    let limited: bool = env::var("TRAVIS_LIMITED_TESTS").unwrap_or("false".into()).parse().unwrap();

    if !limited {
        println!("# test_ping_event");
        test_ping_event();
        println!("ok test_ping_event");
    }
}

fn has_config() {
    let cfg = Config::read();
    assert!(cfg.is_ok(), "config was readable")
}


// TODO: do something like this..
//pub fn hook_update(&self, repo: &str, hook: u64) -> VolfResult<()> {
//    let uri = format!("repos/{}/hooks/{}", repo, hook);
//    let data = object!{
//        "content_type" => "json",
//        "url" => "http://109.146.235.224:54857/github",
//        "insecure_ssl" => "0",
//        "secret" => "hunter2"
//    };
//    self.patch(&uri, &json::stringify(data))?;
//    Ok(())
//}
//pub fn ping(&self, repo: &str, hook: u64) -> VolfResult<()> {
//    let uri = format!("repos/{}/hooks/{}/pings", repo, hook);
//    self.post(&uri, "")?;
//    Ok(())
//}

// Test API client and webhook server in one go (in a simple way)
fn test_ping_event() {
    use std::thread;
    use std::time::Duration;

    let token = env::var("GITHUB_TOKEN").unwrap();
    let hookid: u64 = env::var("VOLF_HOOK").unwrap().parse().unwrap();

    let cfg = Config::read().unwrap();
    let state: PullRequestState = Arc::new(Mutex::new(vec![]));
    let client = Client::new();
    let github = Arc::new(Github::new("volf-test", client, token));

    let addr = format!("0.0.0.0:{}", cfg.port);
    let srv = ServerHandle::new(state.clone(), github.clone());

    thread::spawn(move || { Server::http(&addr.as_str()).unwrap().handle(srv).unwrap(); });

    let r = github.ping("clux/volf", hookid);
    assert!(r.is_ok(), "could authenticate and ping our hook");
    // wait for github to forward event to this
    thread::sleep(Duration::from_millis(500));
}

extern crate volf;
extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;

use volf::config::Config;
use volf::server::{ServerHandle, PullRequestState};
use volf::client::{Github, Credentials};

use hyper::{Server, Client};

use std::env;
use std::sync::{Arc, Mutex};


fn main() {
    env_logger::init().unwrap();

    println!("# volf tests");

    println!("# has_config");
    has_config();
    println!("ok has_config");

    let token = env::var("GITHUB_TOKEN").unwrap();
    let hook : u64 = env::var("VOLF_HOOK").unwrap().parse().unwrap();
    let limited : bool = env::var("TRAVIS_LIMITED_TESTS").unwrap_or("false".into()).parse().unwrap();

    if !limited {
        println!("# test_ping_event");
        test_ping_event(token, hook);
        println!("ok test_ping_event");
    }
}

fn has_config() {
    let cfg = Config::read();
    assert!(cfg.is_ok(), "config was readable")
}

// Test API client and webhook server in one go (in a simple way)
fn test_ping_event(token: String, hookid: u64) {
    use std::thread;
    use std::time::Duration;

    let cfg = Config::read().unwrap();
    let state: PullRequestState = Arc::new(Mutex::new(vec![]));
    let client = Client::new();
    let github = Arc::new(Github::new("volf-test",
        client,
        Credentials::Token(token)));

    let addr = format!("0.0.0.0:{}", cfg.port);
    let srv = ServerHandle::new(state.clone(), github.clone());

    thread::spawn(move || {
        Server::http(&addr.as_str()).unwrap().handle(srv).unwrap();
    });

    let r = github.ping("clux/volf", hookid);
    assert!(r.is_ok(), "could ping our hook");
    // wait for github to forward event to this
    thread::sleep(Duration::from_millis(500));
}

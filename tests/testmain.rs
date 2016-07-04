extern crate volf;
extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;

use volf::Config;
use volf::github::{Hub, Push, PullRequest, IssueComment};

use hyper::Server;

use std::env;


fn main() {
    let num_tests = 1;

    println!("# volf tests");
    println!("1..{}", num_tests);
    let mut i = 0;

    i += 1;
    has_config();
    println!("ok {} has_config", i);

    i += 1;
    test_ping_event();
    println!("ok {} test_ping_event", i);
}

fn has_config() {
    let cfg = Config::read();
    assert!(cfg.is_ok(), "config was readable")
}

fn test_ping_event() {
    use std::thread;
    use std::time::Duration;

    let child = thread::spawn(|| {
        let cfg = Config::read().unwrap();
        let mut hub = Hub::new();
        // TODO: ping handler
        let addr = format!("0.0.0.0:{}", cfg.port);
        //let srv = Server::http(&addr.as_str()).unwrap().handle(hub);
    });
    // TODO: perform github request to hit the ping hook
    thread::sleep(Duration::from_millis(500));
    // Then close the server.. probably need to Arc up the server;
}

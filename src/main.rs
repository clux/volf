#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;

use hyper::{Server, Client};

extern crate volf;
use volf::config::Config;
use volf::server::{ServerHandle, PullRequestState};
use volf::client::{Github, Credentials};

use clap::{Arg, App, AppSettings};
use std::process;
use std::sync::{Arc, Mutex};
use std::env;

fn main() {
    let args = App::new("volf")
        .version(crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("volf")
        .arg(Arg::with_name("synchronize")
            .short("s")
            .long("synchronize")
            .help("Re-synchronize github state before starting"))
        .get_matches();

    env_logger::init().unwrap();

    // Force config to exists before allowing remaining actions
    let config = Config::read()
        .map_err(|e| {
            error!("Configuration error: {}", e);
            println!("Ensure you have volf.json is valid");
            process::exit(1);
        })
        .unwrap();

    // Create a github client from our credentials
    let token = env::var("GITHUB_TOKEN")
        .map_err(|_| {
            error!("Missing GITHUB_TOKEN environment variable");
            process::exit(1)
        })
        .unwrap();
    let client = Client::new();
    let github = Github::new(format!("volf/{}", crate_version!()),
                             &client,
                             Credentials::Token(token));

    // Application state is just a shared vector of PRs
    let state: PullRequestState = Arc::new(Mutex::new(vec![]));

    // Synchronize state before starting the server if requested
    if args.is_present("synchronize") {
        unimplemented!();
    }

    // Set up webhook server
    let srv = ServerHandle::new(state.clone());
    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);
    Server::http(&addr.as_str()).unwrap().handle(srv).unwrap();
}

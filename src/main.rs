#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate reroute;

use hyper::Server;
use hyper::server::{Request, Response};
use reroute::{Captures, Router};

extern crate volf;
use volf::{Config, Pull};
use volf::github::{Hub, Push, PullRequest, IssueComment, Ping};

use clap::{Arg, App, AppSettings};
use std::process;
use std::sync::Arc;

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

    // Synchronize before starting the server if requested
    if args.is_present("synchronize") {
        unimplemented!();
    }
    let state : Arc<Vec<Pull>> = Arc::new(vec![]);

    // Start webhook server
    let mut hub = Hub::new();
    hub.on_push(|data: &Push| {
        info!("got push {:?}", data);
    });
    hub.on_pull_request(|data: &PullRequest| {
        info!("got pr {:?}", data);
    });
    hub.on_issue_comment(|data: &IssueComment| {
        info!("got issue comment {:?}", data);
    });
    hub.on_ping(|data: &Ping| {
        info!("Ping - {}", data.zen);
    });

    let mut router = Router::new();
    router.post(r"/github", move |req: Request, res: Response, _: Captures| {
        hub.handler(req, res)
    });
    router.finalize().unwrap();

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);
    Server::http(&addr.as_str()).unwrap().handle(router).unwrap();
}

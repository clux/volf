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
use volf::{Config, PullRequestState};
use volf::github::webhook_handler;

use clap::{Arg, App, AppSettings};
use std::process;
use std::sync::{Arc, Mutex};

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

    // God object
    let state: Arc<PullRequestState> = Arc::new(Mutex::new(vec![]));

    // Synchronize state before starting the server if requested
    if args.is_present("synchronize") {
        unimplemented!();
    }

    // Multiplex routes with reroute
    let mut router = Router::new();
    router.post(r"/github",
                move |req: Request, res: Response, _: Captures| {
                    webhook_handler(&state.clone(), req, res)
                });
    router.finalize().unwrap();

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);
    Server::http(&addr.as_str()).unwrap().handle(router).unwrap();
}

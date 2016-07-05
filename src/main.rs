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
use volf::{Config, Pull, PullRequestState};
use volf::github::{Hub, Push, PullRequest, IssueComment, Ping};

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
    let main_state : Arc<PullRequestState> = Arc::new(Mutex::new(vec![]));

    // Synchronize state before starting the server if requested
    if args.is_present("synchronize") {
        unimplemented!();
    }

    // Handle webhook events
    let mut hub = Hub::new(main_state);
    hub.on_push(|_: &PullRequestState, data: &Push| {
        info!("got push {:?}", data);
        // TODO: need ref here to match up with a pr
    });
    hub.on_pull_request(|state: &PullRequestState, data: &PullRequest| {
        info!("got pr {:?}", data);
        let prdata = &data.pull_request;
        if data.action == "opened" || data.action == "reopened" {
            let pr = Pull::new(&data.repository.full_name, &prdata.title, data.number);
            let mut prs = state.lock().unwrap();
            prs.push(pr);
        }
    });
    hub.on_issue_comment(|state: &PullRequestState, data: &IssueComment| {
        info!("got issue comment {:?}", data);
        if let Some(ref prdata) = data.issue.pull_request {
            let mut prs = state.lock().unwrap();
            if let Some(pr) = prs.iter().find(|&pr| pr.num == prdata.number) {
                info!("found corresponding pr {}", pr.num);
            }
            else {
                warn!("ignoring comment on untracked pr {}", prdata.number);
            }
        }
    });
    hub.on_ping(|_: &PullRequestState, data: &Ping| {
        info!("Ping - {}", data.zen);
    });

    // Multiplex routes with reroute
    let mut router = Router::new();
    router.post(r"/github", move |req: Request, res: Response, _: Captures| {
        hub.handler(req, res) // defer to Hub instance entirely
    });
    router.finalize().unwrap();

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);
    Server::http(&addr.as_str()).unwrap().handle(router).unwrap();
}

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;

extern crate hubcaps;
extern crate hyper;
extern crate hyper_native_tls;

use hyper::{Server, Client};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hubcaps::{Credentials, Github};


extern crate volf;
use volf::config::Config;
use volf::server::{ServerHandle, PullRequestState};

use clap::{Arg, App, AppSettings, SubCommand};
use std::process;
use std::sync::{Arc, Mutex};
use std::env;
use std::thread;

fn result_exit<T, E>(name: &str, x: Result<T, E>)
    where E: std::fmt::Display
{
    let _ = x.map_err(|e| {
        println!(""); // add a separator
        error!("{} error: {}", name, e);
        process::exit(1);
    });
    process::exit(0);
}

fn main() {
    let args = App::new("volf")
        .about("Github webhook server and CI control bot")
        .version(crate_version!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::DeriveDisplayOrder)
        .global_settings(&[AppSettings::ColoredHelp, AppSettings::ColorAuto])
        .subcommand(SubCommand::with_name("start")
            .about("Start volf server")
            .alias("run")
            .arg(Arg::with_name("synchronize")
                .short("s")
                .long("synchronize")
                .help("Re-synchronize github state before starting")))
        .subcommand(SubCommand::with_name("config")
            .about("Generate or edit the local config")
            .subcommand(SubCommand::with_name("edit")
                .about("Open the local config with $EDITOR"))
            .subcommand(SubCommand::with_name("generate")
                .about("Generate an empty config in current directory")))
        .get_matches();

    env_logger::init().unwrap();

    if let Some(cfgargs) = args.subcommand_matches("config") {
        if let Some(_) = cfgargs.subcommand_matches("generate") {
            result_exit("generate", Config::generate());
        }
        if let Some(_) = cfgargs.subcommand_matches("edit") {
            result_exit("edit", Config::edit())
        }
    }

    // Force config to exists before allowing remaining actions
    let config = Config::read()
        .map_err(|e| {
            error!("Configuration error: {}", e);
            println!("Ensure you have volf.json is valid");
            process::exit(1);
        })
        .unwrap();

    // Create a github client from our credentials
    // TODO: env secrets -> struct (there's a nice crate for it)
    let token = env::var("GITHUB_TOKEN")
        .map_err(|_| {
            error!("Missing GITHUB_TOKEN environment variable");
            process::exit(1)
        })
        .unwrap();

    let github =
        Arc::new(Github::new(format!("volf/{}", crate_version!()),
                             Client::with_connector(HttpsConnector::new(NativeTlsClient::new()
                                                                            .unwrap())),
                             Credentials::Token(token)));

    // Application state is just a shared vector of PRs
    let prs: PullRequestState = Arc::new(Mutex::new(vec![]));

    let serverargs = args.subcommand_matches("start").unwrap();
    // Synchronize state before starting the server if requested
    if serverargs.is_present("synchronize") {
        for repo in &config.repositories {
            let _ = repo.synchronize(github.clone(), &repo.name);
        }
    }

    // Set up webhook server
    let port = config.port;
    let srv = ServerHandle::new(prs.clone(), github, Arc::new(config));
    // Start pull request queue thread first on the server object
    //TODO: may be able to just run this off events in main webhook handler?
    let srv2 = srv.clone();
    thread::spawn(move || { srv2.queue(); });
    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);
    Server::http(&addr.as_str()).unwrap().handle(srv).unwrap();
}

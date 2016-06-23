#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;

extern crate afterparty;
extern crate hyper;

extern crate volf;
use volf::Config;

use hyper::Server;
use afterparty::{Delivery, Event, Hub};

use clap::{Arg, App, AppSettings};
use std::process;

fn main() {
    let args = App::new("volf")
        .version(crate_version!())
        .setting(AppSettings::ColoredHelp)
        .about("volf")
        .arg(Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .help("Use verbose output"))
        .arg(Arg::with_name("synchronize")
            .short("s")
            .long("synchronize")
            .help("Re-synchronize github state before starting"))
        .get_matches();

    // by default, always show INFO messages for now (+1)
    loggerv::init_with_verbosity(args.occurrences_of("verbose") + 1).unwrap();

    // Force config to exists before allowing remaining actions
    let config = Config::read()
        .map_err(|e| {
            error!("Configuration error: {}", e);
            println!("Ensure you have volf.toml is valid");
            process::exit(1);
        })
        .unwrap();

    // Synchronize before starting the server if requested
    if args.is_present("synchronize") {
        unimplemented!();
    }

    // Start webhook server
    let port = 54857; // TODO: put in cfg
    let addr = format!("0.0.0.0:{}", port);
    let mut hub = Hub::new();
    hub.handle("push", |delivery: &Delivery| {
        info!("rec delivery {:#?}", delivery);
        match delivery.payload {
            Event::Push { ref after, ref sender, .. } => {
                info!("sender {} after {}", sender.login, after)
            }
            _ => (),
        }
    });
    let srvc = Server::http(&addr[..])
        .unwrap()
        .handle(hub);
    info!("Listening on {}", addr);
    srvc.unwrap();

}

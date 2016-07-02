#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate pencil;

extern crate volf;
use volf::Config;

use pencil::Pencil;

use clap::{Arg, App, AppSettings};
use std::process;

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

    // Start webhook server
    let mut app = Pencil::new("/");
    app.post("/github", "github", volf::github::hook);

    let addr = format!("0.0.0.0:{}", config.port);
    info!("Listening on {}", addr);
    app.run(addr.as_str());
}

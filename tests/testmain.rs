extern crate volf;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate pencil;

use volf::Config;

use std::env;


fn main() {
    let num_tests = 1;

    println!("# volf tests");
    println!("1..{}", num_tests);
    let mut i = 0;

    i += 1;
    has_config();
    println!("ok {} has_config", i);

    //i += 1;
    //test_ping_event();
    //println!("ok {} test_ping_event", i);
}

fn has_config() {
    let cfg = Config::read();
    assert!(cfg.is_ok(), "config was readable")
}

fn test_ping_event() {
    use pencil::Pencil;
    use std::thread;
    use std::time::Duration;

    let child = thread::spawn(|| {
        let cfg = Config::read().unwrap();
        let mut app = Pencil::new("/");
        app.post("/github", "github", volf::github::hook);
        let addr = format!("0.0.0.0:{}", cfg.port);
        app.run(addr.as_str());
    });
    // TODO: perform github request to hit the ping hook
    //thread::sleep(Duration::from_millis(500));
    // would like to close the server  after this, but not supported by pencil
    // https://github.com/fengsp/pencil/issues/26
}

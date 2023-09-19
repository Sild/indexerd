#[macro_use]
extern crate indexerd_derive;
extern crate ctrlc;
extern crate hwloc2;
extern crate log;
extern crate mysql;

mod config;
mod data;
mod engine;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod helpers;
mod server;
mod task;
mod worker;
use log::LevelFilter;

use crate::server::Server;

fn main() -> std::io::Result<()> {
    // init logger first
    let mut builder = env_logger::Builder::from_default_env();
    if std::env::var("RUST_LOG").is_err() {
        // override default 'error'
        builder.filter_level(LevelFilter::Debug);
    }
    builder.init();

    // get config then
    let mut conf_path = String::from("configs/dev.json");
    let args: Vec<_> = std::env::args().collect();
    if args.len() > 1 {
        conf_path = args.get(1).expect("fail to extract argument").clone()
    }
    log::info!("Using config from path={}", conf_path);

    // let c1 = objects::Campaign::from_select();
    // let p1 = objects::Package::from_select();
    // let pad1 = objects::Pad::from_select();
    //
    // let mut data_manager = data::data_manager::DataManager::default();
    // data_manager.insert(c1);
    // data_manager.insert(p1);
    // data_manager.insert(pad1);

    let server_conf = config::Server::from_file(conf_path.as_str())?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_ctrlc = shutdown.clone();
    let server = Server::new(&server_conf, shutdown)?;
    ctrlc::set_handler(move || {
        log::info!("received SIGINT");
        shutdown_ctrlc.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    server.wait_shutdown()
}

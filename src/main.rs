#[macro_use]
extern crate indexerd_derive;
extern crate ctrlc;
extern crate hwloc;
extern crate log;
mod config;
mod db;
mod engine;
mod objects;
mod store;

use hwloc::Topology;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use mysql_cdc::errors::Error;
mod helpers;
mod request;
mod server;
mod worker;
use log::LevelFilter;

// use crate::objects::MysqlObject;
use crate::server::Server;

fn _check_cpu() {
    let topo = Topology::new();

    // Check if Process Binding for CPUs is supported
    println!(
        "CPU Binding (current process) supported: {}",
        topo.support().cpu().set_current_process()
    );
    println!(
        "CPU Binding (any process) supported: {}",
        topo.support().cpu().set_process()
    );
    // Check if Thread Binding for CPUs is supported
    println!(
        "CPU Binding (current thread) supported: {}",
        topo.support().cpu().set_current_thread()
    );
    println!(
        "CPU Binding (any thread) supported: {}",
        topo.support().cpu().set_thread()
    );

    // Debug Print all the Support Flags
    println!("All Flags:\n{:?}", topo.support());
}

fn main() -> Result<(), Error> {
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

    // check_cpu();

    // let c1 = objects::Campaign::from_select();
    // let p1 = objects::Package::from_select();
    // let pad1 = objects::Pad::from_select();
    //
    // let mut data_manager = store::data_manager::DataManager::default();
    // data_manager.insert(c1);
    // data_manager.insert(p1);
    // data_manager.insert(pad1);

    let mut server = Server::new(8089, 8088)?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_local = shutdown.clone();
    ctrlc::set_handler(move || {
        log::info!("received SIGINT");
        shutdown_local.store(true, Ordering::Relaxed);
        server.shutdown().expect("fail to shutdown server properly");
    })
    .expect("Error setting Ctrl-C handler");

    db::run_slave(shutdown, conf_path)
}

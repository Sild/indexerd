#[macro_use]
extern crate indexerd_derive;
extern crate ctrlc;
extern crate hwloc;
mod config;
mod db;
mod engine;
mod objects;
mod store;
use hwloc::Topology;

use mysql_cdc::errors::Error;
mod helpers;
mod request;
mod server;
mod worker;

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
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        server.shutdown().expect("fail to shutdown server properly");
    })
    .expect("Error setting Ctrl-C handler");

    db::run_slave()
}
